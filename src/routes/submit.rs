use std::time::SystemTime;

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use sqlx::{query, PgPool};

use crate::domain::Nuid;

#[derive(serde::Deserialize)]
pub struct BodyData(Vec<String>);

impl std::fmt::Display for BodyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

impl AsRef<Vec<String>> for BodyData {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

#[derive(serde::Serialize)]
pub struct ResponseData {
    pub correct: bool,
    pub message: String,
}

pub struct IntermediarySolution {
    pub nuid: String,
    pub actual_solution: Vec<String>,
}

pub struct SolutionToBeChecked {
    pub nuid: Nuid,
    pub solution: Vec<String>,
}

#[tracing::instrument(
    name = "Submit challenge.",
    skip(token, body, pool),
    fields(
        applicant_token = %token,
        applicant_solution = %body
    )
)]
pub async fn submit(
    token: web::Path<String>,
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let token = match uuid::Uuid::parse_str(&token.into_inner()) {
        Ok(token) => token,
        Err(token) => {
            tracing::error!("Invalid token! Given: \"{:?}\"", token);
            return HttpResponse::BadRequest()
                .json(format!("Invalid token! Given: \"{:?}\"", token));
        }
    };

    let solution_to_be_checked = match retrieve_solution(&pool, &token).await {
        Ok(intermediary_solution) => {
            let nuid = match Nuid::parse(intermediary_solution.nuid) {
                Ok(nuid) => nuid,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            SolutionToBeChecked {
                nuid,
                solution: intermediary_solution.actual_solution,
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!("Row not found: {:?}", token);
            return HttpResponse::NotFound().json(format!(
                "Record associated with given token not found! Token: {}",
                token
            ));
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let correct = solution_to_be_checked.solution == *body.as_ref();
    match write_submission(&pool, &solution_to_be_checked.nuid, &correct).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    let response_data = ResponseData {
        correct,
        message: if correct {
            "Correct - nice work!".to_string()
        } else {
            "Incorrect Solution".to_string()
        },
    };

    HttpResponse::Ok().json(response_data)
}

#[tracing::instrument(
    name = "Fetching applicant solution from the database.",
    skip(token, pool)
)]
pub async fn retrieve_solution(
    pool: &PgPool,
    token: &uuid::Uuid,
) -> Result<IntermediarySolution, sqlx::Error> {
    let record = query!(
        r#"SELECT nuid, solution FROM applicants WHERE token=$1"#,
        token
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    if record.nuid.is_empty() && record.solution.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(IntermediarySolution {
        nuid: record.nuid,
        actual_solution: record.solution,
    })
}

#[tracing::instrument(
    name = "Saving applicant submission to the database.",
    skip(pool, nuid, correct)
)]
pub async fn write_submission(
    pool: &PgPool,
    nuid: &Nuid,
    correct: &bool,
) -> Result<(), sqlx::Error> {
    let submission_time: DateTime<Utc> = SystemTime::now().into();

    query!(
        r#"INSERT INTO submissions (nuid, correct, submission_time) VALUES ($1, $2, $3);"#,
        nuid.as_ref(),
        correct,
        submission_time,
    )
    .execute(pool)
    .await?;

    Ok(())
}
