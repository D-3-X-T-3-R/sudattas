use crate::handlers::shipments::process_cancel_pending_logistics_orders;
use sea_orm::DatabaseConnection;
use tonic::Status;

pub async fn process_cancel_pending_logistics(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    process_cancel_pending_logistics_orders(db, batch_limit).await
}
