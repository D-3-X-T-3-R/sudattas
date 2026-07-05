//! Shared conversions for the customer-editable profile fields (gender, date of birth)
//! used across create/update/search/delete/export user handlers.

use chrono::NaiveDate;
use core_db_entities::entity::sea_orm_active_enums::Gender;
use tonic::Status;

pub(crate) fn parse_gender(raw: &str) -> Result<Gender, Status> {
    match raw.trim().to_lowercase().as_str() {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        "other" => Ok(Gender::Other),
        _ => Err(Status::invalid_argument(
            "Invalid gender; expected male, female, or other",
        )),
    }
}

pub(crate) fn gender_to_string(gender: &Gender) -> String {
    match gender {
        Gender::Male => "male".to_string(),
        Gender::Female => "female".to_string(),
        Gender::Other => "other".to_string(),
    }
}

pub(crate) fn parse_date_of_birth(raw: &str) -> Result<NaiveDate, Status> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| Status::invalid_argument("Invalid date_of_birth; expected yyyy-mm-dd"))
}
