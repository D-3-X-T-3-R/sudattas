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

#[cfg(test)]
mod tests {
    use super::{
        extract_scan_from_webhook_item, flatten_shiprocket_webhook_items,
        parse_local_order_id_from_ref,
    };
    use serde_json::json;

    #[test]
    fn flatten_handles_top_level_array() {
        let payload = json!([{ "awb": "AWB1" }, { "awb": "AWB2" }]);
        let items = flatten_shiprocket_webhook_items(&payload);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["awb"], "AWB1");
    }

    #[test]
    fn flatten_handles_shipments_array_wrapper() {
        let payload = json!({ "shipments": [{ "awb": "AWB1" }] });
        let items = flatten_shiprocket_webhook_items(&payload);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["awb"], "AWB1");
    }

    #[test]
    fn flatten_handles_singular_shipment_wrapper() {
        let payload = json!({ "shipment": { "awb": "AWB1" } });
        let items = flatten_shiprocket_webhook_items(&payload);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["awb"], "AWB1");
    }

    #[test]
    fn flatten_falls_back_to_bare_object() {
        let payload = json!({ "awb": "AWB1" });
        let items = flatten_shiprocket_webhook_items(&payload);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["awb"], "AWB1");
    }

    #[test]
    fn extract_scan_prefers_shipment_status_id_over_current_status_id() {
        let item = json!({ "shipment_status_id": 7, "current_status_id": 42 });
        let (status_id, _, _) = extract_scan_from_webhook_item(&item);
        assert_eq!(status_id, Some(7));
    }

    #[test]
    fn extract_scan_falls_back_to_current_status_id() {
        let item = json!({ "current_status_id": 42 });
        let (status_id, _, _) = extract_scan_from_webhook_item(&item);
        assert_eq!(status_id, Some(42));
    }

    #[test]
    fn extract_scan_reads_label_and_track_array() {
        let item = json!({
            "shipment_status": "Delivered",
            "shipment_track": [{ "date": "2026-01-01", "status": "Delivered" }],
        });
        let (_, label, scan) = extract_scan_from_webhook_item(&item);
        assert_eq!(label, Some("Delivered".to_string()));
        assert!(scan.unwrap().is_array());
    }

    #[test]
    fn extract_scan_ignores_non_array_scans_field() {
        let item = json!({ "scans": "not-an-array" });
        let (_, _, scan) = extract_scan_from_webhook_item(&item);
        assert_eq!(scan, None);
    }

    #[test]
    fn extract_scan_returns_none_when_no_status_fields_present() {
        let item = json!({ "unrelated": true });
        let (status_id, label, scan) = extract_scan_from_webhook_item(&item);
        assert_eq!(status_id, None);
        assert_eq!(label, None);
        assert_eq!(scan, None);
    }

    #[test]
    fn parse_local_order_id_accepts_plain_integer() {
        assert_eq!(parse_local_order_id_from_ref("123"), Some(123));
    }

    #[test]
    fn parse_local_order_id_accepts_sud_prefix_case_insensitively() {
        assert_eq!(parse_local_order_id_from_ref("sud-456"), Some(456));
        assert_eq!(parse_local_order_id_from_ref("SUD-456"), Some(456));
    }

    #[test]
    fn parse_local_order_id_rejects_empty_or_garbage() {
        assert_eq!(parse_local_order_id_from_ref(""), None);
        assert_eq!(parse_local_order_id_from_ref("   "), None);
        assert_eq!(parse_local_order_id_from_ref("not-an-order-ref"), None);
    }
}
