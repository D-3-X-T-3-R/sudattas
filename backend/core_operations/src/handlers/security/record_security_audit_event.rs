//! P2 Security: record a security audit event (e.g. after secrets rotation).

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::security_audit_log;
use proto::proto::core::{RecordSecurityAuditRequest, RecordSecurityAuditResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn record_security_audit_event(
    txn: &DatabaseTransaction,
    request: Request<RecordSecurityAuditRequest>,
) -> Result<Response<RecordSecurityAuditResponse>, Status> {
    let req = request.into_inner();
    let event_type = req.event_type.trim();
    if event_type.is_empty() {
        return Err(Status::invalid_argument("event_type must be non-empty"));
    }

    let row = security_audit_log::ActiveModel {
        id: ActiveValue::NotSet,
        event_type: ActiveValue::Set(event_type.to_string()),
        details: ActiveValue::Set(req.details),
        created_at: ActiveValue::NotSet,
    };
    row.insert(txn).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(RecordSecurityAuditResponse { success: true }))
}
