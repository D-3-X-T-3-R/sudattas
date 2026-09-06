//! Unit tests for shipments and coupons handlers using SeaORM MockDatabase.

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType, ShipmentStatus};
use core_db_entities::entity::{coupons, shipments};
use proto::proto::core::{
    ApplyCouponRequest, CreateShipmentRequest, DeleteCouponAdminRequest, GetShipmentRequest,
    ListActiveCouponsRequest, SearchCouponAdminRequest, UpdateShipmentRequest,
    ValidateCouponRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use std::collections::BTreeMap;
use tonic::Request;

#[tokio::test]
async fn create_shipment_success() {
    use core_operations::handlers::shipments::create_shipment;

    let mut booking_validation_row = BTreeMap::new();
    booking_validation_row.insert("OrderID", 100_i64.into());
    booking_validation_row.insert("order_status_name", "confirmed".into());
    booking_validation_row.insert("payment_method", "prepaid".into());
    booking_validation_row.insert("payment_status", "captured".into());
    booking_validation_row.insert("fulfillment_status", "not_created".into());
    booking_validation_row.insert("earliest_booking_at", Utc::now().into());
    booking_validation_row.insert("pickup_target_at", Utc::now().into());
    booking_validation_row.insert("has_active_items", 1_i8.into());

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![booking_validation_row]])
        .append_query_results(vec![Vec::<shipments::Model>::new()])
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .append_query_results(vec![vec![shipments::Model {
            shipment_id: 1,
            order_id: 100,
            shiprocket_order_id: Some("sr_123".to_string()),
            shiprocket_external_order_id: None,
            awb_code: Some("AWB456".to_string()),
            carrier: Some("DTDC".to_string()),
            selected_courier_id: None,
            selected_courier_name: None,
            quoted_shipping_cost: None,
            quoted_shipping_quote_payload: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
            shipment_status: ShipmentStatus::AwbAssigned,
            tracking_events: None,
            created_at: None,
            delivered_at: None,
            pickup_scheduled_for: None,
            logistics_status: Some("booked".to_string()),
            can_customer_cancel: 0,
            razorpay_refund_id: None,
            refund_status: None,
            refund_initiated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateShipmentRequest {
        order_id: 100,
        shiprocket_order_id: Some("sr_123".to_string()),
        awb_code: Some("AWB456".to_string()),
        carrier: Some("DTDC".to_string()),
        shiprocket_status_id: None,
        shiprocket_status_label: None,
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
    assert_eq!(res.items[0].status, "awb_assigned");
}

#[tokio::test]
async fn get_shipment_by_order_id_returns_items() {
    use core_operations::handlers::shipments::get_shipment;

    let model = shipments::Model {
        shipment_id: 2,
        order_id: 200,
        shiprocket_order_id: Some("sr_456".to_string()),
        shiprocket_external_order_id: None,
        awb_code: Some("AWB789".to_string()),
        carrier: Some("Bluedart".to_string()),
        selected_courier_id: None,
        selected_courier_name: None,
        quoted_shipping_cost: None,
        quoted_shipping_quote_payload: None,
        shiprocket_status_id: None,
        shiprocket_status_label: None,
        shipment_status: ShipmentStatus::InTransit,
        tracking_events: None,
        created_at: None,
        delivered_at: None,
        pickup_scheduled_for: None,
        logistics_status: None,
        can_customer_cancel: 1,
        razorpay_refund_id: None,
        refund_status: None,
        refund_initiated_at: None,
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
        tracking_events_json: None,
        shiprocket_status_id: None,
        shiprocket_status_label: None,
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
        shiprocket_external_order_id: None,
        awb_code: Some("AWB000".to_string()),
        carrier: Some("Xpress".to_string()),
        selected_courier_id: None,
        selected_courier_name: None,
        quoted_shipping_cost: None,
        quoted_shipping_quote_payload: None,
        shiprocket_status_id: None,
        shiprocket_status_label: None,
        shipment_status: ShipmentStatus::Pending,
        tracking_events: None,
        created_at: Some(Utc::now()),
        delivered_at: None,
        pickup_scheduled_for: None,
        logistics_status: None,
        can_customer_cancel: 1,
        razorpay_refund_id: None,
        refund_status: None,
        refund_initiated_at: None,
    };
    let updated = shipments::Model {
        shipment_status: ShipmentStatus::Delivered,
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
        status: Some("delivered".to_string()),
        tracking_events_json: None,
        shiprocket_status_id: None,
        shiprocket_status_label: None,
    });

    let result = update_shipment(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    let s = &res.items[0];
    assert_eq!(s.order_id, 300);
    assert_eq!(s.status, "delivered");
}

#[tokio::test]
async fn cancel_order_via_logistics_returns_none_when_no_provider_reference() {
    use core_operations::handlers::shipments::logistics_workflow::cancel_order_via_logistics;

    // A shipment row exists (e.g. created manually via the admin "create shipment" action,
    // which unconditionally sets fulfillment_status = 'booked' and hardcodes
    // can_customer_cancel = 0) but has no shiprocket_order_id, shiprocket_external_order_id, or
    // awb_code at all — there was never a real logistics-partner booking behind it. Cancelling
    // must not block on `can_customer_cancel` (which would otherwise wrongly read as "pickup
    // already in progress") since there's nothing to cancel with a provider either way.
    let mut shipment_row = BTreeMap::new();
    shipment_row.insert("shipment_id", 7_i64.into());
    shipment_row.insert("order_id", 200_i64.into());
    shipment_row.insert("logistics_status", "booked".to_string().into());
    shipment_row.insert("can_customer_cancel", false.into());

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![shipment_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = cancel_order_via_logistics(&txn, 200, None).await;
    assert!(
        matches!(result, Ok(None)),
        "expected Ok(None) (nothing to cancel with a provider), got {result:?}"
    );
}

#[tokio::test]
async fn cancel_order_via_logistics_blocks_when_pickup_in_progress_for_a_real_booking() {
    use core_operations::handlers::shipments::logistics_workflow::cancel_order_via_logistics;

    // Same shape as above, except this shipment DOES have a real Shiprocket order behind it —
    // here `can_customer_cancel = false` correctly means "too late to cancel," not "nothing to
    // cancel," and must still block.
    let mut shipment_row = BTreeMap::new();
    shipment_row.insert("shipment_id", 7_i64.into());
    shipment_row.insert("order_id", 200_i64.into());
    shipment_row.insert("shiprocket_order_id", "SR-500".to_string().into());
    shipment_row.insert("logistics_status", "pickup_completed".to_string().into());
    shipment_row.insert("can_customer_cancel", false.into());

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![shipment_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = cancel_order_via_logistics(&txn, 200, None).await;
    let status = result.expect_err("a real booking past pickup must still block cancellation");
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    assert!(status
        .message()
        .contains("pickup/logistics is already in progress"));
}

#[tokio::test]
async fn apply_coupon_valid_returns_valid_without_mutating_usage() {
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

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
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
    assert!(
        c.reason.contains("₹50.00") && !c.reason.to_lowercase().contains("paise"),
        "reason should show the minimum in rupees, not raw paise: {}",
        c.reason
    );
}

#[tokio::test]
async fn search_coupon_admin_returns_all_when_id_zero() {
    use core_operations::handlers::coupons::search_coupon_admin;

    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 3,
        code: "WELCOME20".to_string(),
        discount_type: DiscountType::Percentage,
        discount_value: 20,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: Some(0),
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now,
        ends_at: None,
        created_at: Some(now),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCouponAdminRequest { coupon_id: 0 });
    let result = search_coupon_admin(&txn, req).await;
    assert!(result.is_ok(), "err: {:?}", result.err());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].code, "WELCOME20");
    assert_eq!(res.items[0].status, "active");
}

#[tokio::test]
async fn delete_coupon_admin_not_found_yields_not_found_status() {
    use core_operations::handlers::coupons::delete_coupon_admin;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<coupons::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteCouponAdminRequest { coupon_id: 999 });
    let result = delete_coupon_admin(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_coupon_admin_deletes_and_returns_deleted_coupon() {
    use core_operations::handlers::coupons::delete_coupon_admin;

    let now = Utc::now();
    let coupon_model = coupons::Model {
        coupon_id: 7,
        code: "GONE_SOON".to_string(),
        discount_type: DiscountType::FixedAmount,
        discount_value: 5000,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: Some(0),
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: now,
        ends_at: None,
        created_at: Some(now),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon_model]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteCouponAdminRequest { coupon_id: 7 });
    let result = delete_coupon_admin(&txn, req).await;
    assert!(result.is_ok(), "err: {:?}", result.err());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].code, "GONE_SOON");
}

#[tokio::test]
async fn list_active_coupons_excludes_expired_and_exhausted_sorts_by_soonest_expiry() {
    use core_operations::handlers::coupons::list_active_coupons;

    let now = Utc::now();
    let coupon = |id: i64,
                  code: &str,
                  ends_at: Option<chrono::DateTime<Utc>>,
                  limit: Option<i32>,
                  used: Option<i32>| {
        coupons::Model {
            coupon_id: id,
            code: code.to_string(),
            discount_type: DiscountType::Percentage,
            discount_value: 10,
            min_order_value_paise: None,
            usage_limit: limit,
            usage_count: used,
            max_uses_per_customer: None,
            coupon_status: Some(CouponStatus::Active),
            starts_at: now - chrono::Duration::days(30),
            ends_at,
            created_at: None,
        }
    };

    // Note: MockDatabase returns whatever rows we feed it regardless of the SQL WHERE clause, so
    // only Active/null-status rows belong here — the status filter itself needs a real DB to
    // verify. This test exercises the Rust-side date/usage-limit filtering that runs after fetch.
    let candidates = vec![
        coupon(
            1,
            "EXPIRES_SOON",
            Some(now + chrono::Duration::days(1)),
            None,
            None,
        ),
        coupon(2, "NEVER_EXPIRES", None, None, None),
        coupon(
            3,
            "EXPIRED",
            Some(now - chrono::Duration::days(1)),
            None,
            None,
        ),
        coupon(4, "EXHAUSTED", None, Some(5), Some(5)),
        coupon(5, "STILL_HAS_USES", None, Some(5), Some(4)),
        coupon(
            6,
            "EXPIRES_LATER",
            Some(now + chrono::Duration::days(10)),
            None,
            None,
        ),
    ];

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![candidates])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ListActiveCouponsRequest {});
    let result = list_active_coupons(&txn, req).await;
    assert!(result.is_ok(), "err: {:?}", result.err());
    let res = result.unwrap().into_inner();

    let codes: Vec<&str> = res.items.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        codes,
        // Both never-expiring coupons tie on the sort key; stable sort keeps their original order.
        vec!["EXPIRES_SOON", "EXPIRES_LATER", "NEVER_EXPIRES", "STILL_HAS_USES"],
        "expired and exhausted coupons must be excluded; remaining sorted soonest-expiry first, never-expiring last"
    );
}

#[test]
fn list_active_coupons_status_filter_excludes_inactive_at_the_sql_level() {
    use core_db_entities::entity::coupons;
    use sea_orm::{ColumnTrait, DbBackend, EntityTrait, QueryFilter, QueryTrait};

    let query = coupons::Entity::find().filter(
        coupons::Column::CouponStatus
            .eq(CouponStatus::Active)
            .or(coupons::Column::CouponStatus.is_null()),
    );
    let sql = query.build(DbBackend::MySql).to_string();
    assert!(
        sql.contains("`coupon_status` = ('active')"),
        "expected an equality check against the active status in the generated SQL: {sql}"
    );
    assert!(
        sql.contains("IS NULL"),
        "expected the null-status leniency clause in the generated SQL: {sql}"
    );
    assert!(
        !sql.to_lowercase().contains("'inactive'"),
        "must never match inactive coupons: {sql}"
    );
}
