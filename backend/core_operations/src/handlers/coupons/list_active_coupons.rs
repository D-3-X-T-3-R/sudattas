//! Customer-facing: list currently-usable coupons (active, in date window, not exhausted) so the
//! storefront can show an "Available offers" list instead of requiring the customer to already
//! know a code. Does not evaluate per-customer/scope eligibility against a specific cart — that
//! still happens for real at `estimate_checkout_shipping`/`place_order` when a code is applied.

use chrono::Utc;
use core_db_entities::entity::coupons;
use core_db_entities::entity::sea_orm_active_enums::CouponStatus;
use proto::proto::core::{ListActiveCouponsRequest, PublicCouponResponse, PublicCouponsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn list_active_coupons(
    txn: &DatabaseTransaction,
    request: Request<ListActiveCouponsRequest>,
) -> Result<Response<PublicCouponsResponse>, Status> {
    let _req = request.into_inner();
    let now = Utc::now();

    let candidates = coupons::Entity::find()
        .filter(
            coupons::Column::CouponStatus
                .eq(CouponStatus::Active)
                .or(coupons::Column::CouponStatus.is_null()),
        )
        .all(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut eligible: Vec<coupons::Model> = candidates
        .into_iter()
        .filter(|c| c.starts_at <= now)
        .filter(|c| c.ends_at.map(|e| e >= now).unwrap_or(true))
        .filter(|c| match (c.usage_limit, c.usage_count) {
            (Some(limit), Some(used)) => used < limit,
            _ => true,
        })
        .collect();

    // Soonest-expiring first; coupons with no expiry sort last.
    eligible.sort_by_key(|c| c.ends_at.unwrap_or(chrono::DateTime::<Utc>::MAX_UTC));

    let items = eligible
        .into_iter()
        .map(|c| PublicCouponResponse {
            coupon_id: c.coupon_id,
            code: c.code,
            discount_type: format!("{:?}", c.discount_type).to_lowercase(),
            discount_value: c.discount_value,
            min_order_value_paise: c.min_order_value_paise,
            ends_at: c.ends_at.map(|e| e.to_rfc3339()),
        })
        .collect();

    Ok(Response::new(PublicCouponsResponse { items }))
}
