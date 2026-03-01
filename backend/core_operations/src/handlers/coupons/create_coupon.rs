//! P1 Admin: create coupon (code, discount, limits, expiry).

use crate::handlers::db_errors::map_db_error_to_status;
use chrono::{DateTime, Utc};
use core_db_entities::entity::coupons;
use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
use proto::proto::core::{CouponAdminResponse, CouponsAdminResponse, CreateCouponRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_coupon(
    txn: &DatabaseTransaction,
    request: Request<CreateCouponRequest>,
) -> Result<Response<CouponsAdminResponse>, Status> {
    let req = request.into_inner();

    if req.code.is_empty() {
        return Err(Status::invalid_argument("code is required"));
    }
    let starts_at = DateTime::parse_from_rfc3339(&req.starts_at)
        .map_err(|_| Status::invalid_argument("starts_at must be RFC3339"))?
        .with_timezone(&Utc);
    let ends_at = req
        .ends_at
        .as_ref()
        .filter(|s| !s.is_empty())
        .and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

    let discount_type = match req.discount_type.to_lowercase().as_str() {
        "percentage" => DiscountType::Percentage,
        "fixed_amount" => DiscountType::FixedAmount,
        _ => {
            return Err(Status::invalid_argument(
                "discount_type must be percentage or fixed_amount",
            ))
        }
    };

    let model = coupons::ActiveModel {
        coupon_id: ActiveValue::NotSet,
        code: ActiveValue::Set(req.code),
        discount_type: ActiveValue::Set(discount_type),
        discount_value: ActiveValue::Set(req.discount_value),
        min_order_value_paise: ActiveValue::Set(req.min_order_value_paise),
        usage_limit: ActiveValue::Set(req.usage_limit),
        usage_count: ActiveValue::Set(Some(0)),
        max_uses_per_customer: ActiveValue::Set(req.max_uses_per_customer),
        coupon_status: ActiveValue::Set(Some(CouponStatus::Active)),
        starts_at: ActiveValue::Set(starts_at),
        ends_at: ActiveValue::Set(ends_at),
        created_at: ActiveValue::Set(Some(Utc::now())),
    };

    let inserted = model.insert(txn).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(CouponsAdminResponse {
        items: vec![model_to_admin_response(inserted)],
    }))
}

pub fn model_to_admin_response(m: coupons::Model) -> CouponAdminResponse {
    let status = m
        .coupon_status
        .as_ref()
        .map(|s| format!("{:?}", s).to_lowercase())
        .unwrap_or_else(|| "active".to_string());
    CouponAdminResponse {
        coupon_id: m.coupon_id,
        code: m.code,
        discount_type: format!("{:?}", m.discount_type).to_lowercase(),
        discount_value: m.discount_value,
        min_order_value_paise: m.min_order_value_paise,
        usage_limit: m.usage_limit,
        usage_count: m.usage_count,
        max_uses_per_customer: m.max_uses_per_customer,
        status,
        starts_at: m.starts_at.to_rfc3339(),
        ends_at: m.ends_at.map(|t| t.to_rfc3339()),
    }
}
