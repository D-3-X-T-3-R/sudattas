//! P1 Coupons & promotions: per-customer usage limit and allowlist/denylist scope checks.

use core_db_entities::entity::{
    coupon_redemptions, coupon_scope, coupons, sea_orm_active_enums::ScopeType,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter};
use tonic::Status;

/// Cart line for scope check: product_id and its category_id (from Products.CategoryID).
#[derive(Clone, Debug)]
pub struct CartProduct {
    pub product_id: i64,
    pub category_id: Option<i64>,
}

/// Returns true if the user may use this coupon (per-customer limit not exceeded).
/// If coupon has no max_uses_per_customer, returns true.
pub async fn check_per_customer_limit(
    txn: &DatabaseTransaction,
    coupon_id: i64,
    user_id: i64,
) -> Result<bool, Status> {
    let coupon = coupons::Entity::find_by_id(coupon_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let limit = match coupon {
        Some(c) => c.max_uses_per_customer,
        None => return Ok(false),
    };

    let Some(limit) = limit else {
        return Ok(true);
    };

    let count = coupon_redemptions::Entity::find()
        .filter(coupon_redemptions::Column::CouponId.eq(coupon_id))
        .filter(coupon_redemptions::Column::UserId.eq(user_id))
        .count(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok((count as i32) < limit)
}

/// Returns true if the coupon is applicable to this cart (allowlist/denylist rules).
/// - No scope rows: coupon applies to any cart.
/// - Allowlist rows: cart must contain at least one product (or product in category) in the list.
/// - Denylist rows: cart must not contain any product (or product in category) in the list.
pub async fn check_coupon_scope(
    txn: &DatabaseTransaction,
    coupon_id: i64,
    cart: &[CartProduct],
) -> Result<bool, Status> {
    let scope_rows = coupon_scope::Entity::find()
        .filter(coupon_scope::Column::CouponId.eq(coupon_id))
        .all(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    if scope_rows.is_empty() {
        return Ok(true);
    }

    let allow: Vec<_> = scope_rows.iter().filter(|r| r.is_allowlist == 1).collect();
    let deny: Vec<_> = scope_rows.iter().filter(|r| r.is_allowlist == 0).collect();

    for item in cart {
        for d in &deny {
            let matches = match d.scope_type {
                ScopeType::Product => item.product_id == d.scope_id,
                ScopeType::Category => item.category_id.map(|c| c == d.scope_id).unwrap_or(false),
            };
            if matches {
                return Ok(false);
            }
        }
    }

    if allow.is_empty() {
        return Ok(true);
    }

    let cart_matches_allow = cart.iter().any(|item| {
        allow.iter().any(|a| match a.scope_type {
            ScopeType::Product => item.product_id == a.scope_id,
            ScopeType::Category => item.category_id.map(|c| c == a.scope_id).unwrap_or(false),
        })
    });

    Ok(cart_matches_allow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_db_entities::entity::coupons;
    use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
    use sea_orm::{DatabaseBackend, MockDatabase, TransactionTrait};

    #[tokio::test]
    async fn check_per_customer_limit_no_limit_returns_true() {
        let coupon = coupons::Model {
            coupon_id: 1,
            code: "X".to_string(),
            discount_type: DiscountType::Percentage,
            discount_value: 10,
            min_order_value_paise: None,
            usage_limit: None,
            usage_count: None,
            max_uses_per_customer: None,
            coupon_status: Some(CouponStatus::Active),
            starts_at: chrono::Utc::now(),
            ends_at: None,
            created_at: None,
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![coupon]])
            .into_connection();
        let txn = db.begin().await.expect("begin");
        let ok = check_per_customer_limit(&txn, 1, 100).await.expect("check");
        assert!(ok);
    }

    #[tokio::test]
    async fn check_coupon_scope_empty_scope_returns_true() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![Vec::<coupon_scope::Model>::new()])
            .into_connection();
        let txn = db.begin().await.expect("begin");
        let cart = vec![CartProduct {
            product_id: 1,
            category_id: Some(5),
        }];
        let ok = check_coupon_scope(&txn, 1, &cart).await.expect("check");
        assert!(ok);
    }

    #[tokio::test]
    async fn check_coupon_scope_allowlist_product_match_returns_true() {
        use core_db_entities::entity::sea_orm_active_enums::ScopeType;
        let scope = coupon_scope::Model {
            id: 1,
            coupon_id: 1,
            scope_type: ScopeType::Product,
            scope_id: 10,
            is_allowlist: 1,
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![scope]])
            .into_connection();
        let txn = db.begin().await.expect("begin");
        let cart = vec![CartProduct {
            product_id: 10,
            category_id: Some(5),
        }];
        let ok = check_coupon_scope(&txn, 1, &cart).await.expect("check");
        assert!(ok);
    }

    #[tokio::test]
    async fn check_coupon_scope_denylist_product_match_returns_false() {
        use core_db_entities::entity::sea_orm_active_enums::ScopeType;
        let scope = coupon_scope::Model {
            id: 1,
            coupon_id: 1,
            scope_type: ScopeType::Product,
            scope_id: 10,
            is_allowlist: 0,
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![scope]])
            .into_connection();
        let txn = db.begin().await.expect("begin");
        let cart = vec![CartProduct {
            product_id: 10,
            category_id: Some(5),
        }];
        let ok = check_coupon_scope(&txn, 1, &cart).await.expect("check");
        assert!(!ok);
    }
}
