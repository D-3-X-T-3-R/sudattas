use crate::handlers::shipments::{
    create_shipments_after_cancel_window_batch, process_booking_intents_batch,
};
use sea_orm::DatabaseConnection;
use tonic::Status;

pub async fn process_create_shipments_after_cancel_window(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    // create_shipments_after_cancel_window_batch manages its own short-lived transactions
    // internally (a claim transaction, then a prep/write transaction pair per order, with its
    // Shiprocket calls made with no transaction open) — it must NOT be wrapped in an outer
    // transaction here, or that would hold a connection open across this whole batch again,
    // exactly the bug this restructuring fixes.
    create_shipments_after_cancel_window_batch(db, batch_limit).await?;

    process_booking_intents_batch(db, batch_limit).await
}
