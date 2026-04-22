use crate::handlers::shipments::retry_cancel_pending_logistics_batch;
use sea_orm::{DatabaseConnection, TransactionTrait};
use tonic::Status;

pub async fn process_cancel_pending_logistics(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let processed = retry_cancel_pending_logistics_batch(&txn, batch_limit).await?;
    txn.commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(processed)
}
