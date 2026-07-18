use crate::auth::AuthError;
use log::error;
use sea_orm::DbErr;
use tonic::Status;

/// Maps auth errors to gRPC Status for use in handlers.
pub fn map_auth_error_to_status(auth_error: AuthError) -> Status {
    use AuthError::*;
    match auth_error {
        HashingError(msg) => Status::internal(msg),
        WeakPassword => Status::invalid_argument("Password does not meet strength requirements"),
        VerificationFailed => Status::unauthenticated("Invalid password"),
        InvalidHashFormat => Status::internal("Invalid password hash format"),
        UserNotFound => Status::not_found("User not found"),
        InvalidCredentials => Status::unauthenticated("Invalid credentials"),
        AccountInactive => Status::failed_precondition("Account is inactive or suspended"),
        AccountLocked => Status::failed_precondition("Too many failed login attempts"),
    }
}

pub fn map_db_error_to_status(db_error: DbErr) -> Status {
    error!("Database error occurred: {:?}", db_error);

    match db_error {
        DbErr::ConnectionAcquire(_) => Status::unavailable("Database connection acquire error"),
        DbErr::TryIntoErr { from, into, source } => Status::internal(format!(
            "Type conversion error from {} to {}: {}",
            from, into, source
        )),
        DbErr::Conn(_) => Status::unavailable("Database connection error"),
        DbErr::Exec(ref e) => {
            let msg = e.to_string();
            if msg.contains("Duplicate entry") || msg.contains("1062") {
                Status::already_exists("Resource already exists (e.g. duplicate email or username)")
            } else {
                Status::internal(format!("Database execution error: {}", msg))
            }
        }
        DbErr::Query(_) => Status::internal("Database query error"),
        DbErr::ConvertFromU64(type_str) => {
            Status::internal(format!("Type conversion error from u64: {}", type_str))
        }
        DbErr::UnpackInsertId => Status::internal("Failed to unpack last insert ID"),
        DbErr::UpdateGetPrimaryKey => Status::internal("Failed to get primary key from model"),
        DbErr::RecordNotFound(detail) => Status::not_found(format!("Record not found: {}", detail)),
        DbErr::AttrNotSet(attr) => Status::internal(format!("Attribute {} is not set", attr)),
        DbErr::Custom(err) => Status::internal(format!("Custom database error: {}", err)),
        DbErr::Type(err) => Status::internal(format!("Type error: {}", err)),
        DbErr::Json(err) => Status::internal(format!("JSON error: {}", err)),
        DbErr::Migration(err) => Status::internal(format!("Migration error: {}", err)),
        DbErr::RecordNotInserted => Status::failed_precondition("None of the records are inserted"),
        DbErr::RecordNotUpdated => Status::failed_precondition("None of the records are updated"),
    }
}

/// True if `status` was produced from a MySQL/InnoDB deadlock (error 1213: "Deadlock found when
/// trying to get lock; try restarting transaction"). Deadlocks are expected occasionally when
/// concurrent code paths lock the same tables in different orders — e.g. create_refund /
/// transition_order_status lock `orders` first, while admin_mark_return_received locks
/// `return_requests` first (see audit-report-2026-07-17.md § NEW-02). InnoDB always fully rolls
/// back the losing transaction before returning this error, so retrying from scratch is safe and
/// equivalent to the retried attempt never having raced in the first place.
pub fn is_deadlock_status(status: &Status) -> bool {
    status.code() == tonic::Code::Internal
        && (status.message().contains("Deadlock found") || status.message().contains("1213"))
}
