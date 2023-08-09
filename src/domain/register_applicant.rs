use crate::domain::ApplicantName;
use crate::domain::Nuid;

pub struct RegisterApplicant {
    pub name: ApplicantName,
    pub nuid: Nuid,
}
