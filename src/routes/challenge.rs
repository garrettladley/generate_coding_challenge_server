use actix_web::{web, HttpResponse};
use sqlx::{query, PgPool};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ChallengeResponseData {
    pub challenge: Vec<String>,
}

#[tracing::instrument(
    name = "Forgot challenge.",
    skip(token, pool),
    fields(
        applicant_token = %token
    )
)]
pub async fn challenge(token: web::Path<String>, pool: web::Data<PgPool>) -> HttpResponse {
    let token = match uuid::Uuid::parse_str(&token) {
        Ok(token) => token,
        Err(_) => {
            tracing::error!("Invalid token! Given: {}", token);
            return HttpResponse::BadRequest().body(format!("Invalid token! Given: {}", token));
        }
    };
    match retrieve_challenge(&pool, &token).await {
        Ok(response_data) => HttpResponse::Ok().json(response_data),
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!("Row not found: {:?}", token);
            HttpResponse::NotFound().body(format!(
                "Record associated with given token not found! Token: {}",
                token
            ))
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Fetching applicant challenge from the database.",
    skip(token, pool)
)]
pub async fn retrieve_challenge(
    pool: &PgPool,
    token: &uuid::Uuid,
) -> Result<ChallengeResponseData, sqlx::Error> {
    let record = query!(r#"SELECT challenge FROM applicants WHERE token=$1"#, token)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    if record.challenge.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(ChallengeResponseData {
        challenge: record.challenge,
    })
}
