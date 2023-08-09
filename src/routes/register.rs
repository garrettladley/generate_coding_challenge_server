use std::time::SystemTime;

use crate::domain::{generate_challenge, ApplicantName, Color, Nuid, RegisterApplicant};

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
        let name = ApplicantName::parse(body.name)?;
        let nuid = Nuid::parse(body.nuid)?;
        Ok(RegisterApplicant { name, nuid })
    }
}

#[derive(serde::Serialize)]
pub struct ResponseData {
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
            HttpResponse::InternalServerError().finish()
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
) -> Result<ResponseData, sqlx::Error> {
    let registration_time: DateTime<Utc> = SystemTime::now().into();
    let token = Uuid::new_v4();
    let (challenge, solution) = generate_challenge(
        register_applicant.nuid.as_ref(),
        100,
        vec![
            String::from(""),
            Color::Red.to_string(),
            Color::Orange.to_string(),
            Color::Yellow.to_string(),
            Color::Green.to_string(),
            Color::Blue.to_string(),
            Color::Violet.to_string(),
        ],
    );

    query!(
        r#"INSERT INTO applicants (nuid, applicant_name, registration_time, token, challenge, solution)
        VALUES ($1, $2, $3, $4, $5, $6);"#,
        register_applicant.nuid.as_ref(),
        register_applicant.name.as_ref(),
        registration_time,
        &token,
        &challenge,
        &solution,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(ResponseData {
        token: token.to_string(),
        challenge,
    })
}
