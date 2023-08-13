use crate::domain::ApplicantName;
use crate::domain::Nuid;

#[derive(serde::Serialize)]
pub struct RegisterApplicant {
    pub name: ApplicantName,
    pub nuid: Nuid,
}
