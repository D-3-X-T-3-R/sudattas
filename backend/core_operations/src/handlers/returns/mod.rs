use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::order_policy;
use chrono::{DateTime, Utc};
use core_db_entities::entity::{
    order_details, order_status, orders, payment_intents, refunds, return_request_items,
    return_requests,
};
use proto::proto::core::{
    AdminMarkReturnReceivedRequest, AdminUpdateReturnStatusRequest, CreateOrderEventRequest,
    RequestReturnRequest, ReturnRequestItemResponse, ReturnRequestResponse, ReturnRequestsResponse,
    SearchReturnRequestsRequest,
};
use sea_orm::ActiveModelTrait;
use sea_orm::{
    sea_query::LockType, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, Statement,
};
use tonic::{Request, Response, Status};

const STATUS_REQUESTED: &str = "requested";
const STATUS_APPROVED: &str = "approved";
const STATUS_IN_TRANSIT: &str = "in_transit";
const STATUS_RECEIVED: &str = "received";
const STATUS_REFUND_PENDING: &str = "refund_pending";
const STATUS_REFUNDED: &str = "refunded";
const STATUS_REJECTED: &str = "rejected";
const STATUS_CANCELLED: &str = "cancelled";

fn line_total_minor_from_detail(detail: &order_details::Model) -> i64 {
    if detail.line_total_minor > 0 {
        return detail.line_total_minor;
    }
    if detail.line_total_minor == 0 && detail.discount_minor.is_some() {
        return 0;
    }
    i64::from(detail.unit_price_minor).saturating_mul(detail.quantity.max(0))
}

fn normalize_payment_method(raw: Option<&str>) -> String {
    raw.unwrap_or("prepaid").trim().to_ascii_lowercase()
}

fn normalize_status(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn status_is_open_for_duplicate_check(status: &str) -> bool {
    !matches!(status, STATUS_REJECTED | STATUS_CANCELLED)
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

async fn load_order_status_name(
    txn: &DatabaseTransaction,
    status_id: i64,
) -> Result<Option<String>, Status> {
    let row = order_status::Entity::find_by_id(status_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    Ok(row.map(|r| r.status_name))
}

/// Shared with the exchanges handler (`handlers::exchanges`) — both request flows use the same
/// "must be delivered, within N days of delivery" eligibility window.
pub(crate) async fn load_delivered_at(
    txn: &DatabaseTransaction,
    order: &orders::Model,
) -> Result<Option<DateTime<Utc>>, Status> {
    // Canonical source: first transition timestamp into delivered state.
    let delivered_transition_row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT MIN(created_at) AS delivered_at
               FROM OrderEvents
               WHERE order_id = ?
                 AND LOWER(COALESCE(to_status, '')) = 'delivered'"#,
            [order.order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    let delivered_transition_at = delivered_transition_row
        .as_ref()
        .and_then(|row| row.try_get::<DateTime<Utc>>("", "delivered_at").ok());
    if delivered_transition_at.is_some() {
        return Ok(delivered_transition_at);
    }

    // Legacy fallback: historical orders that predate order-event transition logging.
    let delivered_row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT MAX(delivered_at) AS delivered_at FROM Shipments WHERE order_id = ?",
            [order.order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let delivered_at = delivered_row
        .as_ref()
        .and_then(|row| row.try_get::<DateTime<Utc>>("", "delivered_at").ok());
    if delivered_at.is_some() {
        return Ok(delivered_at);
    }

    let status_name = load_order_status_name(txn, order.status_id)
        .await?
        .unwrap_or_default();
    if status_name.eq_ignore_ascii_case("delivered") {
        return Ok(Some(order.updated_at.unwrap_or(order.order_date)));
    }

    Ok(None)
}

async fn load_existing_requested_qty(
    txn: &DatabaseTransaction,
    order_id: i64,
    order_detail_id: i64,
) -> Result<i64, Status> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT CAST(COALESCE(SUM(rri.quantity), 0) AS SIGNED) AS requested_qty
               FROM ReturnRequestItems rri
               JOIN ReturnRequests rr ON rr.return_id = rri.return_id
               WHERE rr.order_id = ?
                 AND rri.order_detail_id = ?
                 AND rr.status NOT IN ('rejected', 'cancelled')
               FOR UPDATE"#,
            [order_id.into(), order_detail_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::internal("ReturnRequestItems duplicate check query returned no row")
        })?;

    row.try_get::<i64>("", "requested_qty")
        .map_err(|e| Status::internal(e.to_string()))
}

async fn load_existing_requested_refund_minor(
    txn: &DatabaseTransaction,
    order_id: i64,
    order_detail_id: i64,
) -> Result<i64, Status> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT CAST(COALESCE(SUM(rri.refund_amount_minor), 0) AS SIGNED) AS requested_refund_minor
               FROM ReturnRequestItems rri
               JOIN ReturnRequests rr ON rr.return_id = rri.return_id
               WHERE rr.order_id = ?
                 AND rri.order_detail_id = ?
                 AND rr.status NOT IN ('rejected', 'cancelled')
               FOR UPDATE"#,
            [order_id.into(), order_detail_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::internal("ReturnRequestItems refund sum query returned no row")
        })?;

    row.try_get::<i64>("", "requested_refund_minor")
        .map_err(|e| Status::internal(e.to_string()))
}

async fn load_return_request_items(
    txn: &DatabaseTransaction,
    return_id: i64,
) -> Result<Vec<return_request_items::Model>, Status> {
    return_request_items::Entity::find()
        .filter(return_request_items::Column::ReturnId.eq(return_id))
        .order_by_asc(return_request_items::Column::OrderDetailId)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)
}

fn return_item_response(row: &return_request_items::Model) -> ReturnRequestItemResponse {
    ReturnRequestItemResponse {
        return_id: row.return_id,
        order_detail_id: row.order_detail_id,
        quantity: row.quantity,
        refund_amount_minor: row.refund_amount_minor,
        status: row.status.clone(),
    }
}

async fn return_response_from_model(
    txn: &DatabaseTransaction,
    row: &return_requests::Model,
) -> Result<ReturnRequestResponse, Status> {
    let items = load_return_request_items(txn, row.return_id).await?;
    Ok(ReturnRequestResponse {
        return_id: row.return_id,
        order_id: row.order_id,
        user_id: row.user_id,
        status: row.status.clone(),
        reason: row.reason.clone(),
        created_at: row.created_at.to_rfc3339(),
        received_at: row.received_at.map(|v| v.to_rfc3339()),
        refund_attempt_id: row.refund_attempt_id,
        items: items.iter().map(return_item_response).collect(),
    })
}

pub async fn request_return(
    txn: &DatabaseTransaction,
    request: Request<RequestReturnRequest>,
) -> Result<Response<ReturnRequestsResponse>, Status> {
    let req = request.into_inner();
    let reason = req.reason.trim();
    if reason.is_empty() {
        return Err(Status::invalid_argument("Return reason is required"));
    }
    if req.items.is_empty() {
        return Err(Status::invalid_argument(
            "At least one return item is required",
        ));
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

    if normalize_payment_method(order.payment_method.as_deref()) == "cod" {
        return Err(Status::failed_precondition(
            "Returns are available only for prepaid orders",
        ));
    }

    let delivered_at = load_delivered_at(txn, &order).await?;
    let delivered_at = delivered_at.ok_or_else(|| {
        Status::failed_precondition("Returns can be requested only after delivery")
    })?;
    if !order_policy::is_within_return_window(delivered_at, Utc::now()) {
        return Err(Status::failed_precondition("Return window has closed"));
    }

    let mut requested_ids = std::collections::HashSet::new();
    let mut requested_qty_by_detail_id = std::collections::BTreeMap::new();
    for item in &req.items {
        if !requested_ids.insert(item.order_detail_id) {
            return Err(Status::invalid_argument(
                "Duplicate order_detail_id in return request",
            ));
        }
        let qty = item.quantity.unwrap_or(0);
        if qty <= 0 {
            return Err(Status::invalid_argument(
                "Return quantity must be positive for each item",
            ));
        }
        requested_qty_by_detail_id.insert(item.order_detail_id, qty);
    }

    let mut details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(req.order_id))
        .filter(order_details::Column::OrderDetailId.is_in(requested_ids.iter().copied()))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if details.len() != requested_ids.len() {
        return Err(Status::not_found(
            "One or more return items do not belong to the order",
        ));
    }

    // Process (and FOR UPDATE lock ReturnRequestItems via the loop below) in sorted
    // OrderDetailID order (not DB return order, which is unspecified without an ORDER BY) for
    // the same deadlock-avoidance reason as the inventory locking in place_order.rs.
    details.sort_by_key(|d| d.order_detail_id);

    let mut validated_items: Vec<(i64, i64, i64)> = Vec::with_capacity(details.len());
    for detail in &details {
        if detail.item_status.eq_ignore_ascii_case("cancelled") {
            return Err(Status::failed_precondition(
                "Cancelled order items cannot be returned",
            ));
        }

        let requested_qty = requested_qty_by_detail_id
            .get(&detail.order_detail_id)
            .copied()
            .unwrap_or(0);
        if requested_qty <= 0 {
            return Err(Status::invalid_argument(
                "Return quantity must be positive for each item",
            ));
        }
        if requested_qty > detail.quantity {
            return Err(Status::failed_precondition(
                "Requested return quantity exceeds purchased quantity",
            ));
        }

        let already_requested_qty =
            load_existing_requested_qty(txn, req.order_id, detail.order_detail_id).await?;
        let available_qty = detail.quantity.saturating_sub(already_requested_qty);
        if requested_qty > available_qty {
            return Err(Status::failed_precondition(
                "Selected item quantity is already part of an existing return request",
            ));
        }

        let already_requested_refund_minor =
            load_existing_requested_refund_minor(txn, req.order_id, detail.order_detail_id).await?;
        let line_total_minor = line_total_minor_from_detail(detail).max(0);
        let remaining_line_refund_minor = line_total_minor
            .saturating_sub(already_requested_refund_minor)
            .max(0);
        let prorated_refund_minor =
            line_total_minor.saturating_mul(requested_qty) / detail.quantity.max(1);
        // If this request consumes all remaining quantity, assign the exact remaining line refund
        // to avoid cumulative rounding drift across multiple partial return requests.
        let refund_amount_minor = if requested_qty == available_qty {
            remaining_line_refund_minor
        } else {
            prorated_refund_minor.min(remaining_line_refund_minor)
        };
        validated_items.push((detail.order_detail_id, requested_qty, refund_amount_minor));
    }

    let return_request = return_requests::ActiveModel {
        return_id: sea_orm::ActiveValue::NotSet,
        order_id: sea_orm::ActiveValue::Set(req.order_id),
        user_id: sea_orm::ActiveValue::Set(req.user_id),
        status: sea_orm::ActiveValue::Set(STATUS_REQUESTED.to_string()),
        reason: sea_orm::ActiveValue::Set(reason.to_string()),
        created_at: sea_orm::ActiveValue::NotSet,
        received_at: sea_orm::ActiveValue::Set(None),
        refund_attempt_id: sea_orm::ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .map_err(map_db_error_to_status)?;

    for (order_detail_id, quantity, refund_amount_minor) in &validated_items {
        return_request_items::ActiveModel {
            return_id: sea_orm::ActiveValue::Set(return_request.return_id),
            order_detail_id: sea_orm::ActiveValue::Set(*order_detail_id),
            quantity: sea_orm::ActiveValue::Set(*quantity),
            refund_amount_minor: sea_orm::ActiveValue::Set(*refund_amount_minor),
            status: sea_orm::ActiveValue::Set(STATUS_REQUESTED.to_string()),
        }
        .insert(txn)
        .await
        .map_err(map_db_error_to_status)?;
    }

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: req.order_id,
            event_type: "return_requested".to_string(),
            from_status: None,
            to_status: Some(STATUS_REQUESTED.to_string()),
            actor_type: "customer".to_string(),
            message: Some(format!(
                "Return requested for {} item(s): {}",
                validated_items.len(),
                reason
            )),
        }),
    )
    .await;

    Ok(Response::new(ReturnRequestsResponse {
        items: vec![return_response_from_model(txn, &return_request).await?],
    }))
}

pub async fn search_return_requests(
    txn: &DatabaseTransaction,
    request: Request<SearchReturnRequestsRequest>,
) -> Result<Response<ReturnRequestsResponse>, Status> {
    let req = request.into_inner();
    let rows = return_requests::Entity::find()
        .apply_if(req.return_id, |query, v| {
            query.filter(return_requests::Column::ReturnId.eq(v))
        })
        .apply_if(req.order_id, |query, v| {
            query.filter(return_requests::Column::OrderId.eq(v))
        })
        .apply_if(req.user_id, |query, v| {
            query.filter(return_requests::Column::UserId.eq(v))
        })
        .order_by_desc(return_requests::Column::ReturnId)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        items.push(return_response_from_model(txn, row).await?);
    }
    Ok(Response::new(ReturnRequestsResponse { items }))
}

pub async fn admin_mark_return_received(
    txn: &DatabaseTransaction,
    request: Request<AdminMarkReturnReceivedRequest>,
) -> Result<Response<ReturnRequestsResponse>, Status> {
    let req = request.into_inner();
    let return_row = return_requests::Entity::find_by_id(req.return_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Return request not found"))?;

    let current_status = normalize_status(&return_row.status);
    if !status_allows_mark_received(&current_status)
        && current_status != STATUS_RECEIVED
        && current_status != STATUS_REFUND_PENDING
        && current_status != STATUS_REFUNDED
    {
        return Err(Status::failed_precondition(
            "Return request cannot be marked received in its current state",
        ));
    }

    if return_row.refund_attempt_id.is_some()
        && matches!(
            current_status.as_str(),
            STATUS_RECEIVED | STATUS_REFUND_PENDING | STATUS_REFUNDED
        )
    {
        return Ok(Response::new(ReturnRequestsResponse {
            items: vec![return_response_from_model(txn, &return_row).await?],
        }));
    }

    let order = orders::Entity::find_by_id(return_row.order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;

    if normalize_payment_method(order.payment_method.as_deref()) == "cod" {
        return Err(Status::failed_precondition(
            "COD orders are not eligible for returns",
        ));
    }

    let item_rows = return_request_items::Entity::find()
        .filter(return_request_items::Column::ReturnId.eq(return_row.return_id))
        .order_by_asc(return_request_items::Column::OrderDetailId)
        .lock(LockType::Update)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if item_rows.is_empty() {
        return Err(Status::failed_precondition(
            "Return request has no return items",
        ));
    }

    if return_row.refund_attempt_id.is_none() {
        let total_refund_minor: i64 = item_rows.iter().map(|row| row.refund_amount_minor).sum();
        if total_refund_minor <= 0 {
            return Err(Status::failed_precondition(
                "Return request refund amount is not positive",
            ));
        }

        let settled_processed: i64 = refunds::Entity::find()
            .filter(refunds::Column::OrderId.eq(order.order_id))
            .all(txn)
            .await
            .map_err(map_db_error_to_status)?
            .iter()
            .map(|r| i64::from(r.amount_paise))
            .sum();

        let intent = payment_intents::Entity::find()
            .filter(payment_intents::Column::OrderId.eq(order.order_id))
            .filter(
                payment_intents::Column::Status
                    .eq(core_db_entities::entity::sea_orm_active_enums::Status::Processed),
            )
            .filter(payment_intents::Column::RazorpayPaymentId.is_not_null())
            .order_by_desc(payment_intents::Column::IntentId)
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
            .ok_or_else(|| {
                Status::failed_precondition(
                    "No canonical captured payment intent available for return refund",
                )
            })?;

        let payment_id = intent
            .razorpay_payment_id
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                Status::failed_precondition(
                    "No canonical captured payment intent available for return refund",
                )
            })?;

        let remaining_cap = i64::from(intent.amount_paise).saturating_sub(settled_processed);
        let amount_to_send = total_refund_minor.min(remaining_cap).max(0);
        if amount_to_send <= 0 {
            return Err(Status::failed_precondition(
                "Refund amount exceeds remaining captured payment",
            ));
        }

        let idem = format!(
            "return_{}_{}_{}",
            return_row.return_id, order.order_id, payment_id
        );
        let inserted = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"INSERT INTO RefundAttempts (
                       order_id, payment_intent_id, razorpay_payment_id,
                       amount_requested_paise, amount_sent_to_gateway_paise,
                       gateway_refund_id, status, provider_error, idempotency_key
                   ) VALUES (?, ?, ?, ?, ?, NULL, 'pending_external', NULL, ?)"#,
                [
                    order.order_id.into(),
                    intent.intent_id.into(),
                    payment_id.into(),
                    total_refund_minor.into(),
                    amount_to_send.into(),
                    idem.into(),
                ],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        let attempt_id = i64::try_from(inserted.last_insert_id())
            .map_err(|_| Status::internal("refund attempt id overflow"))?;

        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE ReturnRequests
               SET status = 'received',
                   received_at = COALESCE(received_at, UTC_TIMESTAMP()),
                   refund_attempt_id = ?
               WHERE return_id = ?"#,
            [attempt_id.into(), return_row.return_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    } else if current_status == STATUS_REQUESTED
        || current_status == STATUS_APPROVED
        || current_status == STATUS_IN_TRANSIT
    {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE ReturnRequests
               SET status = 'received',
                   received_at = COALESCE(received_at, UTC_TIMESTAMP())
               WHERE return_id = ?"#,
            [return_row.return_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    }

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ReturnRequestItems SET status = 'received' WHERE return_id = ?",
        [return_row.return_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: return_row.order_id,
            event_type: "return_received".to_string(),
            from_status: Some(current_status),
            to_status: Some(STATUS_RECEIVED.to_string()),
            actor_type: "admin".to_string(),
            message: Some(format!(
                "Return {} marked received at store",
                return_row.return_id
            )),
        }),
    )
    .await;

    let updated = return_requests::Entity::find_by_id(return_row.return_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Return request not found"))?;
    Ok(Response::new(ReturnRequestsResponse {
        items: vec![return_response_from_model(txn, &updated).await?],
    }))
}

pub async fn admin_update_return_status(
    txn: &DatabaseTransaction,
    request: Request<AdminUpdateReturnStatusRequest>,
) -> Result<Response<ReturnRequestsResponse>, Status> {
    let req = request.into_inner();
    let target_status = normalize_status(&req.status);
    if !status_allows_admin_update_target(&target_status) {
        return Err(Status::invalid_argument(
            "Unsupported return status transition target",
        ));
    }

    let return_row = return_requests::Entity::find_by_id(req.return_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Return request not found"))?;
    let current_status = normalize_status(&return_row.status);

    if current_status == target_status {
        return Ok(Response::new(ReturnRequestsResponse {
            items: vec![return_response_from_model(txn, &return_row).await?],
        }));
    }

    if matches!(
        current_status.as_str(),
        STATUS_RECEIVED | STATUS_REFUND_PENDING | STATUS_REFUNDED
    ) && matches!(target_status.as_str(), STATUS_REJECTED | STATUS_CANCELLED)
    {
        return Err(Status::failed_precondition(
            "Return cannot be rejected or cancelled after store receipt",
        ));
    }

    if matches!(target_status.as_str(), STATUS_REJECTED | STATUS_CANCELLED)
        && !status_allows_admin_reject_or_cancel(&current_status)
    {
        return Err(Status::failed_precondition(
            "Return cannot be rejected or cancelled in its current state",
        ));
    }

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ReturnRequests SET status = ? WHERE return_id = ?",
        [target_status.clone().into(), return_row.return_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ReturnRequestItems SET status = ? WHERE return_id = ?",
        [target_status.clone().into(), return_row.return_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let note = req.note.unwrap_or_default();
    let note = note.trim();
    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: return_row.order_id,
            event_type: format!("return_{}", target_status),
            from_status: Some(current_status),
            to_status: Some(target_status.clone()),
            actor_type: "admin".to_string(),
            message: if note.is_empty() {
                Some(format!(
                    "Return {} updated to {}",
                    return_row.return_id, target_status
                ))
            } else {
                Some(format!(
                    "Return {} updated to {}: {}",
                    return_row.return_id, target_status, note
                ))
            },
        }),
    )
    .await;

    let updated = return_requests::Entity::find_by_id(return_row.return_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Return request not found"))?;
    Ok(Response::new(ReturnRequestsResponse {
        items: vec![return_response_from_model(txn, &updated).await?],
    }))
}

pub async fn set_return_status_and_items(
    txn: &DatabaseTransaction,
    return_id: i64,
    status: &str,
) -> Result<(), Status> {
    let normalized = normalize_status(status);
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ReturnRequests SET status = ? WHERE return_id = ?",
        [normalized.clone().into(), return_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let item_status = if normalized == STATUS_REFUND_PENDING {
        STATUS_REFUND_PENDING
    } else {
        normalized.as_str()
    };
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE ReturnRequestItems SET status = ? WHERE return_id = ?",
        [item_status.into(), return_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    Ok(())
}

pub async fn find_return_by_refund_attempt(
    txn: &DatabaseTransaction,
    attempt_id: i64,
) -> Result<Option<return_requests::Model>, Status> {
    let row = return_requests::Entity::find()
        .filter(return_requests::Column::RefundAttemptId.eq(attempt_id))
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if let Some(ref request) = row {
        if !status_is_open_for_duplicate_check(&normalize_status(&request.status))
            && normalize_status(&request.status) != STATUS_REFUNDED
        {
            return Ok(None);
        }
    }
    Ok(row)
}
