use crate::handlers::shipments::{
    create_shipments_after_cancel_window_batch, process_booking_intents_batch,
};
use sea_orm::{DatabaseConnection, TransactionTrait};
use tonic::Status;

pub async fn process_create_shipments_after_cancel_window(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    let claim_txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    create_shipments_after_cancel_window_batch(&claim_txn, batch_limit).await?;
    claim_txn
        .commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    process_booking_intents_batch(db, batch_limit).await
}
