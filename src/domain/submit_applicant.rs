use std::time::Duration;

use crate::domain::register_applicant::RegisterApplicant;

#[derive(serde::Serialize)]
pub struct SubmitApplicant {
    pub time_to_completion: Duration,
    pub ok: bool,
    pub applicant: RegisterApplicant,
}
