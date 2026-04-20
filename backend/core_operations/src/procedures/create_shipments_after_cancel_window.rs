use crate::handlers::shipments::create_shipments_after_cancel_window_batch;
use sea_orm::{DatabaseConnection, TransactionTrait};
use tonic::Status;

pub async fn process_create_shipments_after_cancel_window(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let processed = create_shipments_after_cancel_window_batch(&txn, batch_limit).await?;
    txn.commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(processed)
}
