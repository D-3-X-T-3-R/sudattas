//! Maps Shiprocket `shipment_status_id` (courier lifecycle) to our `Shipments.shipment_status` enum and labels.
//!
//! IDs follow Shiprocket's tracking / webhook vocabulary (see their API docs; IDs may evolve).

use core_db_entities::entity::sea_orm_active_enums::ShipmentStatus;

/// Human-readable label for a Shiprocket `shipment_status_id` (best-effort; unknown IDs get a generic string).
pub fn shiprocket_status_label_for_id(id: i32) -> String {
    match id {
        1 => "New".to_string(),
        2 => "Booked".to_string(),
        3 => "AWB Assigned".to_string(),
        4 => "Pickup Scheduled".to_string(),
        5 => "Manifest Generated".to_string(),
        6 => "Shipped".to_string(),
        7 => "Delivered".to_string(),
        8 => "Cancelled".to_string(),
        9 => "RTO Initiated".to_string(),
        10 => "RTO Delivered".to_string(),
        11 => "Pickup Error".to_string(),
        12 => "Lost".to_string(),
        13 => "Pickup Generated".to_string(),
        14 => "RTO Acknowledged".to_string(),
        15 => "RTO In Transit".to_string(),
        16 => "RTO Undelivered".to_string(),
        17 => "Out For Delivery".to_string(),
        18 => "In Transit".to_string(),
        19 => "Out For Pickup".to_string(),
        20 => "Pickup Exception".to_string(),
        21 => "Undelivered".to_string(),
        22 => "Delayed".to_string(),
        23 => "Partial Delivered".to_string(),
        24 => "Destroyed".to_string(),
        25 => "Damaged".to_string(),
        26 => "Fulfillment".to_string(),
        38 => "Out For Delivery".to_string(),
        41 => "Handover To Courier".to_string(),
        42 => "Picked Up".to_string(),
        45 => "Handover To Courier".to_string(),
        56 => "Out For Delivery".to_string(),
        _ => format!("Shiprocket status ({id})"),
    }
}

/// Short, customer-facing line for order tracking UI (title case phrases).
pub fn customer_tracking_label(status_id: Option<i32>, line: Option<&ShipmentStatus>) -> String {
    if let Some(id) = status_id {
        return match id {
            1 | 2 => "Processing your shipment".to_string(),
            3 => "Courier assigned - preparing pickup".to_string(),
            4 | 19 => "Out for pickup".to_string(),
            5 | 13 | 26 => "Processing at hub".to_string(),
            42 => "Picked up".to_string(),
            6 | 18 | 41 | 45 => "In transit".to_string(),
            15 => "Returning - in transit".to_string(),
            17 | 38 | 56 => "Out for delivery".to_string(),
            7 | 23 => "Delivered".to_string(),
            8 => "Shipment cancelled".to_string(),
            9 | 14 | 16 => "Return in progress".to_string(),
            10 => "Returned".to_string(),
            11 | 20 | 21 => "Delivery issue - we're looking into it".to_string(),
            12 | 24 | 25 => "Shipment problem - contact support".to_string(),
            22 => "Delivery delayed".to_string(),
            _ => shiprocket_status_label_for_id(id),
        };
    }

    match line {
        Some(ShipmentStatus::Pending) | None => "Preparing shipment".to_string(),
        Some(ShipmentStatus::AwbAssigned)
        | Some(ShipmentStatus::LabelGenerated)
        | Some(ShipmentStatus::ManifestGenerated)
        | Some(ShipmentStatus::PickupScheduled) => {
            "Courier assigned - preparing pickup".to_string()
        }
        Some(ShipmentStatus::PickedUp) | Some(ShipmentStatus::InTransit) => {
            "In transit".to_string()
        }
        Some(ShipmentStatus::OutForDelivery) => "Out for delivery".to_string(),
        Some(ShipmentStatus::Delivered) => "Delivered".to_string(),
        Some(ShipmentStatus::RtoInitiated) => "Return in progress".to_string(),
        Some(ShipmentStatus::RtoDelivered) => "Returned".to_string(),
        Some(ShipmentStatus::Cancelled) => "Shipment cancelled".to_string(),
        Some(ShipmentStatus::Lost) | Some(ShipmentStatus::Failed) => {
            "Shipment problem - contact support".to_string()
        }
        Some(ShipmentStatus::Delayed) => "Delivery delayed".to_string(),
    }
}

/// Map Shiprocket `shipment_status_id` to our granular `Shipments.shipment_status`.
pub fn map_shiprocket_id_to_shipment_status(id: i32) -> ShipmentStatus {
    match id {
        1 | 2 => ShipmentStatus::Pending,
        3 => ShipmentStatus::AwbAssigned,
        4 | 19 => ShipmentStatus::PickupScheduled,
        5 | 13 | 26 => ShipmentStatus::ManifestGenerated,
        42 => ShipmentStatus::PickedUp,
        6 | 18 | 41 | 45 => ShipmentStatus::InTransit,
        17 | 38 | 56 => ShipmentStatus::OutForDelivery,
        7 | 23 => ShipmentStatus::Delivered,
        8 => ShipmentStatus::Cancelled,
        9 | 14 | 15 | 16 => ShipmentStatus::RtoInitiated,
        10 => ShipmentStatus::RtoDelivered,
        11 | 12 | 20 | 21 | 24 | 25 => ShipmentStatus::Failed,
        22 => ShipmentStatus::Delayed,
        _ => ShipmentStatus::InTransit,
    }
}
