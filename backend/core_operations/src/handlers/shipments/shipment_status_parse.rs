//! Parse gRPC `status` strings into shipment DB status (includes legacy aliases).

use core_db_entities::entity::sea_orm_active_enums::ShipmentStatus;

pub fn parse_shipment_status_str(raw: &str) -> Option<ShipmentStatus> {
    match raw.trim().to_lowercase().as_str() {
        "pending" => Some(ShipmentStatus::Pending),
        "awb_assigned" | "awb assigned" => Some(ShipmentStatus::AwbAssigned),
        "label_generated" | "label generated" => Some(ShipmentStatus::LabelGenerated),
        "manifest_generated" | "manifest generated" => Some(ShipmentStatus::ManifestGenerated),
        "pickup_scheduled" | "pickup scheduled" | "out for pickup" => {
            Some(ShipmentStatus::PickupScheduled)
        }
        "picked_up" | "picked up" => Some(ShipmentStatus::PickedUp),
        "in_transit" | "in transit" | "shipped" | "processed" => Some(ShipmentStatus::InTransit),
        "out_for_delivery" | "out for delivery" | "ofd" => Some(ShipmentStatus::OutForDelivery),
        "delivered" => Some(ShipmentStatus::Delivered),
        "rto_initiated" | "rto initiated" | "return in progress" | "rto acknowledged"
        | "rto in transit" | "rto undelivered" => Some(ShipmentStatus::RtoInitiated),
        "rto_delivered" | "rto delivered" | "returned" => Some(ShipmentStatus::RtoDelivered),
        "cancelled" | "canceled" => Some(ShipmentStatus::Cancelled),
        "lost" => Some(ShipmentStatus::Lost),
        "delayed" => Some(ShipmentStatus::Delayed),
        "failed" | "pickup error" | "pickup exception" | "undelivered" | "destroyed"
        | "damaged" | "needs_review" | "needs review" | "client_verified" | "client verified" => {
            Some(ShipmentStatus::Failed)
        }
        _ => None,
    }
}
