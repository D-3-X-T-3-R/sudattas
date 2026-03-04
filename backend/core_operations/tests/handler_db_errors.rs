//! Unit tests for db_errors mapping functions.

use core_operations::auth::AuthError;
use core_operations::handlers::db_errors::{map_auth_error_to_status, map_db_error_to_status};
use sea_orm::DbErr;

#[test]
fn map_auth_error_to_status_covers_all_variants() {
    let status = map_auth_error_to_status(AuthError::HashingError("oops".into()));
    assert_eq!(status.code(), tonic::Code::Internal);

    let status = map_auth_error_to_status(AuthError::VerificationFailed);
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    let status = map_auth_error_to_status(AuthError::InvalidHashFormat);
    assert_eq!(status.code(), tonic::Code::Internal);

    let status = map_auth_error_to_status(AuthError::UserNotFound);
    assert_eq!(status.code(), tonic::Code::NotFound);

    let status = map_auth_error_to_status(AuthError::InvalidCredentials);
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    let status = map_auth_error_to_status(AuthError::AccountInactive);
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);

    let status = map_auth_error_to_status(AuthError::AccountLocked);
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
}

#[test]
fn map_db_error_to_status_maps_not_found_and_custom() {
    let status = map_db_error_to_status(DbErr::RecordNotFound("missing".into()));
    assert_eq!(status.code(), tonic::Code::NotFound);

    let status = map_db_error_to_status(DbErr::Custom("boom".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}
