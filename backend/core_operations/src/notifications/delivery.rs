//! P1 Notification delivery: stub that can be replaced with real email/SMS.
//! Returns Ok(()) on success, Err on failure; outbox worker leaves event Pending on Err for retry.

use core_db_entities::entity::outbox_events;
use tonic::Status;
use tracing::info;

/// Deliver one outbox event (email/SMS). Stub: logs and returns Ok.
/// Replace with real delivery; return Err to leave event Pending for retry.
/// In tests, set env `OUTBOX_DELIVER_FAIL=1` to simulate delivery failure (for retry-path tests).
pub async fn deliver_event(event: &outbox_events::Model) -> Result<(), Status> {
    if std::env::var("OUTBOX_DELIVER_FAIL").as_deref() == Ok("1") {
        return Err(Status::internal("simulated delivery failure for test"));
    }
    info!(
        event_type = event.event_type,
        aggregate_id = %event.aggregate_id,
        "outbox: deliver (stub — replace with real email/SMS)"
    );
    Ok(())
}
