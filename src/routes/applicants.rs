use actix_web::{web, HttpResponse};
use sqlx::{query, PgPool};
use std::time::Duration;

use crate::domain::{ApplicantName, Nuid};

#[derive(serde::Deserialize)]
pub struct ApplicantsBodyData(Vec<String>);

impl TryFrom<ApplicantsBodyData> for Nuids {
    type Error = String;

    fn try_from(body: ApplicantsBodyData) -> Result<Self, Self::Error> {
        let mut failed_parses = Vec::new();
        let nuids = body
            .0
            .into_iter()
            .filter_map(|nuid| match Nuid::parse(&nuid.clone()) {
                Ok(parsed_nuid) => Some(parsed_nuid),
                Err(_) => {
                    failed_parses.push(nuid);
                    None
                }
            })
            .collect::<Vec<_>>();

        if failed_parses.is_empty() {
            Ok(Nuids(nuids))
        } else {
            Err(format!(
                "Failed to parse the following NUIDs: {:?}",
                failed_parses
            ))
        }
    }
}

pub struct Nuids(Vec<Nuid>);

impl std::fmt::Display for Nuids {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|nuid| nuid.as_ref())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct IntermediateApplicant {
    pub nuid: String,
    pub name: String,
    pub correct: bool,
    pub time_to_completion: Duration,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReturnedApplicant {
    pub nuid: Nuid,
    pub name: ApplicantName,
    pub correct: bool,
    pub time_to_completion: Duration,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApplicantsResponseData {
    pub applicants_found: Vec<ReturnedApplicant>,
    pub applicants_not_submitted: Vec<String>,
    pub applicants_not_found: Vec<String>,
}

#[tracing::instrument(
    name = "Fetching applicants.",
    skip(body, pool),
    fields(
        nuids = %body.0.0.join(", ")
    )
)]
pub async fn applicants(
    body: web::Json<ApplicantsBodyData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let nuids = match body.0.try_into() {
        Ok(nuids) => nuids,
        Err(err) => {
            tracing::error!(err);
            return HttpResponse::BadRequest().body(err);
        }
    };
    match select_applicants(&pool, &nuids).await {
        Ok(applicants) => {
            let mut returned_applicants = Vec::with_capacity(applicants.len());

            for applicant in applicants {
                let nuid_result = Nuid::parse(&applicant.nuid);
                let name_result = ApplicantName::parse(&applicant.name);

                match (nuid_result, name_result) {
                    (Ok(nuid), Ok(name)) => {
                        returned_applicants.push(ReturnedApplicant {
                            nuid,
                            name,
                            correct: applicant.correct,
                            time_to_completion: applicant.time_to_completion,
                        });
                    }
                    (Err(_), _) => {
                        tracing::error!(
                            "Invalid database state for NUID! Given: {}",
                            applicant.nuid
                        );
                        return HttpResponse::InternalServerError().finish();
                    }
                    (_, Err(_)) => {
                        tracing::error!(
                            "Invalid database state for name! Given: {}",
                            applicant.name
                        );
                        return HttpResponse::InternalServerError().finish();
                    }
                }
            }

            if returned_applicants.len() != nuids.0.len() {
                let remaining_nuids = nuids
                    .0
                    .iter()
                    .filter(|nuid| {
                        !returned_applicants
                            .iter()
                            .any(|applicant| applicant.nuid.as_ref() == nuid.as_ref())
                    })
                    .collect::<Vec<_>>();

                let mut applicants_not_submitted = Vec::with_capacity(remaining_nuids.len());
                let mut applicants_not_found = Vec::with_capacity(remaining_nuids.len());

                for nuid in remaining_nuids {
                    match check_if_applicant_exists(&pool, nuid).await {
                        Ok(true) => {
                            applicants_not_submitted.push(nuid.to_string());
                        }
                        Ok(false) => {
                            applicants_not_found.push(nuid.to_string());
                        }
                        Err(_) => {
                            tracing::error!(
                                "Error while checking if applicant with NUID of {} exists!",
                                nuid
                            );
                            return HttpResponse::InternalServerError().finish();
                        }
                    }
                }

                if applicants_not_found.is_empty() {
                    HttpResponse::Ok().json(ApplicantsResponseData {
                        applicants_found: returned_applicants,
                        applicants_not_submitted,
                        applicants_not_found: Vec::new(),
                    })
                } else {
                    HttpResponse::NotFound().json(ApplicantsResponseData {
                        applicants_found: returned_applicants,
                        applicants_not_submitted,
                        applicants_not_found,
                    })
                }
            } else {
                HttpResponse::Ok().json(ApplicantsResponseData {
                    applicants_found: returned_applicants,
                    applicants_not_submitted: Vec::new(),
                    applicants_not_found: Vec::new(),
                })
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            let applicants_not_found: Vec<String> =
                nuids.0.iter().map(|nuid| nuid.to_string()).collect();

            HttpResponse::NotFound().json(ApplicantsResponseData {
                applicants_found: Vec::new(),
                applicants_not_submitted: Vec::new(),
                applicants_not_found,
            })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Fetching applicant details from the database.",
    skip(nuids, pool)
)]
pub async fn select_applicants(
    pool: &PgPool,
    nuids: &Nuids,
) -> Result<Vec<IntermediateApplicant>, sqlx::Error> {
    let nuids: Vec<String> = nuids.0.iter().map(|nuid| nuid.to_string()).collect();

    let records = query!(
        r#"SELECT DISTINCT ON (nuid) nuid, applicant_name, correct, submission_time, 
        registration_time FROM submissions JOIN applicants using(nuid) where 
        nuid=ANY($1) ORDER BY nuid, submission_time DESC;"#,
        &nuids.as_slice()
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .iter()
        .map(|record| {
            let time_to_completion = record
                .submission_time
                .signed_duration_since(record.registration_time)
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(0));

            IntermediateApplicant {
                nuid: record.nuid.clone(),
                name: record.applicant_name.clone(),
                correct: record.correct,
                time_to_completion,
            }
        })
        .collect())
}

pub async fn check_if_applicant_exists(pool: &PgPool, nuid: &Nuid) -> Result<bool, sqlx::Error> {
    let record = query!(
        r#"SELECT nuid FROM applicants where nuid=$1;"#,
        nuid.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.is_some())
}
