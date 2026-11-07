//! Durable actor-scoped checkout claims. Never reclaim an ambiguous provider write.
use crate::handlers::{idempotency::compute_request_hash, orders::order_response};
use core_db_entities::entity::orders;
use proto::proto::core::{OrdersResponse, PlaceOrderRequest};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, Statement,
    TransactionTrait,
};
use tonic::{Response, Status};

pub struct Lease {
    pub user: i64,
    pub key: String,
    pub token: String,
    pub receipt: String,
}
pub enum Claim {
    Replay(Response<OrdersResponse>),
    Acquired(Lease),
}
fn err(e: sea_orm::DbErr) -> Status {
    Status::internal(e.to_string())
}

fn payload(req: &PlaceOrderRequest) -> String {
    serde_json::json!({"v":2,"customer":req.user_id,"address":req.shipping_address_id,
        "cart_ids":req.selected_cart_ids,"coupon":req.coupon_code,"payment_mode":req.payment_mode})
    .to_string()
}

#[allow(clippy::result_large_err)]
pub fn normalize(req: &mut PlaceOrderRequest) -> Result<String, Status> {
    if req.user_id <= 0 || req.shipping_address_id <= 0 {
        return Err(Status::invalid_argument(
            "Customer and address are required",
        ));
    }
    req.selected_cart_ids = super::place_order::validate_selected_cart_ids(&req.selected_cart_ids)?;
    req.selected_cart_ids.sort_unstable();
    req.payment_mode = Some(
        req.payment_mode
            .as_deref()
            .unwrap_or("prepaid")
            .trim()
            .to_ascii_lowercase(),
    );
    if !matches!(req.payment_mode.as_deref(), Some("prepaid" | "cod")) {
        return Err(Status::invalid_argument(
            "payment_mode must be prepaid or cod",
        ));
    }
    req.coupon_code = req
        .coupon_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_uppercase);
    Ok(compute_request_hash(&payload(req)))
}

pub async fn claim(
    db: &DatabaseConnection,
    req: &PlaceOrderRequest,
    key: String,
    hash: &str,
) -> Result<Claim, Status> {
    if key.is_empty() || key.len() > 191 {
        return Err(Status::invalid_argument(
            "idempotency key must contain 1..191 bytes",
        ));
    }
    let txn = db.begin().await.map_err(err)?;
    // Legacy hashes include mutable cart data. Never interpret them as v2 or make
    // a new purchase on their behalf, even if the original cart has disappeared.
    if txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT id FROM IdempotencyKeys WHERE scope = 'place_order' AND `key` = ? LIMIT 1",
            [key.clone().into()],
        ))
        .await
        .map_err(err)?
        .is_some()
    {
        return Err(Status::already_exists(
            "Legacy checkout key needs order lookup/reconciliation; do not place another order",
        ));
    }
    let token = uuid::Uuid::new_v4().to_string();
    let receipt = format!("chk_{}", uuid::Uuid::new_v4().simple());
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "INSERT INTO CheckoutAttemptsV2(customer_id,operation,request_key,fingerprint,request_json,token,state,receipt,expires_at) VALUES (?,'place_order',?,?,?,?,'preparing',?,DATE_ADD(UTC_TIMESTAMP(6),INTERVAL 15 MINUTE)) ON DUPLICATE KEY UPDATE customer_id=customer_id",
        [req.user_id.into(), key.clone().into(), hash.into(), payload(req).into(), token.clone().into(), receipt.into()])).await.map_err(err)?;
    let row = txn.query_one(Statement::from_sql_and_values(DbBackend::MySql,
        "SELECT *, expires_at < UTC_TIMESTAMP(6) AS expired FROM CheckoutAttemptsV2 WHERE customer_id=? AND operation='place_order' AND request_key=? FOR UPDATE",
        [req.user_id.into(),key.clone().into()])).await.map_err(err)?.ok_or_else(|| Status::internal("checkout claim missing"))?;
    let fingerprint: String = row
        .try_get("", "fingerprint")
        .map_err(|e| Status::internal(e.to_string()))?;
    let version: i32 = row
        .try_get("", "fingerprint_version")
        .map_err(|e| Status::internal(e.to_string()))?;
    if version != 2 {
        return Err(Status::failed_precondition(
            "Unknown checkout fingerprint version; reconciliation required",
        ));
    }
    if fingerprint != hash {
        return Err(Status::already_exists(
            "Checkout key was used with different inputs",
        ));
    }
    let state: String = row
        .try_get("", "state")
        .map_err(|e| Status::internal(e.to_string()))?;
    if state == "completed" {
        let id: i64 = row
            .try_get("", "order_id")
            .map_err(|e| Status::internal(e.to_string()))?;
        let order = orders::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(err)?
            .ok_or_else(|| Status::internal("Replay order missing; reconciliation required"))?;
        if order.user_id != req.user_id {
            return Err(Status::permission_denied("Checkout ownership mismatch"));
        }
        txn.commit().await.map_err(err)?;
        return Ok(Claim::Replay(Response::new(OrdersResponse {
            items: vec![order_response::from_model(&order)],
        })));
    }
    let current: String = row
        .try_get("", "token")
        .map_err(|e| Status::internal(e.to_string()))?;
    let expired: i8 = row
        .try_get("", "expired")
        .map_err(|e| Status::internal(e.to_string()))?;
    if current != token && state != "retryable" && !(state == "preparing" && expired != 0) {
        return Err(Status::failed_precondition(if state == "preparing" {
            "Checkout is in progress; retry the same key"
        } else {
            "Checkout outcome unresolved; reconciliation required, do not submit again"
        }));
    }
    let receipt: String = row
        .try_get("", "receipt")
        .map_err(|e| Status::internal(e.to_string()))?;
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE CheckoutAttemptsV2 SET token=?,state='preparing',last_error=NULL,expires_at=DATE_ADD(UTC_TIMESTAMP(6),INTERVAL 15 MINUTE),updated_at=UTC_TIMESTAMP(6) WHERE customer_id=? AND operation='place_order' AND request_key=?",
        [token.clone().into(),req.user_id.into(),key.clone().into()])).await.map_err(err)?;
    txn.commit().await.map_err(err)?;
    Ok(Claim::Acquired(Lease {
        user: req.user_id,
        key,
        token,
        receipt,
    }))
}

impl Lease {
    pub async fn total(&self, db: &DatabaseConnection, amount: i64) -> Result<(), Status> {
        let result=db.execute(Statement::from_sql_and_values(DbBackend::MySql,
            "UPDATE CheckoutAttemptsV2 SET amount_paise=?,updated_at=UTC_TIMESTAMP(6) WHERE customer_id=? AND operation='place_order' AND request_key=? AND token=? AND state='preparing' AND expires_at>=UTC_TIMESTAMP(6)",
            [amount.into(),self.user.into(),self.key.clone().into(),self.token.clone().into()])).await.map_err(err)?;
        if result.rows_affected() != 1 {
            return Err(Status::aborted("Checkout claim expired or superseded"));
        }
        Ok(())
    }
    pub async fn state(
        &self,
        db: &DatabaseConnection,
        from: &str,
        to: &str,
        gateway: Option<&str>,
    ) -> Result<(), Status> {
        let result = db.execute(Statement::from_sql_and_values(DbBackend::MySql,
            "UPDATE CheckoutAttemptsV2 SET state=?,gateway_order_id=COALESCE(?,gateway_order_id),updated_at=UTC_TIMESTAMP(6) WHERE customer_id=? AND operation='place_order' AND request_key=? AND token=? AND state=? AND (? <> 'preparing' OR expires_at >= UTC_TIMESTAMP(6))",
            [to.into(),gateway.map(str::to_owned).into(),self.user.into(),self.key.clone().into(),self.token.clone().into(),from.into(),from.into()])).await.map_err(err)?;
        if result.rows_affected() != 1 {
            return Err(Status::aborted("Checkout claim expired or superseded"));
        }
        Ok(())
    }
    pub async fn complete(&self, txn: &DatabaseTransaction, order: i64) -> Result<(), Status> {
        let result = txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
            "UPDATE CheckoutAttemptsV2 SET state='completed',order_id=?,updated_at=UTC_TIMESTAMP(6) WHERE customer_id=? AND operation='place_order' AND request_key=? AND token=? AND state='ready' AND expires_at>=UTC_TIMESTAMP(6)",
            [order.into(),self.user.into(),self.key.clone().into(),self.token.clone().into()])).await.map_err(err)?;
        if result.rows_affected() != 1 {
            return Err(Status::aborted(
                "Checkout claimant no longer owns this attempt",
            ));
        }
        Ok(())
    }
}
