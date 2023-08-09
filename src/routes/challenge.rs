
use actix_web::{web, HttpResponse};
use sqlx::{query, PgPool};

#[derive(serde::Serialize)]
pub struct ResponseData {
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
        Err(_) => return HttpResponse::BadRequest().json("Invalid token"),
    };
    match retrieve_challenge(&pool, &token).await {
        Ok(response_data) => HttpResponse::Ok().json(response_data),
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!("Row not found: {:?}", token);
            HttpResponse::NotFound().finish()},
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
pub async fn retrieve_challenge(pool: &PgPool, token: &uuid::Uuid) -> Result<ResponseData, sqlx::Error> {
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

    Ok(ResponseData {
        challenge: record.challenge,
    })
}
