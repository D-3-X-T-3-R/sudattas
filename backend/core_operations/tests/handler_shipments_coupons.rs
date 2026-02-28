//! Unit tests for shipments and coupons handlers using SeaORM MockDatabase.

use chrono::Utc;
use core_db_entities::entity::{coupons, shipments};
use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType, Status};
use proto::proto::core::{ApplyCouponRequest, CreateShipmentRequest};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_shipment_success() {
    use core_operations::handlers::shipments::create_shipment;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![shipments::Model {
            shipment_id: 1,
            order_id: 100,
            shiprocket_order_id: Some("sr_123".to_string()),
            awb_code: Some("AWB456".to_string()),
            carrier: Some("DTDC".to_string()),
            status: Some(Status::Pending),
            tracking_events: None,
            created_at: None,
            delivered_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateShipmentRequest {
        order_id: 100,
        shiprocket_order_id: Some("sr_123".to_string()),
        awb_code: Some("AWB456".to_string()),
        carrier: Some("DTDC".to_string()),
    });
    let result = create_shipment(&txn, req).await;
    assert!(result.is_ok(), "create_shipment should succeed: {:?}", result.err());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].order_id, 100);
    assert_eq!(res.items[0].shiprocket_order_id.as_deref(), Some("sr_123"));
    assert_eq!(res.items[0].awb_code.as_deref(), Some("AWB456"));
    assert_eq!(res.items[0].carrier.as_deref(), Some("DTDC"));
    assert_eq!(res.items[0].status, "pending");
}

#[tokio::test]
async fn apply_coupon_valid_returns_valid_and_increments_usage() {
    use core_operations::handlers::coupons::apply_coupon;

    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 1,
        code: "SAVE10".to_string(),
        discount_type: DiscountType::Percentage,
        discount_value: 10,
        min_order_value_paise: Some(1000),
        usage_limit: Some(100),
        usage_count: Some(5),
        coupon_status: Some(CouponStatus::Active),
        starts_at: now - chrono::Duration::days(1),
        ends_at: Some(now + chrono::Duration::days(7)),
        created_at: None,
    };

    // apply_coupon: check_coupon does find by code; then if valid, find by code again + update.
    // Some DB drivers may run an extra select; supply enough results.
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![
            vec![coupon_model.clone()], // check_coupon find
            vec![coupon_model.clone()], // apply_coupon find for update
            vec![coupon_model],         // optional extra select (e.g. after update)
        ])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ApplyCouponRequest {
        code: "SAVE10".to_string(),
        order_amount_paise: 50_000, // 500 INR; 10% = 5000 paise discount
    });
    let result = apply_coupon(&txn, req).await;
    assert!(result.is_ok(), "apply_coupon should succeed: {:?}", result.err());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert!(res.items[0].is_valid, "coupon should be valid");
    assert_eq!(res.items[0].code, "SAVE10");
    assert_eq!(res.items[0].discount_amount_paise, 5_000);
    assert_eq!(res.items[0].final_amount_paise, 45_000);
}

#[tokio::test]
async fn apply_coupon_not_found_returns_invalid() {
    use core_operations::handlers::coupons::apply_coupon;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<coupons::Model>::new()]) // find by code returns empty
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ApplyCouponRequest {
        code: "MISSING".to_string(),
        order_amount_paise: 10_000,
    });
    let result = apply_coupon(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert!(!res.items[0].is_valid);
    assert!(res.items[0].reason.to_lowercase().contains("not found"));
}
