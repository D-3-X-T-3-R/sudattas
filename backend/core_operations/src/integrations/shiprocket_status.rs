//! Maps Shiprocket `shipment_status_id` (courier lifecycle) to our `Shipments.status` enum and labels.
//!
//! IDs follow Shiprocket’s tracking / webhook vocabulary (see their API docs; IDs may evolve).

use core_db_entities::entity::sea_orm_active_enums::Status;

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

/// Map Shiprocket `shipment_status_id` to our granular `Shipments.status`.
/// Short, customer-facing line for order tracking UI (title case phrases).
pub fn customer_tracking_label(status_id: Option<i32>, line: Option<&Status>) -> String {
    if let Some(id) = status_id {
        return match id {
            1 | 2 => "Processing your shipment".to_string(),
            3 => "Courier assigned — preparing pickup".to_string(),
            4 | 19 => "Out for pickup".to_string(),
            5 | 13 | 26 => "Processing at hub".to_string(),
            42 => "Picked up".to_string(),
            6 | 18 | 41 | 45 => "In transit".to_string(),
            15 => "Returning — in transit".to_string(),
            17 | 38 | 56 => "Out for delivery".to_string(),
            7 | 23 => "Delivered".to_string(),
            8 => "Shipment cancelled".to_string(),
            9 | 14 | 16 => "Return in progress".to_string(),
            10 => "Returned".to_string(),
            11 | 20 | 21 => "Delivery issue — we're looking into it".to_string(),
            12 | 24 | 25 => "Shipment problem — contact support".to_string(),
            22 => "Delivery delayed".to_string(),
            _ => shiprocket_status_label_for_id(id),
        };
    }
    match line {
        Some(Status::Pending) | None => "Preparing shipment".to_string(),
        Some(Status::Processed) => "In transit".to_string(),
        Some(Status::Failed) => "Delivery issue — we're looking into it".to_string(),
        Some(Status::NeedsReview) => "Shipment problem — contact support".to_string(),
        Some(Status::ClientVerified) => "Courier assigned — preparing pickup".to_string(),
    }
}

pub fn map_shiprocket_id_to_shipment_status(id: i32) -> Status {
    match id {
        1 | 2 | 3 | 4 | 5 | 13 | 19 | 26 => Status::Pending,
        6 | 7 | 17 | 18 | 23 | 38 | 41 | 42 | 45 | 56 => Status::Processed,
        8 | 9 | 10 | 11 | 12 | 14 | 15 | 16 | 20 | 21 | 22 | 24 | 25 => Status::Failed,
        _ => Status::Processed,
    }
}
