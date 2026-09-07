//! Category-scoped exchanges: same product, different size/colour, exact same price — a swap,
//! not a refund. Distinct pipeline from `handlers::returns` (which is refund-only).
//!
//! Fulfillment model: once an admin marks an exchange "received" (the original item physically
//! back in hand), the desired variant's stock is decremented, the original variant's stock is
//! restored, and a brand-new $0 replacement order is created via `place_order_admin` — so the
//! replacement gets its own full order/shipment/tracking lifecycle, exactly like a real order,
//! rather than silently mutating the original order in place.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::returns::load_delivered_at;
use crate::order_policy;
use crate::procedures::orders::place_order_admin;
use chrono::Utc;
use core_db_entities::entity::{
    exchange_requests, order_details, orders, product_categories, product_variants, products,
};
use proto::proto::core::{
    AdminMarkExchangeReceivedRequest, AdminUpdateExchangeStatusRequest, CreateOrderEventRequest,
    ExchangeRequestResponse, ExchangeRequestsResponse, PlaceOrderAdminLineItem,
    PlaceOrderAdminRequest, RequestExchangeRequest, SearchExchangeRequestsRequest,
};
use sea_orm::{
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, QueryTrait, Statement, TransactionTrait,
};
use tonic::{Request, Response, Status};

const STATUS_REQUESTED: &str = "requested";
const STATUS_APPROVED: &str = "approved";
const STATUS_IN_TRANSIT: &str = "in_transit";
const STATUS_RECEIVED: &str = "received";
const STATUS_COMPLETED: &str = "completed";
const STATUS_REJECTED: &str = "rejected";
const STATUS_CANCELLED: &str = "cancelled";

fn normalize_status(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn status_allows_admin_reject_or_cancel(current_status: &str) -> bool {
    matches!(
        current_status,
        STATUS_REQUESTED | STATUS_APPROVED | STATUS_IN_TRANSIT
    )
}

fn status_allows_mark_received(current_status: &str) -> bool {
    matches!(
        current_status,
        STATUS_REQUESTED | STATUS_APPROVED | STATUS_IN_TRANSIT
    )
}

fn status_allows_admin_update_target(target_status: &str) -> bool {
    matches!(
        target_status,
        STATUS_REQUESTED | STATUS_APPROVED | STATUS_IN_TRANSIT | STATUS_REJECTED | STATUS_CANCELLED
    )
}

fn exchange_response(row: &exchange_requests::Model) -> ExchangeRequestResponse {
    ExchangeRequestResponse {
        exchange_id: row.exchange_id,
        order_id: row.order_id,
        user_id: row.user_id,
        order_detail_id: row.order_detail_id,
        desired_variant_id: row.desired_variant_id,
        quantity: row.quantity,
        status: row.status.clone(),
        reason: row.reason.clone(),
        created_at: row.created_at.to_rfc3339(),
        received_at: row.received_at.map(|v| v.to_rfc3339()),
        replacement_order_id: row.replacement_order_id,
    }
}

/// Effective selling price of a variant: the product's base price plus that variant's own
/// delta (e.g. a larger size costing more) — the same arithmetic checkout itself uses.
fn variant_price_minor(product: &products::Model, variant: &product_variants::Model) -> i64 {
    i64::from(product.price_paise) + i64::from(variant.additional_price.unwrap_or(0))
}

pub async fn request_exchange(
    txn: &DatabaseTransaction,
    request: Request<RequestExchangeRequest>,
) -> Result<Response<ExchangeRequestsResponse>, Status> {
    let req = request.into_inner();
    let reason = req.reason.trim();
    if reason.is_empty() {
        return Err(Status::invalid_argument("Exchange reason is required"));
    }

    let order = orders::Entity::find_by_id(req.order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;
    if order.user_id != req.user_id {
        return Err(Status::not_found("Order not found"));
    }

    let detail = order_details::Entity::find_by_id(req.order_detail_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order item not found"))?;
    if detail.order_id != req.order_id {
        return Err(Status::not_found("Order item not found"));
    }
    if detail.item_status.eq_ignore_ascii_case("cancelled")
        || detail.item_status.eq_ignore_ascii_case("exchanged")
    {
        return Err(Status::failed_precondition(
            "This item has already been cancelled or exchanged",
        ));
    }

    let delivered_at = load_delivered_at(txn, &order).await?;
    let delivered_at = delivered_at.ok_or_else(|| {
        Status::failed_precondition("Exchanges can be requested only after delivery")
    })?;
    if !order_policy::is_within_return_window(delivered_at, Utc::now()) {
        return Err(Status::failed_precondition("Exchange window has closed"));
    }

    let variant = product_variants::Entity::find_by_id(detail.variant_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Order item's variant no longer exists"))?;
    let product = products::Entity::find_by_id(variant.product_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Order item's product no longer exists"))?;
    let category = product_categories::Entity::find_by_id(product.category_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Product's category no longer exists"))?;
    if category.exchange_eligible == 0 {
        return Err(Status::failed_precondition(
            "This product's category is not eligible for exchanges",
        ));
    }

    let desired_variant = product_variants::Entity::find_by_id(req.desired_variant_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Desired variant not found"))?;
    if desired_variant.product_id != product.product_id {
        return Err(Status::invalid_argument(
            "Exchanges are only allowed for a different size/colour of the same product",
        ));
    }
    if desired_variant.variant_id == variant.variant_id {
        return Err(Status::invalid_argument(
            "Desired variant must be different from the one originally purchased",
        ));
    }
    if variant_price_minor(&product, &variant) != variant_price_minor(&product, &desired_variant) {
        return Err(Status::failed_precondition(
            "Exchanges are only available when the replacement is the exact same price — request a return instead",
        ));
    }

    let quantity = req.quantity.unwrap_or(detail.quantity);
    if quantity <= 0 || quantity > detail.quantity {
        return Err(Status::invalid_argument(
            "Exchange quantity must be positive and not exceed the purchased quantity",
        ));
    }

    let open_exchange_count = exchange_requests::Entity::find()
        .filter(exchange_requests::Column::OrderDetailId.eq(req.order_detail_id))
        .filter(exchange_requests::Column::Status.is_not_in([STATUS_REJECTED, STATUS_CANCELLED]))
        .count(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if open_exchange_count > 0 {
        return Err(Status::failed_precondition(
            "This item already has an active exchange request",
        ));
    }

    let open_return_count = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS cnt
               FROM ReturnRequestItems rri
               JOIN ReturnRequests rr ON rr.return_id = rri.return_id
               WHERE rri.order_detail_id = ?
                 AND rr.status NOT IN ('rejected', 'cancelled')"#,
            [req.order_detail_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .and_then(|row| row.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);
    if open_return_count > 0 {
        return Err(Status::failed_precondition(
            "This item already has an active return request",
        ));
    }

    let inserted = exchange_requests::ActiveModel {
        exchange_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(req.order_id),
        user_id: ActiveValue::Set(req.user_id),
        order_detail_id: ActiveValue::Set(req.order_detail_id),
        desired_variant_id: ActiveValue::Set(req.desired_variant_id),
        quantity: ActiveValue::Set(quantity),
        status: ActiveValue::Set(STATUS_REQUESTED.to_string()),
        reason: ActiveValue::Set(reason.to_string()),
        replacement_order_id: ActiveValue::Set(None),
        created_at: ActiveValue::NotSet,
        received_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .map_err(map_db_error_to_status)?;

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: req.order_id,
            event_type: "exchange_requested".to_string(),
            from_status: None,
            to_status: Some(STATUS_REQUESTED.to_string()),
            actor_type: "customer".to_string(),
            message: Some(format!(
                "Exchange requested for order item {}: {}",
                req.order_detail_id, reason
            )),
        }),
    )
    .await;

    Ok(Response::new(ExchangeRequestsResponse {
        items: vec![exchange_response(&inserted)],
    }))
}

pub async fn search_exchange_requests(
    txn: &DatabaseTransaction,
    request: Request<SearchExchangeRequestsRequest>,
) -> Result<Response<ExchangeRequestsResponse>, Status> {
    let req = request.into_inner();
    let rows = exchange_requests::Entity::find()
        .apply_if(req.exchange_id, |query, v| {
            query.filter(exchange_requests::Column::ExchangeId.eq(v))
        })
        .apply_if(req.order_id, |query, v| {
            query.filter(exchange_requests::Column::OrderId.eq(v))
        })
        .apply_if(req.user_id, |query, v| {
            query.filter(exchange_requests::Column::UserId.eq(v))
        })
        .order_by_desc(exchange_requests::Column::ExchangeId)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(ExchangeRequestsResponse {
        items: rows.iter().map(exchange_response).collect(),
    }))
}

pub async fn admin_update_exchange_status(
    txn: &DatabaseTransaction,
    request: Request<AdminUpdateExchangeStatusRequest>,
) -> Result<Response<ExchangeRequestsResponse>, Status> {
    let req = request.into_inner();
    let target_status = normalize_status(&req.status);
    if !status_allows_admin_update_target(&target_status) {
        return Err(Status::invalid_argument(
            "Unsupported exchange status transition target",
        ));
    }

    let row = exchange_requests::Entity::find_by_id(req.exchange_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Exchange request not found"))?;
    let current_status = normalize_status(&row.status);

    if current_status == target_status {
        return Ok(Response::new(ExchangeRequestsResponse {
            items: vec![exchange_response(&row)],
        }));
    }

    if matches!(target_status.as_str(), STATUS_REJECTED | STATUS_CANCELLED)
        && !status_allows_admin_reject_or_cancel(&current_status)
    {
        return Err(Status::failed_precondition(
            "Exchange cannot be rejected or cancelled in its current state",
        ));
    }

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ExchangeRequests SET status = ? WHERE exchange_id = ?",
        [target_status.clone().into(), row.exchange_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let note = req.note.unwrap_or_default();
    let note = note.trim();
    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: row.order_id,
            event_type: format!("exchange_{}", target_status),
            from_status: Some(current_status),
            to_status: Some(target_status.clone()),
            actor_type: "admin".to_string(),
            message: if note.is_empty() {
                Some(format!(
                    "Exchange {} updated to {}",
                    row.exchange_id, target_status
                ))
            } else {
                Some(format!(
                    "Exchange {} updated to {}: {}",
                    row.exchange_id, target_status, note
                ))
            },
        }),
    )
    .await;

    let updated = exchange_requests::Entity::find_by_id(row.exchange_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Exchange request not found"))?;
    Ok(Response::new(ExchangeRequestsResponse {
        items: vec![exchange_response(&updated)],
    }))
}

/// Admin marks the original item received back at the store. This is the one step that actually
/// does something physical: restores the original variant's stock, decrements the desired
/// variant's stock, and creates a brand-new $0 replacement order (its own shipment/tracking
/// lifecycle, via the same `place_order_admin` path manual admin sales use).
///
/// Deliberately idempotent/retryable: if the replacement order fails to create (e.g. the desired
/// variant sold out in the meantime), the exchange is left in `received` with the original item's
/// stock already restored but no `replacement_order_id` — calling this again retries just the
/// replacement-order step rather than re-restoring stock or erroring on "wrong state".
pub async fn admin_mark_exchange_received(
    db: &DatabaseConnection,
    request: Request<AdminMarkExchangeReceivedRequest>,
) -> Result<Response<ExchangeRequestsResponse>, Status> {
    let req = request.into_inner();

    let prep_txn = db.begin().await.map_err(|e| {
        Status::internal(format!(
            "failed to begin admin_mark_exchange_received txn: {e}"
        ))
    })?;

    let row = exchange_requests::Entity::find_by_id(req.exchange_id)
        .lock(LockType::Update)
        .one(&prep_txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Exchange request not found"))?;
    let current_status = normalize_status(&row.status);

    if row.replacement_order_id.is_some() {
        // Already fully completed — return as-is, same "already done, no-op" shape as the
        // returns equivalent.
        prep_txn.commit().await.map_err(map_db_error_to_status)?;
        return Ok(Response::new(ExchangeRequestsResponse {
            items: vec![exchange_response(&row)],
        }));
    }

    let already_received = current_status == STATUS_RECEIVED;
    if !status_allows_mark_received(&current_status) && !already_received {
        return Err(Status::failed_precondition(
            "Exchange request cannot be marked received in its current state",
        ));
    }

    let order = orders::Entity::find_by_id(row.order_id)
        .one(&prep_txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Original order not found"))?;

    let detail = order_details::Entity::find_by_id(row.order_detail_id)
        .one(&prep_txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Original order item not found"))?;

    if !already_received {
        // Restore the original variant's stock — the physical item is back in the store.
        let restore = prep_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Inventory SET QuantityAvailable = QuantityAvailable + ? WHERE VariantID = ?"#,
                [row.quantity.into(), detail.variant_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if restore.rows_affected() == 0 {
            return Err(Status::failed_precondition(format!(
                "No inventory row exists for variant {} while restoring the exchanged item",
                detail.variant_id
            )));
        }

        prep_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE OrderDetails SET item_status = 'exchanged' WHERE OrderDetailID = ?"#,
                [detail.order_detail_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;

        prep_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE ExchangeRequests
                   SET status = 'received',
                       received_at = COALESCE(received_at, UTC_TIMESTAMP())
                   WHERE exchange_id = ?"#,
                [row.exchange_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;

        let _ = create_order_event(
            &prep_txn,
            Request::new(CreateOrderEventRequest {
                order_id: row.order_id,
                event_type: "exchange_received".to_string(),
                from_status: Some(current_status),
                to_status: Some(STATUS_RECEIVED.to_string()),
                actor_type: "admin".to_string(),
                message: Some(format!(
                    "Exchange {} item received at store; original stock restored",
                    row.exchange_id
                )),
            }),
        )
        .await;
    }

    prep_txn.commit().await.map_err(map_db_error_to_status)?;

    // Separate transaction: place_order_admin manages its own, and can't run nested inside one
    // of ours. Line item is recorded at the real per-unit price (for accurate order/shipment
    // records) but the order total is $0 — this replacement was already paid for as part of the
    // original order, so it must never look like a second real sale in revenue reporting.
    let replacement = place_order_admin::place_order_admin(
        db,
        Request::new(PlaceOrderAdminRequest {
            user_id: row.user_id,
            shipping_address_id: order.shipping_address_id,
            payment_method: "cod".to_string(),
            line_items: vec![PlaceOrderAdminLineItem {
                variant_id: row.desired_variant_id,
                quantity: row.quantity,
                price_paise: 0,
            }],
            shipping_minor: Some(0),
            applied_coupon_id: None,
            applied_coupon_code: None,
            applied_discount_paise: None,
        }),
    )
    .await?
    .into_inner()
    .items
    .into_iter()
    .next()
    .ok_or_else(|| Status::internal("place_order_admin returned no order"))?;

    let finish_txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(format!("failed to begin exchange-completion txn: {e}")))?;
    finish_txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE ExchangeRequests SET status = 'completed', replacement_order_id = ? WHERE exchange_id = ?"#,
            [replacement.order_id.into(), row.exchange_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    let _ = create_order_event(
        &finish_txn,
        Request::new(CreateOrderEventRequest {
            order_id: row.order_id,
            event_type: "exchange_completed".to_string(),
            from_status: Some(STATUS_RECEIVED.to_string()),
            to_status: Some(STATUS_COMPLETED.to_string()),
            actor_type: "admin".to_string(),
            message: Some(format!(
                "Exchange {} completed — replacement order {} created",
                row.exchange_id, replacement.order_id
            )),
        }),
    )
    .await;
    finish_txn.commit().await.map_err(map_db_error_to_status)?;

    let updated = exchange_requests::Entity::find_by_id(row.exchange_id)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Exchange request not found"))?;
    Ok(Response::new(ExchangeRequestsResponse {
        items: vec![exchange_response(&updated)],
    }))
}
