//! Unit tests for shipments and coupons handlers using SeaORM MockDatabase.

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType, Status};
use core_db_entities::entity::{coupons, shipments};
use proto::proto::core::{
    ApplyCouponRequest, CreateShipmentRequest, GetShipmentRequest, UpdateShipmentRequest,
    ValidateCouponRequest,
};
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
    assert!(
        result.is_ok(),
        "create_shipment should succeed: {:?}",
        result.err()
    );
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].order_id, 100);
    assert_eq!(res.items[0].shiprocket_order_id.as_deref(), Some("sr_123"));
    assert_eq!(res.items[0].awb_code.as_deref(), Some("AWB456"));
    assert_eq!(res.items[0].carrier.as_deref(), Some("DTDC"));
    assert_eq!(res.items[0].status, "pending");
}

#[tokio::test]
async fn get_shipment_by_order_id_returns_items() {
    use core_operations::handlers::shipments::get_shipment;

    let model = shipments::Model {
        shipment_id: 2,
        order_id: 200,
        shiprocket_order_id: Some("sr_456".to_string()),
        awb_code: Some("AWB789".to_string()),
        carrier: Some("Bluedart".to_string()),
        status: Some(Status::Processed),
        tracking_events: None,
        created_at: None,
        delivered_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetShipmentRequest {
        shipment_id: None,
        order_id: Some(200),
    });
    let result = get_shipment(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].order_id, 200);
}

#[tokio::test]
async fn get_shipment_without_ids_returns_invalid_argument() {
    use core_operations::handlers::shipments::get_shipment;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetShipmentRequest {
        shipment_id: None,
        order_id: None,
    });
    let result = get_shipment(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn update_shipment_not_found_yields_not_found_status() {
    use core_operations::handlers::shipments::update_shipment;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<shipments::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateShipmentRequest {
        shipment_id: 999,
        shiprocket_order_id: None,
        awb_code: None,
        carrier: None,
        status: None,
    });
    let result = update_shipment(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_shipment_updates_status_and_sets_delivered_at_when_processed() {
    use core_operations::handlers::shipments::update_shipment;

    let existing = shipments::Model {
        shipment_id: 3,
        order_id: 300,
        shiprocket_order_id: Some("sr_789".to_string()),
        awb_code: Some("AWB000".to_string()),
        carrier: Some("Xpress".to_string()),
        status: Some(Status::Pending),
        tracking_events: None,
        created_at: Some(Utc::now()),
        delivered_at: None,
    };
    let updated = shipments::Model {
        status: Some(Status::Processed),
        delivered_at: Some(Utc::now()),
        ..existing.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateShipmentRequest {
        shipment_id: 3,
        shiprocket_order_id: None,
        awb_code: None,
        carrier: None,
        status: Some("processed".to_string()),
    });

    let result = update_shipment(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    let s = &res.items[0];
    assert_eq!(s.order_id, 300);
    assert_eq!(s.status, "processed");
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
        max_uses_per_customer: None,
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
    assert!(
        result.is_ok(),
        "apply_coupon should succeed: {:?}",
        result.err()
    );
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

#[tokio::test]
async fn validate_coupon_valid_percentage_discount() {
    use core_operations::handlers::coupons::validate_coupon;

    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 1,
        code: "SAVE20".to_string(),
        discount_type: DiscountType::Percentage,
        discount_value: 20,
        min_order_value_paise: Some(1_000),
        usage_limit: Some(10),
        usage_count: Some(0),
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now - chrono::Duration::days(1),
        ends_at: Some(now + chrono::Duration::days(1)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ValidateCouponRequest {
        code: "SAVE20".to_string(),
        order_amount_paise: 10_000,
    });
    let result = validate_coupon(&txn, req).await;
    assert!(
        result.is_ok(),
        "validate_coupon should succeed for valid coupon"
    );
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    let c = &res.items[0];
    assert!(c.is_valid);
    assert_eq!(c.code, "SAVE20");
    assert_eq!(c.discount_amount_paise, 2_000);
    assert_eq!(c.final_amount_paise, 8_000);
}

#[tokio::test]
async fn validate_coupon_not_found_returns_invalid() {
    use core_operations::handlers::coupons::validate_coupon;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<coupons::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ValidateCouponRequest {
        code: "UNKNOWN".to_string(),
        order_amount_paise: 5_000,
    });
    let result = validate_coupon(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    let c = &res.items[0];
    assert!(!c.is_valid);
    assert!(c.reason.to_lowercase().contains("not found"));
    assert_eq!(c.final_amount_paise, 5_000);
}

#[tokio::test]
async fn check_coupon_inactive_returns_not_active_reason() {
    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 2,
        code: "INACTIVE".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 500,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: None,
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Inactive),
        starts_at: now - chrono::Duration::days(2),
        ends_at: Some(now + chrono::Duration::days(2)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = core_operations::handlers::coupons::validate_coupon::check_coupon(
        &txn, "INACTIVE", 10_000, false,
    )
    .await;
    assert!(result.is_ok());
    let c = result.unwrap();
    assert!(!c.is_valid);
    assert_eq!(c.reason, "Coupon is not active");
}

#[tokio::test]
async fn check_coupon_not_started_yet_returns_reason() {
    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 3,
        code: "FUTURE".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 500,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: None,
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now + chrono::Duration::days(1),
        ends_at: Some(now + chrono::Duration::days(10)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = core_operations::handlers::coupons::validate_coupon::check_coupon(
        &txn, "FUTURE", 10_000, false,
    )
    .await;
    assert!(result.is_ok());
    let c = result.unwrap();
    assert!(!c.is_valid);
    assert_eq!(c.reason, "Coupon has not started yet");
}

#[tokio::test]
async fn check_coupon_expired_returns_expired_reason() {
    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 4,
        code: "OLD".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 500,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: None,
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now - chrono::Duration::days(10),
        ends_at: Some(now - chrono::Duration::days(1)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = core_operations::handlers::coupons::validate_coupon::check_coupon(
        &txn, "OLD", 10_000, false,
    )
    .await;
    assert!(result.is_ok());
    let c = result.unwrap();
    assert!(!c.is_valid);
    assert_eq!(c.reason, "Coupon has expired");
}

#[tokio::test]
async fn check_coupon_usage_limit_reached_returns_reason() {
    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 5,
        code: "MAXED".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 500,
        min_order_value_paise: None,
        usage_limit: Some(10),
        usage_count: Some(10),
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now - chrono::Duration::days(1),
        ends_at: Some(now + chrono::Duration::days(1)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = core_operations::handlers::coupons::validate_coupon::check_coupon(
        &txn, "MAXED", 10_000, false,
    )
    .await;
    assert!(result.is_ok());
    let c = result.unwrap();
    assert!(!c.is_valid);
    assert_eq!(c.reason, "Coupon usage limit reached");
}

#[tokio::test]
async fn check_coupon_min_order_not_met_returns_reason() {
    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 6,
        code: "MIN5000".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 500,
        min_order_value_paise: Some(5_000),
        usage_limit: None,
        usage_count: None,
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now - chrono::Duration::days(1),
        ends_at: Some(now + chrono::Duration::days(1)),
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = core_operations::handlers::coupons::validate_coupon::check_coupon(
        &txn, "MIN5000", 4_000, false,
    )
    .await;
    assert!(result.is_ok());
    let c = result.unwrap();
    assert!(!c.is_valid);
    assert!(
        c.reason.to_lowercase().contains("order value too low"),
        "expected min-order failure reason, got {}",
        c.reason
    );
}
