//! Parse gRPC `status` strings into generic DB `Status` (includes legacy aliases).

use core_db_entities::entity::sea_orm_active_enums::Status;

pub fn parse_shipment_status_str(raw: &str) -> Option<Status> {
    match raw.trim().to_lowercase().as_str() {
        "pending" => Some(Status::Pending),
        "client_verified" | "client verified" => Some(Status::ClientVerified),
        "needs_review" | "needs review" => Some(Status::NeedsReview),
        "failed" | "cancelled" | "canceled" | "lost" | "rto_initiated" | "rto delivered"
        | "rto_delivered" => Some(Status::Failed),
        "processed" | "delivered" | "in_transit" | "in transit" | "shipped"
        | "out_for_delivery" | "out for delivery" | "ofd" | "picked_up" | "picked up" => {
            Some(Status::Processed)
        }
        "awb_assigned" | "awb assigned" | "label_generated" | "label generated"
        | "manifest_generated" | "manifest generated" | "pickup_scheduled" | "pickup scheduled" => {
            Some(Status::Pending)
        }
        _ => None,
    }
}
