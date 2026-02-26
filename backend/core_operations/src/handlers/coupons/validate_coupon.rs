use chrono::Utc;
use core_db_entities::entity::coupons;
use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
use proto::proto::core::{CouponResponse, CouponsResponse, ValidateCouponRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn validate_coupon(
    txn: &DatabaseTransaction,
    request: Request<ValidateCouponRequest>,
) -> Result<Response<CouponsResponse>, TonicStatus> {
    let req = request.into_inner();
    let response = check_coupon(txn, &req.code, req.order_amount_paise, false).await?;
    Ok(Response::new(CouponsResponse {
        items: vec![response],
    }))
}

/// Core validation logic reused by both validate and apply.
/// If `apply` is true the usage_count will be incremented inside the caller.
pub async fn check_coupon(
    txn: &DatabaseTransaction,
    code: &str,
    order_amount_paise: i64,
    _apply: bool,
) -> Result<CouponResponse, TonicStatus> {
    let coupon = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(code))
        .one(txn)
        .await
        .map_err(|e| TonicStatus::internal(e.to_string()))?;

    let coupon = match coupon {
        Some(c) => c,
        None => {
            return Ok(CouponResponse {
                coupon_id: 0,
                code: code.to_string(),
                discount_type: String::new(),
                discount_value: 0,
                discount_amount_paise: 0,
                final_amount_paise: order_amount_paise,
                is_valid: false,
                reason: "Coupon not found".to_string(),
            })
        }
    };

    // Status check — only active coupons are usable
    if coupon.coupon_status != Some(CouponStatus::Active) && coupon.coupon_status.is_some() {
        return Ok(coupon_invalid(
            coupon,
            order_amount_paise,
            "Coupon is not active",
        ));
    }

    // Date range
    let now = Utc::now();
    if now < coupon.starts_at {
        return Ok(coupon_invalid(
            coupon,
            order_amount_paise,
            "Coupon has not started yet",
        ));
    }
    if let Some(ends_at) = coupon.ends_at {
        if now > ends_at {
            return Ok(coupon_invalid(
                coupon,
                order_amount_paise,
                "Coupon has expired",
            ));
        }
    }

    // Usage limit
    if let (Some(limit), Some(used)) = (coupon.usage_limit, coupon.usage_count) {
        if used >= limit {
            return Ok(coupon_invalid(
                coupon,
                order_amount_paise,
                "Coupon usage limit reached",
            ));
        }
    }

    // Minimum order value
    if let Some(min) = coupon.min_order_value_paise {
        if order_amount_paise < min as i64 {
            return Ok(coupon_invalid(
                coupon,
                order_amount_paise,
                &format!("Order value too low; minimum is {} paise", min),
            ));
        }
    }

    let discount_amount_paise = match coupon.discount_type {
        DiscountType::Percentage => (order_amount_paise * coupon.discount_value as i64) / 100,
        DiscountType::FixedAmount => coupon.discount_value as i64,
    };
    let discount_amount_paise = discount_amount_paise.min(order_amount_paise);
    let final_amount_paise = order_amount_paise - discount_amount_paise;

    Ok(CouponResponse {
        coupon_id: coupon.coupon_id,
        code: coupon.code,
        discount_type: format!("{:?}", coupon.discount_type).to_lowercase(),
        discount_value: coupon.discount_value,
        discount_amount_paise,
        final_amount_paise,
        is_valid: true,
        reason: "OK".to_string(),
    })
}

fn coupon_invalid(coupon: coupons::Model, order_amount_paise: i64, reason: &str) -> CouponResponse {
    CouponResponse {
        coupon_id: coupon.coupon_id,
        code: coupon.code,
        discount_type: format!("{:?}", coupon.discount_type).to_lowercase(),
        discount_value: coupon.discount_value,
        discount_amount_paise: 0,
        final_amount_paise: order_amount_paise,
        is_valid: false,
        reason: reason.to_string(),
    }
}
