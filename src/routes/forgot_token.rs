use crate::domain::Nuid;

use actix_web::{web, HttpResponse};
use sqlx::{query, PgPool};

#[derive(serde::Serialize)]
pub struct ForgotTokenResponseData {
    pub token: String,
}

#[tracing::instrument(
    name = "Forgot token.",
    skip(nuid, pool),
    fields(
        applicant_nuid = %nuid
    )
)]
pub async fn forgot_token(nuid: web::Path<String>, pool: web::Data<PgPool>) -> HttpResponse {
    let nuid = match Nuid::parse(&nuid) {
        Ok(nuid) => nuid,
        Err(err) => {
            tracing::error!(err);
            return HttpResponse::BadRequest().json(err);
        }
    };
    match retrieve_token(&pool, &nuid).await {
        Ok(response_data) => HttpResponse::Ok().body(response_data.token),
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!(
                "Record associated with given NUID not found! NUID: {}",
                nuid
            );
            HttpResponse::NotFound().json(format!(
                "Record associated with given NUID not found! NUID: {}",
                nuid
            ))
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Fetching applicant token from the database.", skip(nuid, pool))]
pub async fn retrieve_token(
    pool: &PgPool,
    nuid: &Nuid,
) -> Result<ForgotTokenResponseData, sqlx::Error> {
    let record = query!(
        r#"SELECT token FROM applicants WHERE nuid=$1"#,
        nuid.as_ref()
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    if record.token.to_string().is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(ForgotTokenResponseData {
        token: record.token.to_string(),
    })
}
