//! Apply a Shiprocket tracking scan (webhook or track API) to a `Shipments` row.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::integrations::shiprocket_status::{
    map_shiprocket_id_to_shipment_status, shiprocket_status_label_for_id,
};
use chrono::Utc;
use core_db_entities::entity::{orders, shipments};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder,
};
use serde_json::Value as JsonVal;
use tonic::Status as TonicStatus;

pub async fn apply_shiprocket_scan_to_shipment(
    txn: &DatabaseTransaction,
    shipment: shipments::Model,
    status_id: Option<i32>,
    status_label: Option<String>,
    scan_events: Option<JsonVal>,
) -> Result<shipments::Model, TonicStatus> {
    let Some(id) = status_id else {
        if let Some(ev) = scan_events {
            if ev.is_array() {
                let mut active = shipment.into_active_model();
                active.tracking_events = ActiveValue::Set(Some(ev));
                return active.update(txn).await.map_err(map_db_error_to_status);
            }
        }
        return Ok(shipment);
    };

    let label = status_label.unwrap_or_else(|| shiprocket_status_label_for_id(id));
    let line = map_shiprocket_id_to_shipment_status(id);
    let mut active = shipment.into_active_model();
    active.shiprocket_status_id = ActiveValue::Set(Some(id));
    active.shiprocket_status_label = ActiveValue::Set(Some(label));
    active.shipment_status = ActiveValue::Set(line.clone());
    if matches!(id, 7 | 23) {
        active.delivered_at = ActiveValue::Set(Some(Utc::now()));
    }
    if let Some(ev) = scan_events {
        if ev.is_array() {
            active.tracking_events = ActiveValue::Set(Some(ev));
        }
    }
    active.update(txn).await.map_err(map_db_error_to_status)
}

/// Resolve webhook JSON: single object, or `shipments` / `shipment` / top-level array.
pub fn flatten_shiprocket_webhook_items(payload: &JsonVal) -> Vec<JsonVal> {
    if let Some(arr) = payload.as_array() {
        return arr.clone();
    }
    if let Some(arr) = payload.get("shipments").and_then(|x| x.as_array()) {
        return arr.clone();
    }
    if let Some(one) = payload.get("shipment") {
        return vec![one.clone()];
    }
    vec![payload.clone()]
}

pub async fn find_shipment_for_shiprocket_event(
    txn: &DatabaseTransaction,
    item: &JsonVal,
) -> Result<Option<shipments::Model>, TonicStatus> {
    let awb = item
        .get("awb")
        .and_then(|x| x.as_str())
        .or_else(|| item.get("awb_code").and_then(|x| x.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let sr_shipment_id = item
        .get("shipment_id")
        .and_then(|x| x.as_i64())
        .map(|n| n.to_string())
        .or_else(|| {
            item.get("shipment_id")
                .and_then(|x| x.as_str())
                .map(String::from)
        })
        .or_else(|| {
            item.get("sr_order_id")
                .and_then(|x| x.as_str())
                .map(String::from)
        });
    let order_ref = item
        .get("order_id")
        .and_then(|x| x.as_str())
        .or_else(|| item.get("channel_order_id").and_then(|x| x.as_str()))
        .or_else(|| item.get("reference_number").and_then(|x| x.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    if let Some(a) = awb {
        if let Some(m) = shipments::Entity::find()
            .filter(shipments::Column::AwbCode.eq(a))
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
        {
            return Ok(Some(m));
        }
    }
    if let Some(ref sid) = sr_shipment_id {
        if sid.is_empty() {
            return Ok(None);
        }
        if let Some(m) = shipments::Entity::find()
            .filter(shipments::Column::ShiprocketOrderId.eq(sid.as_str()))
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
        {
            return Ok(Some(m));
        }
    }
    if let Some(ref_id) = order_ref {
        let mut resolved_order_id = parse_local_order_id_from_ref(&ref_id);
        if resolved_order_id.is_none() {
            let pref = ref_id.trim().to_ascii_uppercase();
            if !pref.is_empty() {
                if let Some(row) = orders::Entity::find()
                    .filter(orders::Column::PublicOrderRef.eq(pref))
                    .one(txn)
                    .await
                    .map_err(map_db_error_to_status)?
                {
                    resolved_order_id = Some(row.order_id);
                }
            }
        }
        if let Some(order_id) = resolved_order_id {
            if let Some(m) = shipments::Entity::find()
                .filter(shipments::Column::OrderId.eq(order_id))
                .order_by_desc(shipments::Column::ShipmentId)
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
            {
                return Ok(Some(m));
            }
        }
    }
    Ok(None)
}

fn parse_local_order_id_from_ref(raw: &str) -> Option<i64> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(n) = t.parse::<i64>() {
        return Some(n);
    }
    let upper = t.to_uppercase();
    if let Some(rest) = upper.strip_prefix("SUD-") {
        if let Ok(n) = rest.parse::<i64>() {
            return Some(n);
        }
    }
    None
}

pub fn extract_scan_from_webhook_item(
    item: &JsonVal,
) -> (Option<i32>, Option<String>, Option<JsonVal>) {
    let status_id = item
        .get("shipment_status_id")
        .and_then(|x| x.as_i64())
        .or_else(|| item.get("current_status_id").and_then(|x| x.as_i64()))
        .map(|x| x as i32);
    let label = item
        .get("shipment_status")
        .or_else(|| item.get("current_status"))
        .and_then(|x| x.as_str())
        .map(std::string::ToString::to_string);
    let scan = item
        .get("shipment_track")
        .or_else(|| item.get("scans"))
        .filter(|x| x.is_array())
        .cloned();
    (status_id, label, scan)
}
