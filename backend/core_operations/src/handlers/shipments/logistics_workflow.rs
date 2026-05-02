use crate::cancellation_saga;
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::orders::update_order;
use crate::integrations::shiprocket;
use crate::money::paise_to_decimal;
use crate::order_state_machine;
use chrono::{DateTime, Duration, Utc};
use core_db_entities::entity::orders;
use proto::proto::core::{CreateOrderEventRequest, UpdateOrderRequest};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, Statement,
    TransactionTrait,
};
use serde_json::Value;
use tonic::{Request, Status as TonicStatus};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct ShipmentLogisticsRecord {
    pub shipment_id: i64,
    pub order_id: i64,
    pub shiprocket_order_id: Option<String>,
    pub shiprocket_external_order_id: Option<String>,
    pub shipment_status: Option<String>,
    pub awb_code: Option<String>,
    pub carrier: Option<String>,
    pub selected_courier_id: Option<i64>,
    pub selected_courier_name: Option<String>,
    pub quoted_shipping_cost: Option<i64>,
    pub pickup_scheduled_for: Option<DateTime<Utc>>,
    pub logistics_status: Option<String>,
    pub can_customer_cancel: bool,
    pub razorpay_refund_id: Option<String>,
    pub refund_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookingValidation {
    pub order_id: i64,
    pub payment_method: String,
    pub order_status_name: String,
    pub earliest_booking_at: DateTime<Utc>,
    pub pickup_target_at: DateTime<Utc>,
}

fn bool_to_i32(value: bool) -> i32 {
    if value {
        1
    } else {
        0
    }
}

fn shipment_cancel_allowed_for_status(logistics_status: Option<&str>) -> bool {
    matches!(
        logistics_status.unwrap_or("quote_selected"),
        "quote_selected" | "ready_to_ship" | "pickup_scheduled" | "booked"
    )
}

fn logistics_status_from_shiprocket(
    shiprocket_status_id: Option<i32>,
    shipment_status: Option<&str>,
) -> &'static str {
    if matches!(shiprocket_status_id, Some(5 | 13 | 26 | 42)) {
        return "pickup_completed";
    }
    if matches!(shiprocket_status_id, Some(6 | 18 | 41 | 45)) {
        return "in_transit";
    }
    if matches!(shiprocket_status_id, Some(17 | 38 | 56)) {
        return "out_for_delivery";
    }
    if matches!(shiprocket_status_id, Some(7 | 23)) {
        return "delivered";
    }
    if matches!(shiprocket_status_id, Some(9 | 14 | 15 | 16)) {
        return "rto_initiated";
    }
    if matches!(shiprocket_status_id, Some(10)) {
        return "rto_delivered";
    }
    if matches!(shiprocket_status_id, Some(8)) {
        return "cancelled";
    }
    match shipment_status.unwrap_or("pending") {
        "pickup_scheduled" => "pickup_scheduled",
        "picked_up" => "pickup_completed",
        "in_transit" => "in_transit",
        "out_for_delivery" => "out_for_delivery",
        "delivered" => "delivered",
        "rto_initiated" => "rto_initiated",
        "rto_delivered" => "rto_delivered",
        "cancelled" => "cancelled",
        _ => "ready_to_ship",
    }
}

fn fulfillment_status_from_logistics(logistics_status: &str) -> &'static str {
    match logistics_status {
        "pickup_completed" => "pickup_completed",
        "in_transit" | "out_for_delivery" => "in_transit",
        "delivered" => "delivered",
        "rto_initiated" | "rto_delivered" => "rto",
        _ => "booked",
    }
}

fn parse_dt_utc(row: &sea_orm::QueryResult, column: &str) -> Option<DateTime<Utc>> {
    row.try_get("", column).ok()
}

fn parse_bool_like(row: &sea_orm::QueryResult, column: &str) -> bool {
    row.try_get::<i8>("", column)
        .map(|v| v != 0)
        .or_else(|_| row.try_get::<bool>("", column))
        .unwrap_or(false)
}

fn cod_status_allows_booking(status_name: &str) -> bool {
    matches!(status_name, "confirmed" | "partially_cancelled")
}

fn shipment_is_fully_booked(shipment: &ShipmentLogisticsRecord) -> bool {
    shipment.awb_code.is_some() && shipment.shiprocket_order_id.is_some()
}

fn has_non_empty_ref(value: Option<&str>) -> bool {
    value.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn shipment_has_provider_booking_reference(shipment: &ShipmentLogisticsRecord) -> bool {
    has_non_empty_ref(shipment.shiprocket_order_id.as_deref())
        || has_non_empty_ref(shipment.shiprocket_external_order_id.as_deref())
        || has_non_empty_ref(shipment.awb_code.as_deref())
}

fn shipment_booking_claimable(
    logistics_status: Option<&str>,
    order_updated_at: DateTime<Utc>,
    stale_before: DateTime<Utc>,
) -> bool {
    match logistics_status.unwrap_or_default() {
        "booking_pending" | "booking_failed" | "booking_claimed" | "booking_persist_pending" => {
            true
        }
        "booking_in_progress" => order_updated_at < stale_before,
        _ => false,
    }
}

async fn upsert_booking_intent(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<i64, TonicStatus> {
    if let Some(existing) = load_shipment_for_order(txn, order_id, true).await? {
        if shipment_is_fully_booked(&existing) {
            return Err(TonicStatus::failed_precondition(
                "Shipment already created for this order",
            ));
        }
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Shipments
               SET logistics_status = 'booking_pending',
                   shipment_status = 'pending',
                   can_customer_cancel = 1
               WHERE shipment_id = ?"#,
            [existing.shipment_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
        return Ok(existing.shipment_id);
    }

    let insert_result = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"INSERT INTO Shipments (
                   order_id,
                   shiprocket_order_id,
                   shiprocket_external_order_id,
                   awb_code,
                   carrier,
                   selected_courier_id,
                   selected_courier_name,
                   quoted_shipping_cost,
                   quoted_shipping_quote_payload,
                   shiprocket_status_id,
                   shiprocket_status_label,
                   shipment_status,
                   tracking_events,
                   created_at,
                   delivered_at,
                   pickup_scheduled_for,
                   logistics_status,
                   can_customer_cancel,
                   razorpay_refund_id,
                   refund_status,
                   refund_initiated_at
               ) VALUES (?, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'pending', NULL, UTC_TIMESTAMP(), NULL, NULL, 'booking_pending', 1, NULL, NULL, NULL)"#,
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    i64::try_from(insert_result.last_insert_id())
        .map_err(|_| TonicStatus::internal("shipment id overflow"))
}

pub async fn validate_order_can_be_booked(
    txn: &DatabaseTransaction,
    order_id: i64,
    now: DateTime<Utc>,
    for_update: bool,
) -> Result<BookingValidation, TonicStatus> {
    let cancel_window_hours = crate::order_policy::cancel_window_hours();
    let pickup_delay_hours = crate::order_policy::pickup_delay_hours();
    let sql = if for_update {
        r#"SELECT o.OrderID,
                  s.StatusName AS order_status_name,
                  LOWER(COALESCE(o.payment_method, 'prepaid')) AS payment_method,
                  COALESCE(o.payment_status, 'pending') AS payment_status,
                  o.fulfillment_status,
                  COALESCE(o.earliest_booking_at, COALESCE(o.cancel_window_ends_at, DATE_ADD(o.created_at, INTERVAL ? HOUR))) AS earliest_booking_at,
                  COALESCE(o.pickup_target_at, DATE_ADD(o.created_at, INTERVAL ? HOUR)) AS pickup_target_at,
                  EXISTS (
                      SELECT 1
                      FROM OrderDetails od
                      WHERE od.OrderID = o.OrderID
                        AND od.item_status = 'active'
                  ) AS has_active_items
           FROM Orders o
           JOIN OrderStatus s ON s.StatusID = o.StatusID
           WHERE o.OrderID = ?
           LIMIT 1
           FOR UPDATE"#
    } else {
        r#"SELECT o.OrderID,
                  s.StatusName AS order_status_name,
                  LOWER(COALESCE(o.payment_method, 'prepaid')) AS payment_method,
                  COALESCE(o.payment_status, 'pending') AS payment_status,
                  o.fulfillment_status,
                  COALESCE(o.earliest_booking_at, COALESCE(o.cancel_window_ends_at, DATE_ADD(o.created_at, INTERVAL ? HOUR))) AS earliest_booking_at,
                  COALESCE(o.pickup_target_at, DATE_ADD(o.created_at, INTERVAL ? HOUR)) AS pickup_target_at,
                  EXISTS (
                      SELECT 1
                      FROM OrderDetails od
                      WHERE od.OrderID = o.OrderID
                        AND od.item_status = 'active'
                  ) AS has_active_items
           FROM Orders o
           JOIN OrderStatus s ON s.StatusID = o.StatusID
           WHERE o.OrderID = ?
           LIMIT 1"#
    };

    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            sql,
            [
                cancel_window_hours.into(),
                pickup_delay_hours.into(),
                order_id.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Order {} not found", order_id)))?;

    let order_status_name: String = row
        .try_get("", "order_status_name")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let payment_method: String = row
        .try_get("", "payment_method")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let payment_status: String = row
        .try_get("", "payment_status")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let fulfillment_status: String = row
        .try_get("", "fulfillment_status")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let earliest_booking_at: DateTime<Utc> = row
        .try_get("", "earliest_booking_at")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let pickup_target_at: DateTime<Utc> = row
        .try_get("", "pickup_target_at")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let has_active_items = parse_bool_like(&row, "has_active_items");

    if !fulfillment_status.eq_ignore_ascii_case("not_created") {
        return Err(TonicStatus::failed_precondition(
            "Shipment already created for this order",
        ));
    }
    if order_status_name.eq_ignore_ascii_case("cancelled") {
        return Err(TonicStatus::failed_precondition(
            "Cancelled orders cannot be booked",
        ));
    }
    if !has_active_items {
        return Err(TonicStatus::failed_precondition(
            "No active line items available for shipment booking",
        ));
    }
    if !crate::order_policy::is_booking_open(now, earliest_booking_at) {
        return Err(TonicStatus::failed_precondition(
            "Booking window has not opened for this order",
        ));
    }

    if payment_method.eq_ignore_ascii_case("prepaid")
        && !payment_status.eq_ignore_ascii_case("captured")
    {
        return Err(TonicStatus::failed_precondition(
            "Prepaid order is not captured and cannot be booked",
        ));
    }
    if payment_method.eq_ignore_ascii_case("cod") && !cod_status_allows_booking(&order_status_name)
    {
        return Err(TonicStatus::failed_precondition(
            "COD order is not in a bookable state",
        ));
    }

    if let Some(existing) = load_shipment_for_order(txn, order_id, for_update).await? {
        if shipment_is_fully_booked(&existing) {
            return Err(TonicStatus::failed_precondition(
                "Shipment already created for this order",
            ));
        }
    }

    Ok(BookingValidation {
        order_id,
        payment_method,
        order_status_name,
        earliest_booking_at,
        pickup_target_at,
    })
}

async fn recompute_cod_payable_before_booking(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT o.payment_method,
                      o.shipping_charge_minor,
                      o.shipping_minor,
                      sa.PostalCode AS postal_code,
                      COALESCE(SUM(CASE WHEN od.item_status = 'active' THEN od.line_total_minor ELSE 0 END), 0) AS active_items_minor,
                      COALESCE(SUM(CASE WHEN od.item_status = 'active' THEN GREATEST(od.Quantity, 1) ELSE 0 END), 0) AS active_qty
               FROM Orders o
               JOIN ShippingAddresses sa ON sa.ShippingAddressID = o.ShippingAddressID
               LEFT JOIN OrderDetails od ON od.OrderID = o.OrderID
               WHERE o.OrderID = ?
               GROUP BY o.OrderID, o.payment_method, o.shipping_charge_minor, o.shipping_minor, sa.PostalCode"#,
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    let Some(row) = row else {
        return Ok(());
    };
    let payment_method: String = row.try_get("", "payment_method").unwrap_or_default();
    if !payment_method.trim().eq_ignore_ascii_case("cod") {
        return Ok(());
    }

    let active_items_minor: i64 = row
        .try_get("", "active_items_minor")
        .unwrap_or(0_i64)
        .max(0);
    let active_qty: i64 = row.try_get("", "active_qty").unwrap_or(0_i64).max(0);
    let postcode: String = row.try_get("", "postal_code").unwrap_or_default();
    let prior_shipping_minor: i64 = row
        .try_get("", "shipping_charge_minor")
        .or_else(|_| row.try_get("", "shipping_minor"))
        .unwrap_or(0_i64)
        .max(0);

    let threshold = crate::order_policy::free_shipping_threshold_minor();
    let recomputed_shipping_minor = if active_items_minor >= threshold {
        0
    } else {
        match shiprocket::best_courier_quote_for_checkout(
            postcode.trim(),
            active_items_minor,
            active_qty.max(1),
        )
        .await
        {
            Ok(Some(quote)) => quote.shipping_amount_minor.max(0),
            Ok(None) | Err(_) => prior_shipping_minor,
        }
    };
    let recomputed_grand_total_minor = active_items_minor.saturating_add(recomputed_shipping_minor);

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET items_total_minor_after_discount = ?,
               shipping_charge_minor = ?,
               shipping_minor = ?,
               grand_total_minor = ?,
               TotalAmount = ?,
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [
            active_items_minor.into(),
            recomputed_shipping_minor.into(),
            recomputed_shipping_minor.into(),
            recomputed_grand_total_minor.into(),
            paise_to_decimal(recomputed_grand_total_minor).into(),
            order_id.into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    Ok(())
}

pub async fn upsert_quote_selection(
    txn: &DatabaseTransaction,
    order_id: i64,
    selected_courier_id: i64,
    selected_courier_name: &str,
    quoted_shipping_cost: i64,
    quote_payload: &Value,
) -> Result<(), TonicStatus> {
    let existing = load_shipment_for_order(txn, order_id, false).await?;
    if let Some(row) = existing {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Shipments
               SET carrier = ?,
                   selected_courier_id = ?,
                   selected_courier_name = ?,
                   quoted_shipping_cost = ?,
                   quoted_shipping_quote_payload = ?,
                   logistics_status = 'quote_selected',
                   can_customer_cancel = 1
               WHERE shipment_id = ?"#,
            [
                selected_courier_name.into(),
                selected_courier_id.into(),
                selected_courier_name.into(),
                quoted_shipping_cost.into(),
                quote_payload.to_string().into(),
                row.shipment_id.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;
        return Ok(());
    }

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"INSERT INTO Shipments (
               order_id,
               shiprocket_order_id,
               shiprocket_external_order_id,
               awb_code,
               carrier,
               selected_courier_id,
               selected_courier_name,
               quoted_shipping_cost,
               quoted_shipping_quote_payload,
               shiprocket_status_id,
               shiprocket_status_label,
               shipment_status,
               tracking_events,
               created_at,
               delivered_at,
               pickup_scheduled_for,
               logistics_status,
               can_customer_cancel,
               razorpay_refund_id,
               refund_status,
               refund_initiated_at
           ) VALUES (?, NULL, NULL, NULL, ?, ?, ?, ?, ?, NULL, NULL, 'pending', NULL, UTC_TIMESTAMP(), NULL, NULL, 'quote_selected', 1, NULL, NULL, NULL)"#,
        [
            order_id.into(),
            selected_courier_name.into(),
            selected_courier_id.into(),
            selected_courier_name.into(),
            quoted_shipping_cost.into(),
            quote_payload.to_string().into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

pub async fn load_shipment_for_order(
    txn: &DatabaseTransaction,
    order_id: i64,
    for_update: bool,
) -> Result<Option<ShipmentLogisticsRecord>, TonicStatus> {
    let sql = if for_update {
        r#"SELECT shipment_id,
                  order_id,
                  shiprocket_order_id,
                  shiprocket_external_order_id,
                  shipment_status,
                  awb_code,
                  carrier,
                  selected_courier_id,
                  selected_courier_name,
                  quoted_shipping_cost,
                  pickup_scheduled_for,
                  logistics_status,
                  can_customer_cancel,
                  razorpay_refund_id,
                  refund_status
           FROM Shipments
           WHERE order_id = ?
           ORDER BY shipment_id DESC
           LIMIT 1
           FOR UPDATE"#
    } else {
        r#"SELECT shipment_id,
                  order_id,
                  shiprocket_order_id,
                  shiprocket_external_order_id,
                  shipment_status,
                  awb_code,
                  carrier,
                  selected_courier_id,
                  selected_courier_name,
                  quoted_shipping_cost,
                  pickup_scheduled_for,
                  logistics_status,
                  can_customer_cancel,
                  razorpay_refund_id,
                  refund_status
           FROM Shipments
           WHERE order_id = ?
           ORDER BY shipment_id DESC
           LIMIT 1"#
    };
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            sql,
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let Some(row) = row else { return Ok(None) };
    Ok(Some(ShipmentLogisticsRecord {
        shipment_id: row
            .try_get("", "shipment_id")
            .map_err(map_db_error_to_status)?,
        order_id: row
            .try_get("", "order_id")
            .map_err(map_db_error_to_status)?,
        shiprocket_order_id: row.try_get("", "shiprocket_order_id").ok(),
        shiprocket_external_order_id: row.try_get("", "shiprocket_external_order_id").ok(),
        shipment_status: row.try_get("", "shipment_status").ok(),
        awb_code: row.try_get("", "awb_code").ok(),
        carrier: row.try_get("", "carrier").ok(),
        selected_courier_id: row.try_get("", "selected_courier_id").ok(),
        selected_courier_name: row.try_get("", "selected_courier_name").ok(),
        quoted_shipping_cost: row.try_get("", "quoted_shipping_cost").ok(),
        pickup_scheduled_for: parse_dt_utc(&row, "pickup_scheduled_for"),
        logistics_status: row.try_get("", "logistics_status").ok(),
        can_customer_cancel: row
            .try_get::<i8>("", "can_customer_cancel")
            .map(|v| v != 0)
            .or_else(|_| row.try_get::<bool>("", "can_customer_cancel"))
            .unwrap_or(true),
        razorpay_refund_id: row.try_get("", "razorpay_refund_id").ok(),
        refund_status: row.try_get("", "refund_status").ok(),
    }))
}

pub async fn ensure_shiprocket_booking_for_paid_order(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    validate_order_can_be_booked(txn, order_id, Utc::now(), true).await?;
    if let Some(existing) = load_shipment_for_order(txn, order_id, true).await? {
        if shipment_is_fully_booked(&existing) {
            return Ok(());
        }
    }
    upsert_booking_intent(txn, order_id).await?;
    Ok(())
}

pub async fn process_booking_intent(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<bool, TonicStatus> {
    let prep_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let existing_shipment = load_shipment_for_order(&prep_txn, order_id, true).await?;
    if let Some(existing) = existing_shipment.as_ref() {
        if shipment_is_fully_booked(existing) {
            if existing.logistics_status.as_deref() != Some("booked") {
                prep_txn
                    .execute(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        "UPDATE Orders SET fulfillment_status = 'booked', updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
                        [order_id.into()],
                    ))
                    .await
                    .map_err(map_db_error_to_status)?;
                prep_txn
                    .execute(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        "UPDATE Shipments SET logistics_status = 'booked', can_customer_cancel = ? WHERE shipment_id = ?",
                        [
                            bool_to_i32(shipment_cancel_allowed_for_status(Some("booked"))).into(),
                            existing.shipment_id.into(),
                        ],
                    ))
                    .await
                    .map_err(map_db_error_to_status)?;
                prep_txn.commit().await.map_err(map_db_error_to_status)?;
                return Ok(true);
            }
            prep_txn.commit().await.map_err(map_db_error_to_status)?;
            return Ok(false);
        }

        if shipment_has_provider_booking_reference(existing) {
            prep_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET logistics_status = 'booking_persist_pending'
                       WHERE shipment_id = ?
                         AND logistics_status IN (
                             'booking_pending',
                             'booking_failed',
                             'booking_claimed',
                             'booking_in_progress',
                             'booking_persist_pending'
                         )"#,
                    [existing.shipment_id.into()],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            prep_txn.commit().await.map_err(map_db_error_to_status)?;
            warn!(
                order_id,
                shipment_id = existing.shipment_id,
                logistics_status = ?existing.logistics_status,
                "shipment booking skipped because provider references already exist; avoiding duplicate Shiprocket booking"
            );
            return Ok(false);
        }
    }

    let validation = validate_order_can_be_booked(&prep_txn, order_id, Utc::now(), true).await?;
    if validation.payment_method.eq_ignore_ascii_case("cod") {
        recompute_cod_payable_before_booking(&prep_txn, order_id).await?;
    }

    let shipment = if let Some(existing) = existing_shipment {
        existing
    } else {
        let shipment_id = upsert_booking_intent(&prep_txn, order_id).await?;
        load_shipment_for_order(&prep_txn, order_id, true)
            .await?
            .ok_or_else(|| {
                TonicStatus::internal(format!(
                    "Booking intent inserted for order {} but shipment {} missing",
                    order_id, shipment_id
                ))
            })?
    };

    prep_txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Shipments
               SET logistics_status = 'booking_in_progress'
               WHERE shipment_id = ?"#,
            [shipment.shipment_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    prep_txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "UPDATE Orders SET updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let pickup_row = prep_txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COALESCE(pickup_target_at, DATE_ADD(created_at, INTERVAL ? HOUR)) AS pickup_target_at
               FROM Orders
               WHERE OrderID = ?
               LIMIT 1"#,
            [
                crate::order_policy::pickup_delay_hours().into(),
                order_id.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Order {} not found", order_id)))?;
    let pickup_at: DateTime<Utc> = pickup_row
        .try_get("", "pickup_target_at")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;

    let preferred_courier_id = shipment.selected_courier_id;
    prep_txn.commit().await.map_err(map_db_error_to_status)?;

    let booking = match shiprocket::book_shipment_for_order_with_preferred_courier(
        db,
        order_id,
        preferred_courier_id,
    )
    .await
    {
        Ok(booking) => booking,
        Err(error) => {
            warn!(
                order_id,
                error = %error,
                "Shiprocket booking call failed for queued booking intent"
            );
            crate::observability::record_shiprocket_booking_failure_total("provider_error");
            let fail_txn = db.begin().await.map_err(map_db_error_to_status)?;
            fail_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET logistics_status = 'booking_failed'
                       WHERE order_id = ?
                         AND shiprocket_order_id IS NULL
                         AND awb_code IS NULL"#,
                    [order_id.into()],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            let _ = create_order_event(
                &fail_txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "shipment_booking_failed".to_string(),
                    from_status: None,
                    to_status: None,
                    actor_type: "system".to_string(),
                    message: Some(error.to_string()),
                }),
            )
            .await;
            fail_txn.commit().await.map_err(map_db_error_to_status)?;
            return Ok(false);
        }
    };

    let (shipment_status, logistics_status, pickup_for_db) =
        match shiprocket::schedule_pickup_for_shipment(
            booking.shiprocket_shipment_id.as_str(),
            pickup_at,
        )
        .await
        {
            Ok(_) => ("pickup_scheduled", "booked", Some(pickup_at)),
            Err(error) => {
                warn!(
                    order_id,
                    error = %error,
                    pickup_target_at = %pickup_at,
                    "pickup scheduling failed after shipment booking; keeping shipment in booked state"
                );
                ("awb_assigned", "booked", None)
            }
        };
    let can_customer_cancel = shipment_cancel_allowed_for_status(Some(logistics_status));

    let persist_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let persist_result: Result<(), TonicStatus> = async {
        let latest = load_shipment_for_order(&persist_txn, order_id, true).await?;
        let Some(latest_shipment) = latest else {
            return Ok(());
        };
        if shipment_is_fully_booked(&latest_shipment) {
            return Ok(());
        }

        persist_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Shipments
                   SET shiprocket_order_id = ?,
                       shiprocket_external_order_id = ?,
                       awb_code = ?,
                       carrier = ?,
                       shiprocket_status_id = ?,
                       shiprocket_status_label = ?,
                        shipment_status = ?,
                        pickup_scheduled_for = ?,
                        logistics_status = ?,
                        can_customer_cancel = ?
                    WHERE shipment_id = ?"#,
                [
                    booking.shiprocket_shipment_id.clone().into(),
                    booking
                        .shiprocket_order_id
                        .clone()
                        .unwrap_or_default()
                        .into(),
                    booking.awb_code.clone().into(),
                    booking.courier_name.clone().into(),
                    booking.shiprocket_status_id.unwrap_or(3).into(),
                    booking
                        .shiprocket_status_label
                        .clone()
                        .unwrap_or_else(|| "Booked".to_string())
                        .into(),
                    shipment_status.into(),
                    pickup_for_db.into(),
                    logistics_status.into(),
                    bool_to_i32(can_customer_cancel).into(),
                    latest_shipment.shipment_id.into(),
                ],
            ))
            .await
            .map_err(map_db_error_to_status)?;

        persist_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE Orders SET fulfillment_status = 'booked', updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
                [order_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;

        let public_order_ref_for_log = orders::Entity::find_by_id(order_id)
            .one(&persist_txn)
            .await
            .map_err(map_db_error_to_status)?
            .map(|o| o.public_order_ref)
            .unwrap_or_default();
        crate::observability::log_operational_event(
            "shipment_booked",
            &[
                ("order_id", order_id.to_string()),
                ("public_order_ref", public_order_ref_for_log),
                (
                    "shiprocket_shipment_id",
                    booking.shiprocket_shipment_id.clone(),
                ),
                (
                    "shiprocket_order_id",
                    booking.shiprocket_order_id.clone().unwrap_or_default(),
                ),
                (
                    "courier",
                    if booking.courier_name.trim().is_empty() {
                        "unknown".to_string()
                    } else {
                        booking.courier_name.clone()
                    },
                ),
                ("awb_code", booking.awb_code.clone()),
                ("pickup_scheduled_for", pickup_at.to_rfc3339()),
            ],
        );
        Ok(())
    }
    .await;

    match persist_result {
        Ok(()) => {
            persist_txn.commit().await.map_err(map_db_error_to_status)?;
            Ok(true)
        }
        Err(err) => {
            persist_txn.rollback().await.ok();
            let fallback_txn = db.begin().await.map_err(map_db_error_to_status)?;
            fallback_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET shiprocket_order_id = ?,
                           shiprocket_external_order_id = ?,
                           awb_code = ?,
                           carrier = ?,
                           shiprocket_status_id = ?,
                           shiprocket_status_label = ?,
                           shipment_status = ?,
                           pickup_scheduled_for = ?,
                           logistics_status = 'booking_persist_pending',
                            can_customer_cancel = ?
                        WHERE order_id = ?"#,
                    [
                        booking.shiprocket_shipment_id.clone().into(),
                        booking
                            .shiprocket_order_id
                            .clone()
                            .unwrap_or_default()
                            .into(),
                        booking.awb_code.clone().into(),
                        booking.courier_name.clone().into(),
                        booking.shiprocket_status_id.unwrap_or(3).into(),
                        booking
                            .shiprocket_status_label
                            .clone()
                            .unwrap_or_else(|| "Booked".to_string())
                            .into(),
                        shipment_status.into(),
                        pickup_for_db.into(),
                        bool_to_i32(can_customer_cancel).into(),
                        order_id.into(),
                    ],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            fallback_txn
                .commit()
                .await
                .map_err(map_db_error_to_status)?;
            Err(err)
        }
    }
}

pub async fn update_cancelability_from_webhook(
    txn: &DatabaseTransaction,
    order_id: i64,
    shiprocket_status_id: Option<i32>,
    shipment_status: Option<&str>,
) -> Result<(), TonicStatus> {
    let logistics_status = logistics_status_from_shiprocket(shiprocket_status_id, shipment_status);
    let can_cancel = shipment_cancel_allowed_for_status(Some(logistics_status));
    let fulfillment_status = fulfillment_status_from_logistics(logistics_status);
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Shipments
           SET logistics_status = ?,
               can_customer_cancel = ?
           WHERE order_id = ?"#,
        [
            logistics_status.into(),
            bool_to_i32(can_cancel).into(),
            order_id.into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE Orders SET fulfillment_status = ?, updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
        [fulfillment_status.into(), order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

pub async fn cancel_order_via_logistics(
    txn: &DatabaseTransaction,
    order_id: i64,
    _acting_user_id: Option<i64>,
) -> Result<Option<proto::proto::core::OrderResponse>, TonicStatus> {
    let Some(shipment) = load_shipment_for_order(txn, order_id, true).await? else {
        return Ok(None);
    };
    let already_cancelled = shipment.logistics_status.as_deref() == Some("cancelled");
    if already_cancelled {
        return Ok(None);
    }

    if !shipment.can_customer_cancel {
        return Err(TonicStatus::failed_precondition(
            "Order can no longer be cancelled because pickup/logistics is already in progress",
        ));
    }

    let cancel_ref = shipment
        .shiprocket_external_order_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(shipment.shiprocket_order_id.as_deref())
        .ok_or_else(|| {
            TonicStatus::failed_precondition("Order has no Shiprocket identifier to cancel")
        })?;

    move_order_to_cancel_pending_logistics(txn, order_id).await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Shipments
           SET logistics_status = 'cancel_pending_logistics',
               can_customer_cancel = 1
           WHERE shipment_id = ?"#,
        [shipment.shipment_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    crate::observability::log_operational_event(
        "cancellation_initiated",
        &[
            ("order_id", order_id.to_string()),
            ("shiprocket_ref", cancel_ref.to_string()),
        ],
    );

    Err(TonicStatus::unavailable(
        "Shipment cancellation is pending with the logistics partner; retry will continue automatically",
    ))
}

pub async fn book_order_after_validation(
    txn: &DatabaseTransaction,
    order_id: i64,
    now: DateTime<Utc>,
    event_name: &'static str,
) -> Result<i64, TonicStatus> {
    let validation = validate_order_can_be_booked(txn, order_id, now, true).await?;

    if validation.payment_method.eq_ignore_ascii_case("cod") {
        recompute_cod_payable_before_booking(txn, order_id).await?;
    }

    let shipment_id = upsert_booking_intent(txn, order_id).await?;

    crate::observability::log_operational_event(
        event_name,
        &[
            ("order_id", order_id.to_string()),
            ("shipment_id", shipment_id.to_string()),
            ("pickup_target_at", validation.pickup_target_at.to_rfc3339()),
            (
                "booking_opened_at",
                validation.earliest_booking_at.to_rfc3339(),
            ),
            ("order_status_name", validation.order_status_name),
        ],
    );
    info!(
        order_id, shipment_id,
        "validated shipment booking intent persisted; external booking deferred until post-commit worker"
    );

    Ok(shipment_id)
}

pub async fn create_shipments_after_cancel_window_batch(
    txn: &DatabaseTransaction,
    batch_limit: u64,
) -> Result<u64, TonicStatus> {
    let window_hours = crate::order_policy::cancel_window_hours();
    let eligibility_sql = format!(
        r#"SELECT o.OrderID
               FROM Orders o
               JOIN OrderStatus s ON s.StatusID = o.StatusID
                WHERE o.fulfillment_status = 'not_created'
                  AND s.StatusName <> 'cancelled'
                 AND EXISTS (
                     SELECT 1
                     FROM OrderDetails od
                     WHERE od.OrderID = o.OrderID
                       AND od.item_status = 'active'
                  )
                  AND UTC_TIMESTAMP() >= COALESCE(
                      o.earliest_booking_at,
                      COALESCE(o.cancel_window_ends_at, DATE_ADD(o.created_at, INTERVAL {window_hours} HOUR))
                  )
                ORDER BY o.OrderID ASC
                LIMIT ?
                FOR UPDATE SKIP LOCKED"#
    );
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::MySql,
            eligibility_sql,
            [i64::try_from(batch_limit).unwrap_or(i64::MAX).into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut processed = 0_u64;
    for row in rows {
        let order_id: i64 = row.try_get("", "OrderID").map_err(map_db_error_to_status)?;
        match book_order_after_validation(
            txn,
            order_id,
            Utc::now(),
            "shipment_booked_after_cancel_window",
        )
        .await
        {
            Ok(_) => processed += 1,
            Err(error) => {
                warn!(
                    order_id,
                    error = %error,
                    "delayed shipment booking failed; will retry on next worker tick"
                );
            }
        }
    }
    Ok(processed)
}

pub async fn process_booking_intents_batch(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, TonicStatus> {
    let reclaim_timeout_minutes = crate::order_policy::shipment_booking_reclaim_timeout_minutes();
    let stale_before = Utc::now() - Duration::minutes(reclaim_timeout_minutes);
    let claim_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let rows = claim_txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT s.order_id,
                      s.logistics_status,
                      o.updated_at AS order_updated_at
               FROM Shipments s
               JOIN Orders o ON o.OrderID = s.order_id
               WHERE s.logistics_status IN (
                       'booking_pending',
                       'booking_failed',
                       'booking_claimed',
                       'booking_persist_pending'
                    )
                  OR (
                      s.logistics_status = 'booking_in_progress'
                      AND o.updated_at < DATE_SUB(UTC_TIMESTAMP(), INTERVAL ? MINUTE)
                  )
               ORDER BY s.shipment_id ASC
               LIMIT ?
               FOR UPDATE SKIP LOCKED"#,
            [
                reclaim_timeout_minutes.into(),
                i64::try_from(batch_limit).unwrap_or(i64::MAX).into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut order_ids = Vec::with_capacity(rows.len());
    let mut stale_in_progress_reclaimed = 0_u64;
    for row in rows {
        let order_id = row
            .try_get::<i64>("", "order_id")
            .map_err(map_db_error_to_status)?;
        let logistics_status = row
            .try_get::<Option<String>>("", "logistics_status")
            .map_err(map_db_error_to_status)?;
        let order_updated_at = row
            .try_get::<DateTime<Utc>>("", "order_updated_at")
            .map_err(map_db_error_to_status)?;
        let claimable =
            shipment_booking_claimable(logistics_status.as_deref(), order_updated_at, stale_before);
        if !claimable {
            continue;
        }
        if logistics_status.as_deref() == Some("booking_in_progress") {
            stale_in_progress_reclaimed += 1;
        }

        let claim = claim_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Shipments
                   SET logistics_status = 'booking_claimed'
                   WHERE order_id = ?
                     AND (
                         logistics_status IN (
                             'booking_pending',
                             'booking_failed',
                             'booking_claimed',
                             'booking_persist_pending'
                         )
                         OR (
                             logistics_status = 'booking_in_progress'
                             AND EXISTS (
                                 SELECT 1
                                 FROM Orders o
                                 WHERE o.OrderID = Shipments.order_id
                                   AND o.updated_at < DATE_SUB(UTC_TIMESTAMP(), INTERVAL ? MINUTE)
                             )
                         )
                     )"#,
                [order_id.into(), reclaim_timeout_minutes.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if claim.rows_affected() > 0 {
            order_ids.push(order_id);
        }
    }
    if stale_in_progress_reclaimed > 0 {
        warn!(
            stale_in_progress_reclaimed,
            reclaim_timeout_minutes, "shipment worker: reclaiming stale booking_in_progress rows"
        );
    }
    claim_txn.commit().await.map_err(map_db_error_to_status)?;

    let mut processed = 0_u64;
    for order_id in order_ids {
        match process_booking_intent(db, order_id).await {
            Ok(true) => processed += 1,
            Ok(false) => {}
            Err(error) => {
                warn!(
                    order_id,
                    error = %error,
                    "queued shipment booking failed; will retry on next worker tick"
                );
            }
        }
    }
    Ok(processed)
}

async fn claim_cancel_pending_logistics_order_ids(
    txn: &DatabaseTransaction,
    batch_limit: u64,
) -> Result<Vec<i64>, TonicStatus> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT OrderID
               FROM Orders o
               JOIN OrderStatus s ON s.StatusID = o.StatusID
               WHERE s.StatusName = 'cancel_pending_logistics'
               ORDER BY o.OrderID ASC
               LIMIT ?
               FOR UPDATE SKIP LOCKED"#,
            [i64::try_from(batch_limit).unwrap_or(i64::MAX).into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut order_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let order_id: i64 = row.try_get("", "OrderID").map_err(map_db_error_to_status)?;
        let claim = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Shipments
                   SET logistics_status = 'cancel_claimed'
                   WHERE order_id = ?
                     AND logistics_status IN (
                         'cancel_pending_logistics',
                         'cancel_claimed',
                         'cancel_in_progress',
                         'cancel_persist_pending'
                     )"#,
                [order_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if claim.rows_affected() > 0 {
            order_ids.push(order_id);
        }
    }
    Ok(order_ids)
}

pub async fn process_cancel_pending_logistics_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<bool, TonicStatus> {
    let prep_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let Some(shipment) = load_shipment_for_order(&prep_txn, order_id, true).await? else {
        prep_txn.rollback().await.ok();
        return Ok(false);
    };
    if shipment.logistics_status.as_deref() == Some("cancelled") {
        prep_txn.commit().await.map_err(map_db_error_to_status)?;
        return Ok(false);
    }
    let cancel_ref = shipment
        .shiprocket_external_order_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(shipment.shiprocket_order_id.as_deref())
        .ok_or_else(|| {
            TonicStatus::failed_precondition("Order has no Shiprocket identifier to cancel")
        })?
        .to_string();

    let external_cancel_already_succeeded = shipment.logistics_status.as_deref()
        == Some("cancel_persist_pending")
        || shipment
            .shipment_status
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("cancelled"));

    if !external_cancel_already_succeeded {
        prep_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Shipments
                   SET logistics_status = 'cancel_in_progress',
                       can_customer_cancel = 0
                   WHERE shipment_id = ?"#,
                [shipment.shipment_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        prep_txn.commit().await.map_err(map_db_error_to_status)?;

        if let Err(error) = shiprocket::cancel_shiprocket_order(cancel_ref.as_str()).await {
            warn!(order_id, error = %error, "Shiprocket cancellation failed");
            crate::observability::record_shiprocket_cancel_failure_total("provider_error");
            let fail_txn = db.begin().await.map_err(map_db_error_to_status)?;
            move_order_to_cancel_pending_logistics(&fail_txn, order_id).await?;
            fail_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET logistics_status = 'cancel_pending_logistics',
                           can_customer_cancel = 1
                       WHERE order_id = ?"#,
                    [order_id.into()],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            fail_txn.commit().await.map_err(map_db_error_to_status)?;
            return Ok(false);
        }
    } else {
        prep_txn.commit().await.map_err(map_db_error_to_status)?;
    }

    let persist_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let persist_result: Result<(), TonicStatus> = async {
        persist_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Shipments
                   SET logistics_status = 'cancelled',
                       can_customer_cancel = 0,
                       shipment_status = 'cancelled',
                       shiprocket_status_label = COALESCE(shiprocket_status_label, 'Cancelled')
                   WHERE order_id = ?"#,
                [order_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;

        if let Err(err) = ensure_local_order_cancelled(&persist_txn, order_id).await {
            if err.code() != tonic::Code::InvalidArgument {
                return Err(err);
            }
        }
        cancellation_saga::run_order_settlement(&persist_txn, order_id).await?;
        Ok(())
    }
    .await;

    match persist_result {
        Ok(()) => {
            persist_txn.commit().await.map_err(map_db_error_to_status)?;
            Ok(true)
        }
        Err(err) => {
            persist_txn.rollback().await.ok();
            let fallback_txn = db.begin().await.map_err(map_db_error_to_status)?;
            fallback_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET logistics_status = 'cancel_persist_pending',
                           shipment_status = 'cancelled',
                           can_customer_cancel = 0
                       WHERE order_id = ?"#,
                    [order_id.into()],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            move_order_to_cancel_pending_logistics(&fallback_txn, order_id).await?;
            fallback_txn
                .commit()
                .await
                .map_err(map_db_error_to_status)?;
            Err(err)
        }
    }
}

pub async fn process_cancel_pending_logistics_orders(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, TonicStatus> {
    let claim_txn = db.begin().await.map_err(map_db_error_to_status)?;
    let order_ids = claim_cancel_pending_logistics_order_ids(&claim_txn, batch_limit).await?;
    claim_txn.commit().await.map_err(map_db_error_to_status)?;

    let mut processed = 0_u64;
    for order_id in order_ids {
        match process_cancel_pending_logistics_order(db, order_id).await {
            Ok(true) => processed += 1,
            Ok(false) => {}
            Err(err) => warn!(order_id, error = %err, "cancel-pending-logistics retry failed"),
        }
    }
    Ok(processed)
}

pub async fn ensure_local_order_cancelled(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<tonic::Response<proto::proto::core::OrdersResponse>, TonicStatus> {
    let existing_order = orders::Entity::find_by_id(order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Order {} not found", order_id)))?;
    let cancelled_status_id = order_state_machine::get_status_id(txn, "cancelled")
        .await?
        .ok_or_else(|| TonicStatus::internal("OrderStatus 'cancelled' not found"))?;
    update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id: existing_order.user_id,
            shipping_address_id: existing_order.shipping_address_id,
            total_amount_paise: existing_order.grand_total_minor,
            status_id: cancelled_status_id,
        }),
    )
    .await
}

async fn move_order_to_cancel_pending_logistics(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    let status_id = order_state_machine::get_status_id(txn, "cancel_pending_logistics")
        .await?
        .ok_or_else(|| TonicStatus::internal("OrderStatus 'cancel_pending_logistics' not found"))?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET StatusID = ?,
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [status_id.into(), order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id,
            event_type: "cancel_pending_logistics".to_string(),
            from_status: None,
            to_status: Some("cancel_pending_logistics".to_string()),
            actor_type: "system".to_string(),
            message: Some("Waiting for Shiprocket cancellation confirmation".to_string()),
        }),
    )
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        process_booking_intent, process_booking_intents_batch, shipment_booking_claimable,
        shipment_has_provider_booking_reference, ShipmentLogisticsRecord,
    };
    use chrono::{Duration, Utc};
    use core_db_entities::entity::sea_orm_active_enums::ShipmentStatus;
    use core_db_entities::entity::shipments;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn sample_shipment() -> ShipmentLogisticsRecord {
        ShipmentLogisticsRecord {
            shipment_id: 1,
            order_id: 101,
            shiprocket_order_id: None,
            shiprocket_external_order_id: None,
            shipment_status: None,
            awb_code: None,
            carrier: None,
            selected_courier_id: None,
            selected_courier_name: None,
            quoted_shipping_cost: None,
            pickup_scheduled_for: None,
            logistics_status: Some("booking_in_progress".to_string()),
            can_customer_cancel: true,
            razorpay_refund_id: None,
            refund_status: None,
        }
    }

    #[test]
    fn stale_booking_in_progress_is_claimable() {
        let now = Utc::now();
        let stale_before = now - Duration::minutes(12);
        let updated_at = now - Duration::minutes(30);
        assert!(shipment_booking_claimable(
            Some("booking_in_progress"),
            updated_at,
            stale_before
        ));
    }

    #[test]
    fn fresh_booking_in_progress_is_not_claimable() {
        let now = Utc::now();
        let stale_before = now - Duration::minutes(12);
        let updated_at = now - Duration::minutes(2);
        assert!(!shipment_booking_claimable(
            Some("booking_in_progress"),
            updated_at,
            stale_before
        ));
    }

    #[test]
    fn booking_pending_is_always_claimable() {
        let now = Utc::now();
        let stale_before = now - Duration::minutes(12);
        assert!(shipment_booking_claimable(
            Some("booking_pending"),
            now,
            stale_before
        ));
    }

    #[test]
    fn provider_identifiers_block_rebooking() {
        let mut shipment = sample_shipment();
        shipment.shiprocket_order_id = Some("555001".to_string());
        assert!(
            shipment_has_provider_booking_reference(&shipment),
            "existing provider references must block duplicate booking calls"
        );
    }

    #[test]
    fn no_provider_identifiers_allows_booking_attempt() {
        let shipment = sample_shipment();
        assert!(!shipment_has_provider_booking_reference(&shipment));
    }

    #[tokio::test]
    async fn provider_reference_path_skips_duplicate_external_booking_call() {
        let now = Utc::now();
        let shipment_row = shipments::Model {
            shipment_id: 11,
            order_id: 4011,
            shiprocket_order_id: Some("555001".to_string()),
            shiprocket_external_order_id: None,
            awb_code: None,
            carrier: None,
            selected_courier_id: None,
            selected_courier_name: None,
            quoted_shipping_cost: None,
            quoted_shipping_quote_payload: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
            shipment_status: ShipmentStatus::Pending,
            tracking_events: None,
            created_at: Some(now),
            delivered_at: None,
            pickup_scheduled_for: None,
            logistics_status: Some("booking_in_progress".to_string()),
            can_customer_cancel: 1,
            razorpay_refund_id: None,
            refund_status: None,
            refund_initiated_at: None,
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![shipment_row]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        let processed = process_booking_intent(&db, 4011)
            .await
            .expect("provider-reference short-circuit should succeed");
        assert!(
            !processed,
            "row should be kept retryable without invoking provider again"
        );

        let logs = db.into_transaction_log();
        let sql: Vec<String> = logs
            .iter()
            .flat_map(|txn| {
                txn.statements()
                    .iter()
                    .map(|stmt| stmt.sql.to_ascii_lowercase())
            })
            .collect();
        let persist_pending_update = sql.iter().any(|stmt| {
            stmt.contains("update shipments")
                && stmt.contains("set logistics_status = 'booking_persist_pending'")
        });
        assert!(
            persist_pending_update,
            "existing provider ids should move record to booking_persist_pending instead of re-booking"
        );
    }

    #[tokio::test]
    async fn booking_claim_query_includes_stale_in_progress_reclaim_clause() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![Vec::<shipments::Model>::new()])
            .into_connection();
        let processed = process_booking_intents_batch(&db, 25)
            .await
            .expect("batch claim should succeed");
        assert_eq!(processed, 0);

        let logs = db.into_transaction_log();
        let sql: Vec<String> = logs
            .iter()
            .flat_map(|txn| {
                txn.statements()
                    .iter()
                    .map(|stmt| stmt.sql.to_ascii_lowercase())
            })
            .collect();
        let claim_select = sql
            .iter()
            .find(|stmt| stmt.contains("from shipments s"))
            .expect("shipment claim query present");
        assert!(claim_select.contains("booking_in_progress"));
        assert!(claim_select.contains("date_sub(utc_timestamp(), interval ? minute)"));
    }
}
