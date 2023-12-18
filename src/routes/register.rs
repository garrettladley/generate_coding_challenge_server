use std::time::SystemTime;

use crate::domain::{generate_challenge, ApplicantName, Nuid, RegisterApplicant};

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use sqlx::{query, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct BodyData {
    pub name: String,
    pub nuid: String,
}

impl TryFrom<BodyData> for RegisterApplicant {
    type Error = String;

    fn try_from(body: BodyData) -> Result<Self, Self::Error> {
        let name = ApplicantName::parse(&body.name)?;
        let nuid = Nuid::parse(&body.nuid)?;
        Ok(RegisterApplicant { name, nuid })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterResponseData {
    pub token: String,
    pub challenge: Vec<String>,
}

#[tracing::instrument(
    name = "Adding a new applicant.",
    skip(body, pool),
    fields(
        applicant_name = %body.name,
        applicant_nuid = %body.nuid
    )
)]
pub async fn register(body: web::Json<BodyData>, pool: web::Data<PgPool>) -> HttpResponse {
    let register_applicant = match body.0.try_into() {
        Ok(register_applicant) => register_applicant,
        Err(err) => {
            tracing::error!(err);
            return HttpResponse::BadRequest().json(err);
        }
    };
    match insert_applicant(&pool, &register_applicant).await {
        Ok(response_data) => HttpResponse::Ok().json(response_data),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            match e {
                sqlx::Error::Database(db_err) => {
                    if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                        HttpResponse::Conflict().body(format!("NUID {} has already registered! Use the forgot-token endpoint to retrieve your token.", register_applicant.nuid.as_ref()))
                    } else {
                        HttpResponse::InternalServerError()
                            .body(format!("Database error: {:?}", db_err))
                    }
                }
                _ => HttpResponse::InternalServerError().body(format!("SQLX error: {:?}", e)),
            }
        }
    }
}

#[tracing::instrument(
    name = "Saving new applicant details in the database.",
    skip(register_applicant, pool)
)]
pub async fn insert_applicant(
    pool: &PgPool,
    register_applicant: &RegisterApplicant,
) -> Result<RegisterResponseData, sqlx::Error> {
    let registration_time: DateTime<Utc> = SystemTime::now().into();
    let token = Uuid::new_v4();
    let challenge = generate_challenge(
        256,
        vec![
            String::from(""),
            String::from("#12#34!#59^#67%#"),
            String::from("#12^!%%###34^#"),
            String::from("##"),
        ],
    );

    query!(
        r#"INSERT INTO applicants (nuid, applicant_name, registration_time, token, challenge, solution)
        VALUES ($1, $2, $3, $4, $5, $6);"#,
        register_applicant.nuid.as_ref(),
        register_applicant.name.as_ref(),
        registration_time,
        &token,
        &challenge.challenge,
        &challenge.solution,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(RegisterResponseData {
        token: token.to_string(),
        challenge: challenge.challenge,
    })
}
