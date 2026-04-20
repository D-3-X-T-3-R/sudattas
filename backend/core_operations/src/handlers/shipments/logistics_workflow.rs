use crate::cancellation_saga;
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::orders::update_order;
use crate::integrations::shiprocket::{self, ShiprocketBooking};
use crate::money::paise_to_decimal;
use crate::order_state_machine;
use chrono::{DateTime, Duration, Utc};
use core_db_entities::entity::orders;
use proto::proto::core::{CreateOrderEventRequest, UpdateOrderRequest};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, Statement};
use serde_json::Value;
use tonic::{Request, Status as TonicStatus};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct ShipmentLogisticsRecord {
    pub shipment_id: i64,
    pub order_id: i64,
    pub shiprocket_order_id: Option<String>,
    pub shiprocket_external_order_id: Option<String>,
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
        "quote_selected" | "ready_to_ship" | "pickup_scheduled"
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

    let active_items_minor: i64 = row.try_get("", "active_items_minor").unwrap_or(0_i64).max(0);
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
    let Some(shipment) = load_shipment_for_order(txn, order_id, true).await? else {
        return Ok(());
    };
    if shipment.awb_code.is_some() && shipment.shiprocket_order_id.is_some() {
        return Ok(());
    }

    let booking = match shiprocket::book_shipment_for_order_with_preferred_courier(
        txn,
        order_id,
        shipment.selected_courier_id,
    )
    .await
    {
        Ok(booking) => booking,
        Err(error) => {
            warn!(order_id, error = %error, "automatic Shiprocket booking failed");
            crate::observability::record_shiprocket_booking_failure_total("provider_error");
            let message = error.to_string();
            txn.execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE Shipments SET logistics_status = 'booking_failed' WHERE shipment_id = ?",
                [shipment.shipment_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "shipment_booking_failed".to_string(),
                    from_status: None,
                    to_status: None,
                    actor_type: "system".to_string(),
                    message: Some(message),
                }),
            )
            .await;
            return Ok(());
        }
    };

    let pickup_at = Utc::now() + Duration::hours(crate::order_policy::pickup_delay_hours());
    if let Err(error) =
        shiprocket::schedule_pickup_for_shipment(booking.shiprocket_shipment_id.as_str(), pickup_at)
            .await
    {
        warn!(order_id, error = %error, "Shiprocket pickup scheduling failed");
        crate::observability::record_shiprocket_booking_failure_total("pickup_schedule_failed");
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Shipments
               SET shiprocket_order_id = ?,
                   shiprocket_external_order_id = ?,
                   awb_code = ?,
                   carrier = ?,
                   shiprocket_status_id = ?,
                   shiprocket_status_label = ?,
                   shipment_status = 'awb_assigned',
                   logistics_status = 'ready_to_ship',
                   can_customer_cancel = 1
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
                    .unwrap_or_else(|| "AWB Assigned".to_string())
                    .into(),
                shipment.shipment_id.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;
        return Ok(());
    }

    let public_order_ref_for_log = orders::Entity::find_by_id(order_id)
        .one(txn)
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
    persist_successful_booking(txn, shipment.shipment_id, &booking, pickup_at).await
}

async fn persist_successful_booking(
    txn: &DatabaseTransaction,
    shipment_id: i64,
    booking: &ShiprocketBooking,
    pickup_at: DateTime<Utc>,
) -> Result<(), TonicStatus> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Shipments
           SET shiprocket_order_id = ?,
               shiprocket_external_order_id = ?,
               awb_code = ?,
               carrier = ?,
               shiprocket_status_id = ?,
               shiprocket_status_label = ?,
               shipment_status = 'pickup_scheduled',
               pickup_scheduled_for = ?,
               logistics_status = 'pickup_scheduled',
               can_customer_cancel = 1
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
            booking.shiprocket_status_id.unwrap_or(4).into(),
            booking
                .shiprocket_status_label
                .clone()
                .unwrap_or_else(|| "Pickup Scheduled".to_string())
                .into(),
            pickup_at.into(),
            shipment_id.into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
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

    if let Err(error) = shiprocket::cancel_shiprocket_order(cancel_ref).await {
        warn!(order_id, error = %error, "Shiprocket cancellation failed");
        crate::observability::record_shiprocket_cancel_failure_total("provider_error");
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
        return Err(TonicStatus::unavailable(
            "Shipment cancellation is pending with the logistics partner; retry will continue automatically",
        ));
    }

    crate::observability::log_operational_event(
        "cancellation_initiated",
        &[
            ("order_id", order_id.to_string()),
            ("shiprocket_ref", cancel_ref.to_string()),
        ],
    );

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Shipments
           SET logistics_status = 'cancelled',
               can_customer_cancel = 0,
               shipment_status = 'cancelled',
               shiprocket_status_label = COALESCE(shiprocket_status_label, 'Cancelled')
           WHERE shipment_id = ?"#,
        [shipment.shipment_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let order = ensure_local_order_cancelled(txn, order_id)
        .await?
        .into_inner()
        .items
        .into_iter()
        .next();

    cancellation_saga::run_order_settlement(txn, order_id).await?;
    Ok(order)
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
                  AND UTC_TIMESTAMP() >= DATE_ADD(o.created_at, INTERVAL {window_hours} HOUR)
                  AND (
                      (LOWER(COALESCE(o.payment_method, 'prepaid')) = 'prepaid' AND o.payment_status = 'captured')
                      OR
                      (
                          LOWER(COALESCE(o.payment_method, '')) = 'cod'
                          AND s.StatusName IN ('confirmed', 'partially_cancelled')
                      )
                  )
                ORDER BY o.OrderID ASC
                LIMIT ?
                FOR UPDATE SKIP LOCKED"#
    );
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::MySql,
            eligibility_sql,
            [
                i64::try_from(batch_limit).unwrap_or(i64::MAX).into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut processed = 0_u64;
    for row in rows {
        let order_id: i64 = row.try_get("", "OrderID").map_err(map_db_error_to_status)?;

        recompute_cod_payable_before_booking(txn, order_id).await?;

        if let Some(existing) = load_shipment_for_order(txn, order_id, true).await? {
            txn.execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE Orders SET fulfillment_status = 'booked', updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
                [order_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
            txn.execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE Shipments SET can_customer_cancel = 0 WHERE shipment_id = ?",
                [existing.shipment_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
            processed += 1;
            continue;
        }

        let booking = match shiprocket::book_shipment_for_order(txn, order_id).await {
            Ok(v) => v,
            Err(error) => {
                warn!(
                    order_id,
                    error = %error,
                    "delayed shipment booking failed; will retry on next worker tick"
                );
                continue;
            }
        };

        let pickup_at = Utc::now() + Duration::hours(crate::order_policy::pickup_delay_hours());
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
                        "delayed pickup scheduling failed; shipment kept as booked without pickup schedule"
                    );
                    ("awb_assigned", "booked", None)
                }
            };

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
               ) VALUES (?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, ?, ?, ?, NULL, UTC_TIMESTAMP(), NULL, ?, ?, 0, NULL, NULL, NULL)"#,
            [
                order_id.into(),
                booking.shiprocket_shipment_id.clone().into(),
                booking.shiprocket_order_id.clone().unwrap_or_default().into(),
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
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;

        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "UPDATE Orders SET fulfillment_status = 'booked', updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

        crate::observability::log_operational_event(
            "shipment_booked_after_cancel_window",
            &[
                ("order_id", order_id.to_string()),
                (
                    "shiprocket_shipment_id",
                    booking.shiprocket_shipment_id.clone(),
                ),
                ("awb_code", booking.awb_code.clone()),
                ("pickup_scheduled_for", pickup_at.to_rfc3339()),
            ],
        );
        processed += 1;
    }
    Ok(processed)
}

pub async fn retry_cancel_pending_logistics_batch(
    txn: &DatabaseTransaction,
    batch_limit: u64,
) -> Result<u64, TonicStatus> {
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

    let mut processed = 0_u64;
    for row in rows {
        let order_id: i64 = row.try_get("", "OrderID").map_err(map_db_error_to_status)?;
        match cancel_order_via_logistics(txn, order_id, None).await {
            Ok(_) => processed += 1,
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
