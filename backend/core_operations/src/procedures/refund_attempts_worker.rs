use crate::cancellation_saga::process_pending_refund_attempts;
use sea_orm::DatabaseConnection;
use tonic::Status;

pub async fn process_refund_attempts(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    process_pending_refund_attempts(db, batch_limit).await
}
