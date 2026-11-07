//! Two-phase, explicitly opt-in real Razorpay Test Mode coverage.
//! No synthetic event is presented as evidence of a real provider payment.
use super::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Case {
    name: String,
    order: i64,
    user: i64,
    intent: i64,
    gateway: String,
}

fn sandbox_env() -> (Env, PathBuf) {
    assert_eq!(
        std::env::var("RUN_RAZORPAY_SANDBOX").as_deref(),
        Ok("1"),
        "real Test Mode calls require RUN_RAZORPAY_SANDBOX=1"
    );
    let key = std::env::var("SANDBOX_RAZORPAY_KEY_ID").expect("external test key required");
    assert!(key.starts_with("rzp_test_") && key.len() > "rzp_test_".len());
    let secret =
        std::env::var("SANDBOX_RAZORPAY_KEY_SECRET").expect("external test secret required");
    let webhook = std::env::var("SANDBOX_RAZORPAY_WEBHOOK_SECRET")
        .expect("dedicated test webhook secret required");
    assert!(!secret.trim().is_empty() && !webhook.trim().is_empty());
    assert!(webhook != secret && webhook != WEBHOOK_SECRET);
    assert!(
        repo_setting("RAZORPAY_WEBHOOK_SECRET").as_deref() != Some(webhook.as_str()),
        "sandbox must not reuse the deployed webhook secret"
    );
    let dir = PathBuf::from(
        std::env::var("RAZORPAY_SANDBOX_ARTIFACT_DIR")
            .expect("external artifact directory required"),
    );
    assert!(
        dir.is_absolute() && dir.is_dir(),
        "pre-create a private external artifact directory"
    );
    let dir = dir.canonicalize().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    assert!(
        !dir.starts_with(root),
        "keep manual payment artifacts outside the checkout/watchers"
    );
    let mut env = Env::new();
    env.set("RAZORPAY_KEY_ID", key);
    env.set("RAZORPAY_KEY_SECRET", secret);
    env.set("RAZORPAY_WEBHOOK_SECRET", webhook);
    env.set("RAZORPAY_API_BASE", "https://api.razorpay.com/v1");
    // This suite never books shipping. Prevent any unexpected provider call from
    // falling back to a deployed Shiprocket endpoint or credential.
    env.set("SHIPROCKET_API_BASE", "http://127.0.0.1:9");
    env.set("SHIPROCKET_EMAIL", "sandbox-disabled@example.invalid");
    env.set("SHIPROCKET_PASSWORD", "disabled");
    (env, dir)
}

#[tokio::test]
#[ignore = "REAL Razorpay Test Mode orders; explicit opt-in, isolated MySQL, external credentials"]
async fn prepare_test_orders() {
    let _lock = ENV_LOCK.lock().await;
    let (_env, dir) = sandbox_env();
    let db = isolated_db().await;
    let manifest = dir.join("orders.json");
    assert!(
        !manifest.exists(),
        "use a fresh artifact directory; never silently recreate provider orders"
    );
    let mut cases = Vec::new();
    for name in ["callback_and_capture", "declined", "webhook_only"] {
        let txn = db.begin().await.unwrap();
        let tag = format!("sandbox_{}", uuid::Uuid::new_v4().simple());
        let user = seed_user(&txn, &tag).await;
        ensure_order_status_row(&txn, "confirmed").await;
        let variant = seed_variant(&txn, &tag, 1000, 5).await;
        let (order, _) = seed_order_with_line(&txn, &tag, user, variant, 1, None).await;
        txn.commit().await.unwrap();
        // Same server-authoritative provider order resolution used by the gRPC
        // service: amount/currency come from the internal order, not the browser.
        let (gateway,amount,currency) = core_operations::handlers::payment_intents::create_payment_intent::resolve_server_created_razorpay_order(&db,order)
            .await.unwrap_or_else(|_| panic!("Test Mode order creation failed; inspect provider dashboard before retrying"));
        assert_eq!(amount, 1000);
        assert_eq!(currency, "INR");
        assert!(gateway.starts_with("order_"));
        let txn = db.begin().await.unwrap();
        let response = create_payment_intent(
            &txn,
            Request::new(CreatePaymentIntentRequest {
                order_id: order,
                user_id: user,
                amount_paise: amount,
                currency: Some(currency),
                razorpay_order_id: Some(gateway.clone()),
            }),
        )
        .await
        .unwrap()
        .into_inner();
        txn.commit().await.unwrap();
        let f = PaymentFixture {
            order,
            user,
            intent: response.items[0].intent_id,
            gateway: gateway.clone(),
            payment: String::new(),
        };
        assert_unpaid(&db, &f).await;
        cases.push(Case {
            name: name.into(),
            order,
            user,
            intent: f.intent,
            gateway,
        });
        // Persist each accepted operation immediately, so a later setup failure
        // cannot hide earlier real Test Mode orders. Contains synthetic IDs only.
        std::fs::write(&manifest, serde_json::to_vec_pretty(&cases).unwrap()).unwrap();
    }
}

#[tokio::test]
#[ignore = "manual Test Mode Checkout + authentic signed webhook artifacts required"]
async fn assert_manual_checkout_outcomes() {
    let _lock = ENV_LOCK.lock().await;
    let (_env, dir) = sandbox_env();
    let db = isolated_db().await;
    let cases: Vec<Case> =
        serde_json::from_slice(&std::fs::read(dir.join("orders.json")).unwrap()).unwrap();
    assert_eq!(
        cases.len(),
        3,
        "prepare phase must have completed all three orders"
    );
    let names: std::collections::BTreeSet<&str> = cases.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        ["callback_and_capture", "declined", "webhook_only"]
            .into_iter()
            .collect()
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();
    for case in cases {
        assert!(matches!(
            case.name.as_str(),
            "callback_and_capture" | "declined" | "webhook_only"
        ));
        let raw = std::fs::read_to_string(dir.join(format!("{}.body.json", case.name))).unwrap();
        let signature =
            std::fs::read_to_string(dir.join(format!("{}.signature.txt", case.name))).unwrap();
        let event_id =
            std::fs::read_to_string(dir.join(format!("{}.event-id.txt", case.name))).unwrap();
        let payload: Value = serde_json::from_str(&raw).unwrap();
        let entity = &payload["payload"]["payment"]["entity"];
        let payment = entity["id"]
            .as_str()
            .expect("payment event required")
            .to_owned();
        assert!(payment.starts_with("pay_"));
        assert_eq!(entity["order_id"].as_str(), Some(case.gateway.as_str()));
        assert_eq!(entity["amount"].as_i64(), Some(1000));
        assert_eq!(entity["currency"].as_str(), Some("INR"));
        let expected = if case.name == "declined" {
            "failed"
        } else {
            "captured"
        };
        assert_eq!(
            payload["event"].as_str(),
            Some(format!("payment.{expected}").as_str())
        );
        // Independent Test Mode API evidence prevents a locally manufactured
        // signed payload from masquerading as a successful sandbox interaction.
        let provider = client
            .get(format!("https://api.razorpay.com/v1/payments/{payment}"))
            .basic_auth(
                std::env::var("SANDBOX_RAZORPAY_KEY_ID").unwrap(),
                Some(std::env::var("SANDBOX_RAZORPAY_KEY_SECRET").unwrap()),
            )
            .send()
            .await
            .unwrap_or_else(|_| panic!("Test Mode payment lookup failed"));
        assert!(
            provider.status().is_success(),
            "Test Mode payment lookup rejected"
        );
        let provider: Value = provider.json().await.expect("Test Mode payment response");
        assert_eq!(provider["status"].as_str(), Some(expected));
        assert_eq!(provider["order_id"].as_str(), Some(case.gateway.as_str()));
        assert_eq!(provider["amount"].as_i64(), Some(1000));
        assert_eq!(provider["currency"].as_str(), Some("INR"));
        let f = PaymentFixture {
            order: case.order,
            user: case.user,
            intent: case.intent,
            gateway: case.gateway,
            payment,
        };
        assert_unpaid(&db, &f).await; // No application callback/worker has run for these fixtures.
        if case.name == "callback_and_capture" {
            let cb: Value =
                serde_json::from_slice(&std::fs::read(dir.join("callback.json")).unwrap()).unwrap();
            assert_eq!(cb["razorpay_order_id"].as_str(), Some(f.gateway.as_str()));
            assert_eq!(cb["razorpay_payment_id"].as_str(), Some(f.payment.as_str()));
            let txn = db.begin().await.unwrap();
            core_operations::handlers::payment_intents::verify_razorpay_payment(
                &txn,
                Request::new(VerifyRazorpayPaymentRequest {
                    order_id: f.order,
                    razorpay_order_id: f.gateway.clone(),
                    razorpay_payment_id: f.payment.clone(),
                    razorpay_signature: cb["razorpay_signature"].as_str().unwrap().into(),
                }),
            )
            .await
            .unwrap();
            txn.commit().await.unwrap();
        }
        // Preserve the EXACT raw body bytes, including whitespace, for HMAC verification.
        let req = IngestWebhookRequest {
            provider: "razorpay".into(),
            event_type: format!("payment.{expected}"),
            webhook_id: event_id.trim().into(),
            provider_event_id: Some(event_id.trim().into()),
            payload_json: raw,
            raw_signature: Some(signature.trim().into()),
            signature_verified: false,
        };
        assert!(!req.webhook_id.is_empty());
        // Validate authenticity even for payment.failed, whose business handler
        // currently does not change financial state.
        let secret = std::env::var("SANDBOX_RAZORPAY_WEBHOOK_SECRET").unwrap();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(req.payload_json.as_bytes());
        mac.verify_slice(&hex::decode(signature.trim()).expect("hex signature"))
            .expect("authentic test webhook required");
        for _ in 0..2 {
            let response = ingest(&db, req.clone()).await.unwrap();
            assert!(response.items.iter().all(|e| e.status == "processed"));
        }
        if case.name == "declined" {
            assert_unpaid(&db, &f).await;
        } else {
            assert_paid_once(&db, &f).await;
        }
    }
}
