use proto::tonic::{self, Code as StatusCode, Status};
use std::num::ParseIntError;
use strum::Display;

#[derive(Debug)]
pub struct GqlError {
    pub code: Code,
    pub message: String,
}

#[derive(Display, Debug)]
pub enum Code {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

impl GqlError {
    pub fn new(message: &str, code: Code) -> GqlError {
        GqlError {
            code,
            message: message.to_string(),
        }
    }
}

pub fn map_err(status: Status) -> GqlError {
    let code: Code = match status.code() {
        StatusCode::Ok => Code::Ok,
        StatusCode::Cancelled => Code::Cancelled,
        StatusCode::Unknown => Code::Unknown,
        StatusCode::InvalidArgument => Code::InvalidArgument,
        StatusCode::DeadlineExceeded => Code::DeadlineExceeded,
        StatusCode::NotFound => Code::NotFound,
        StatusCode::AlreadyExists => Code::AlreadyExists,
        StatusCode::PermissionDenied => Code::PermissionDenied,
        StatusCode::ResourceExhausted => Code::ResourceExhausted,
        StatusCode::FailedPrecondition => Code::FailedPrecondition,
        StatusCode::Aborted => Code::Aborted,
        StatusCode::OutOfRange => Code::OutOfRange,
        StatusCode::Unimplemented => Code::Unimplemented,
        StatusCode::Internal => Code::Internal,
        StatusCode::Unavailable => Code::Unavailable,
        StatusCode::DataLoss => Code::DataLoss,
        StatusCode::Unauthenticated => Code::Unauthenticated,
        #[allow(unreachable_patterns)]
        _ => Code::Unknown,
    };
    GqlError {
        code,
        message: status.message().to_string(),
    }
}

impl std::fmt::Display for GqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.code, self.message)
    }
}

impl From<tonic::Status> for GqlError {
    fn from(value: tonic::Status) -> Self {
        map_err(value)
    }
}

impl From<tonic::transport::Error> for GqlError {
    fn from(value: tonic::transport::Error) -> Self {
        GqlError::new(&value.to_string(), Code::Unavailable)
    }
}

impl From<ParseIntError> for GqlError {
    fn from(value: ParseIntError) -> Self {
        GqlError::new(&value.to_string(), Code::InvalidArgument)
    }
}
