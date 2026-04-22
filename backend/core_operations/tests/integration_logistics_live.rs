//! Opt-in live Shiprocket + Razorpay test-mode verification.
//! These tests never run by default and always attempt cleanup cancellation.

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{AuthProvider, Status as PaymentIntentStatus};
use core_db_entities::entity::{
    inventory, order_status, payment_intents, product_categories, product_variants, products,
    shipping_addresses, user_roles, users,
};
use core_operations::handlers::orders::delete_order;
use core_operations::handlers::payment_intents::verify_razorpay_payment;
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url_optional;
use proto::proto::core::{
    CreateCartItemRequest, DeleteOrderRequest, PlaceOrderRequest, VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Request;

static UNIQUE_COUNTER: AtomicI64 = AtomicI64::new(0);

struct LiveContext {
    db_url: String,
}

struct LiveCheckoutPayment {
    payment_id: String,
    order_id: String,
    signature: String,
}

fn mask_gateway_id(id: &str) -> String {
    let len = id.len();
    if len <= 12 {
        return format!("(len={len})");
    }
    format!("{}…{} (len={len})", &id[..8], &id[len.saturating_sub(4)..])
}

fn mask_signature_hex(sig: &str) -> String {
    let len = sig.len();
    match len {
        0 => "(absent)".to_string(),
        1..=16 => format!("(present, len={len}, value masked)"),
        _ => format!(
            "prefix={}…suffix={} (len={len})",
            &sig[..6.min(len)],
            &sig[len.saturating_sub(4)..]
        ),
    }
}

fn load_live_env_from_repo() {
    let candidates = [
        PathBuf::from("..").join(".env"),
        PathBuf::from(".env"),
        PathBuf::from("backend").join(".env"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            let _ = dotenvy::from_path(candidate);
            break;
        }
    }
}

fn live_context() -> Result<LiveContext, String> {
    load_live_env_from_repo();
    let flag_value = std::env::var("RUN_LIVE_LOGISTICS_TESTS").ok();
    if flag_value.as_deref() != Some("1") {
        let current = flag_value.unwrap_or_else(|| "<unset>".to_string());
        return Err(format!(
            "RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {current})"
        ));
    }
    for key in [
        "SHIPROCKET_EMAIL",
        "SHIPROCKET_PASSWORD",
        "SHIPROCKET_PICKUP_LOCATION",
        "RAZORPAY_KEY_ID",
        "RAZORPAY_KEY_SECRET",
    ] {
        let value = std::env::var(key).ok().filter(|v| !v.trim().is_empty());
        if value.is_none() {
            return Err(format!("missing required env: {key}"));
        }
    }
    let db_url = test_db_url_optional().ok_or_else(|| {
        "TEST_DATABASE_URL (or DATABASE_URL fallback) is not configured".to_string()
    })?;
    Ok(LiveContext { db_url })
}

fn print_live_skip_message(reason: &str) {
    eprintln!(
        "skipping live logistics test: {reason}. To enable, set RUN_LIVE_LOGISTICS_TESTS=1 and provide required live credentials."
    );
}

fn unique_tag() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::SeqCst) as u128;
    let mixed = now.saturating_mul(100).saturating_add(counter);
    (mixed % (i64::MAX as u128)) as i64
}

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Some(existing) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(txn)
        .await
        .expect("query status")
    {
        return existing.status_id;
    }
    order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert status")
    .status_id
}

async fn seed_checkout_user(txn: &sea_orm::DatabaseTransaction, tag: i64) -> (i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;
    let _ = ensure_order_status(txn, "confirmed").await;
    let _ = ensure_order_status(txn, "cancel_pending_logistics").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_live_logistics_role_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert role");

    let phone = format!("+91{:010}", (tag % 10_000_000_000).abs());
    let user = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(format!("itest_live_logistics_{tag}")),
        email: ActiveValue::Set(format!("itest_live_logistics+{tag}@example.com")),
        auth_provider: ActiveValue::Set(AuthProvider::Email),
        password_hash: ActiveValue::Set(Some("itest-hash".to_string())),
        google_sub: ActiveValue::Set(None),
        full_name: ActiveValue::Set(None),
        address: ActiveValue::Set(None),
        phone: ActiveValue::Set(Some(phone.clone())),
        create_date: ActiveValue::Set(Utc::now()),
        role_id: ActiveValue::Set(Some(role.role_id)),
        email_verified: ActiveValue::NotSet,
        email_verified_at: ActiveValue::NotSet,
        user_status_id: ActiveValue::NotSet,
        last_login_at: ActiveValue::NotSet,
        marketing_opt_out: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    }
    .insert(txn)
    .await
    .expect("insert user")
    .user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user)),
        recipient_name: ActiveValue::Set(Some("Live Logistics Test".to_string())),
        phone_number: ActiveValue::Set(Some(phone)),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("MG Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert shipping");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_live_logistics_cat_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert category");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Live Logistics Saree".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(2_500),
        category_id: ActiveValue::Set(category.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert product");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
    .await
    .expect("insert variant");

    inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(3)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert inventory");

    let _ = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create cart");

    (user, shipping.shipping_address_id)
}

async fn place_and_pay_live_order(db: &DatabaseConnection, tag: i64) -> Result<(i64, i64), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let (user_id, shipping_address_id) = seed_checkout_user(&txn, tag).await;
    let cart_item = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(proto::proto::core::GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .map_err(|e| e.to_string())?
    .into_inner()
    .items[0]
        .clone();

    let order = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_item.cart_id],
            payment_mode: None,
        }),
    )
    .await
    .map_err(|e| e.to_string())?
    .into_inner()
    .items[0]
        .clone();

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent".to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    let payment = complete_live_checkout_payment(&intent.razorpay_order_id, tag)?;
    eprintln!(
        "[live-checkout-stage] rust_pre_verify internal_order_id={} razorpay_order_id={} razorpay_payment_id={} razorpay_signature={}",
        order.order_id,
        mask_gateway_id(&payment.order_id),
        mask_gateway_id(&payment.payment_id),
        mask_signature_hex(&payment.signature),
    );
    let verify_txn = db.begin().await.map_err(|e| e.to_string())?;
    let verify_resp = verify_razorpay_payment(
        &verify_txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: order.order_id,
            razorpay_order_id: payment.order_id.clone(),
            razorpay_payment_id: payment.payment_id.clone(),
            razorpay_signature: payment.signature.clone(),
        }),
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[live-checkout-stage] verify_razorpay_payment_transport_err code={:?} message={}",
            e.code(),
            e.message()
        );
        e.to_string()
    })?;
    let verify_inner = verify_resp.into_inner();
    eprintln!(
        "[live-checkout-stage] verify_razorpay_payment_result verified={} payment_intent_present={}",
        verify_inner.verified,
        verify_inner.payment_intent.is_some()
    );
    if !verify_inner.verified {
        return Err(
            "verify_razorpay_payment returned verified=false: classify as signature_mismatch_or_wrong_RAZORPAY_KEY_SECRET_or_order_id_mismatch"
                .to_string(),
        );
    }
    verify_txn.commit().await.map_err(|e| e.to_string())?;

    let intent_after = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent after verify".to_string())?;
    eprintln!(
        "[live-verify] capture path=client_verify_signature razorpay_order_id={} razorpay_payment_id={} internal_order_id={} payment_intent_status={:?}",
        intent_after.razorpay_order_id,
        intent_after.razorpay_payment_id.as_deref().unwrap_or(""),
        order.order_id,
        intent_after.status
    );

    Ok((order.order_id, user_id))
}

fn complete_live_checkout_payment(
    razorpay_order_id: &str,
    tag: i64,
) -> Result<LiveCheckoutPayment, String> {
    let key_id = std::env::var("RAZORPAY_KEY_ID").map_err(|e| e.to_string())?;
    let contact = format!("9{:09}", tag.rem_euclid(1_000_000_000));
    let email = format!("itest_live_payment+{tag}@example.com");
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("sudattas_live_checkout_{tag}.cjs"));
    let output_path = temp_dir.join(format!("sudattas_live_checkout_{tag}.json"));
    let escaped_output = output_path.to_string_lossy().replace('\\', "\\\\");
    let escaped_key = key_id.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_order = razorpay_order_id.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_contact = contact.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_email = email.replace('\\', "\\\\").replace('\'', "\\'");

    let script = format!(
        r#"const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const {{ chromium }} = require(path.resolve(process.cwd(), '../../frontend/node_modules/@playwright/test'));

const outputPath = '{escaped_output}';
const keyId = '{escaped_key}';
const razorpayOrderId = '{escaped_order}';
const contact = '{escaped_contact}';
const email = '{escaped_email}';

function stageLog(message) {{
  console.error('[live-checkout-stage] ' + message);
}}

function maskSig(sig) {{
  if (!sig) return '(absent)';
  if (sig.length <= 12) return '(present len=' + sig.length + ')';
  return sig.slice(0, 4) + '…' + sig.slice(-4) + ' (len=' + sig.length + ')';
}}

function logPayloadPresence(label, p) {{
  stageLog(label + ' payment_id=' + (p.razorpay_payment_id ? 'present' : 'MISSING') +
    ' order_id=' + (p.razorpay_order_id ? 'present' : 'MISSING') +
    ' signature=' + (p.razorpay_signature ? ('present ' + maskSig(p.razorpay_signature)) : 'MISSING'));
}}

function parseCallbackPayload(method, reqUrl, bodyText, contentType) {{
  if (method === 'GET') {{
    const u = new URL(reqUrl, 'http://127.0.0.1');
    return {{
      razorpay_payment_id: u.searchParams.get('razorpay_payment_id'),
      razorpay_order_id: u.searchParams.get('razorpay_order_id'),
      razorpay_signature: u.searchParams.get('razorpay_signature'),
    }};
  }}
  const ct = (contentType || '').toLowerCase();
  const raw = bodyText || '';
  if (ct.includes('application/json') && raw.trim().startsWith('{{')) {{
    try {{
      const j = JSON.parse(raw);
      return {{
        razorpay_payment_id: j.razorpay_payment_id || null,
        razorpay_order_id: j.razorpay_order_id || null,
        razorpay_signature: j.razorpay_signature || null,
      }};
    }} catch (e) {{
      stageLog('CALLBACK_POST_JSON_PARSE_FAILED ' + String(e?.message || e));
    }}
  }}
  const params = new URLSearchParams(raw);
  return {{
    razorpay_payment_id: params.get('razorpay_payment_id'),
    razorpay_order_id: params.get('razorpay_order_id'),
    razorpay_signature: params.get('razorpay_signature'),
  }};
}}

function html(port) {{
  return `<!doctype html>
  <html>
    <head><meta charset="utf-8"><title>Sudattas Live Razorpay Test</title></head>
    <body>
      <button id="pay">Pay</button>
      <script src="https://checkout.razorpay.com/v1/checkout.js"></script>
      <script>
        const openCheckout = () => {{
          const rzp = new Razorpay({{
            key: '{escaped_key}',
            order_id: '{escaped_order}',
            callback_url: 'http://127.0.0.1:${{port}}/callback',
            redirect: true,
            name: 'Sudattas Live Logistics Test',
            description: 'Live logistics refund verification',
            method: {{
              card: true,
              netbanking: false,
              wallet: false,
              emi: false,
              upi: false,
            }},
            prefill: {{
              contact: '{escaped_contact}',
              email: '{escaped_email}',
            }},
            notes: {{
              source: 'integration_logistics_live',
            }},
            theme: {{
              color: '#111827',
            }},
          }});
          rzp.open();
        }};
        document.getElementById('pay').addEventListener('click', openCheckout);
      </script>
    </body>
  </html>`;
}}

async function getCheckoutFrame(page, timeoutMs = 30000) {{
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {{
    if (/razorpay\.com/i.test(page.url())) {{
      stageLog('CHECKOUT_razorpay_host_detected main_page_url=' + page.url().substring(0, 120));
      return page.mainFrame();
    }}
    const popup = page.context().pages().find((candidate) => candidate !== page && /razorpay\.com/i.test(candidate.url()));
    if (popup) {{
      stageLog('CHECKOUT_razorpay_host_detected popup_url=' + popup.url().substring(0, 120));
      return popup.mainFrame();
    }}
    const frame = page.frames().find((candidate) => /razorpay\.com/i.test(candidate.url()));
    if (frame) {{
      stageLog('CHECKOUT_razorpay_host_detected frame_url=' + frame.url().substring(0, 120));
      return frame;
    }}
    await page.waitForTimeout(250);
  }}
  stageLog('CHECKOUT_FAILED razorpay_frame_not_found within_ms=' + timeoutMs);
  throw new Error('Razorpay checkout frame or popup did not appear');
}}

function checkoutFrames(page, primaryFrame) {{
  const seen = new Set();
  const frames = [primaryFrame, ...page.frames()].filter(Boolean);
  return frames.filter((frame) => {{
    const key = frame.url() + ':' + frame.name();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  }});
}}

async function clickIfVisible(page, primaryFrame, selectors) {{
  for (const frame of checkoutFrames(page, primaryFrame)) {{
    for (const selector of selectors) {{
      const locator = frame.locator(selector).first();
      if (await locator.count()) {{
        try {{
          await locator.click({{ timeout: 2000 }});
          return true;
        }} catch (_err) {{
        }}
      }}
    }}
  }}
  return false;
}}

async function fillFirstVisible(page, primaryFrame, selectors, value) {{
  for (const frame of checkoutFrames(page, primaryFrame)) {{
    for (const selector of selectors) {{
      const locator = frame.locator(selector).first();
      if (await locator.count()) {{
        try {{
          await locator.click({{ timeout: 2000 }});
          await locator.press('Control+A').catch(() => {{}});
          await locator.press('Meta+A').catch(() => {{}});
          await locator.press('Backspace').catch(() => {{}});
          await locator.fill('', {{ timeout: 1000 }}).catch(() => {{}});
          await locator.pressSequentially(value, {{ delay: 35 }});
          return true;
        }} catch (_err) {{
        }}
      }}
    }}
  }}
  return false;
}}

async function dumpFrameHints(page, primaryFrame) {{
  for (const [index, frame] of checkoutFrames(page, primaryFrame).entries()) {{
    try {{
      const inputs = await frame.locator('input').evaluateAll((nodes) =>
        nodes.slice(0, 12).map((node) => ({{
          type: node.getAttribute('type'),
          name: node.getAttribute('name'),
          placeholder: node.getAttribute('placeholder'),
          autocomplete: node.getAttribute('autocomplete'),
          inputmode: node.getAttribute('inputmode'),
          id: node.getAttribute('id'),
        }}))
      );
      const buttons = await frame.locator('button').evaluateAll((nodes) =>
        nodes.slice(0, 12).map((node) => (node.textContent || '').trim()).filter(Boolean)
      );
      console.error(`[frame-hints:${{index}}] url=${{frame.url()}} inputs=${{JSON.stringify(inputs)}} buttons=${{JSON.stringify(buttons)}}`);
    }} catch (err) {{
      console.error(`[frame-hints:${{index}}] failed: ${{err?.message || err}}`);
    }}
  }}
}}

async function completeTestModePayment(page) {{
  stageLog('UI_CLICK local_pay_button');
  await page.getByRole('button', {{ name: 'Pay' }}).click();
  stageLog('UI_WAIT razorpay_checkout_frame');
  const frame = await getCheckoutFrame(page);
  stageLog('UI_OK razorpay_checkout_opened');
  const waitForCardStage = async () => {{
    const started = Date.now();
    while (Date.now() - started < 30000) {{
      const cardReady = await clickIfVisible(page, frame, [
        'text=/Card/i',
        '[data-method=\"card\"]',
        'button:has-text(\"Card\")',
      ]);
      if (cardReady) stageLog('UI_CLICK card_method_tab_or_label');
      const hasCardInput = (await fillFirstVisible(page, frame, [
        'input[name=\"card.number\"]',
        'input[name=\"card[number]\"]',
        'input[autocomplete=\"cc-number\"]',
      ], '6527658900001005'));
      if (hasCardInput) {{
        stageLog('UI_OK card_number_field_located_and_filled_probe');
        return true;
      }}

      await fillFirstVisible(page, frame, [
        'input[name=\"contact\"]',
        'input[placeholder*=\"Mobile\"]',
      ], contact);
      await clickIfVisible(page, frame, [
        'button:has-text(\"Using as\")',
        'button:has-text(\"Continue\")',
        'text=/Using as \\+91/i',
      ]);
      if (cardReady) {{
        await page.waitForTimeout(500);
      }} else {{
        await page.waitForTimeout(1500);
      }}
    }}
    return false;
  }};

  const cardStageReady = await waitForCardStage();
  if (!cardStageReady) {{
    stageLog('UI_FAIL could_not_reach_card_entry');
    await dumpFrameHints(page, frame);
    throw new Error('Unable to advance Razorpay checkout to card entry');
  }}

  stageLog('UI_FILL card_number_final');
  const cardFilled = await fillFirstVisible(page, frame, [
    'input[name=\"card.number\"]',
    'input[name=\"card[number]\"]',
    'input[autocomplete=\"cc-number\"]',
  ], '6527658900001005');
  if (!cardFilled) {{
    stageLog('UI_FAIL card_number_final_fill');
    await dumpFrameHints(page, frame);
    throw new Error('Unable to fill test card number');
  }}
  stageLog('UI_OK card_number_field_located');

  stageLog('UI_FILL card_expiry');
  const expiryFilled = await fillFirstVisible(page, frame, [
    'input[name=\"card.expiry\"]',
    'input[name=\"card[expiry]\"]',
    'input[autocomplete=\"cc-exp\"]',
    'input[placeholder*=\"MM\"]',
  ], '12/33');
  if (!expiryFilled) {{
    stageLog('UI_FAIL card_expiry_field');
    throw new Error('Unable to fill expiry');
  }}
  stageLog('UI_OK card_expiry_field_located');

  stageLog('UI_FILL card_cvv');
  const cvvFilled = await fillFirstVisible(page, frame, [
    'input[name=\"card.cvv\"]',
    'input[name=\"card[cvv]\"]',
    'input[autocomplete=\"cc-csc\"]',
    'input[type=\"password\"]',
  ], '000');
  if (!cvvFilled) {{
    stageLog('UI_FAIL card_cvv_field');
    throw new Error('Unable to fill cvv');
  }}
  stageLog('UI_OK card_cvv_field_located');

  await fillFirstVisible(page, frame, [
    'input[name=\"card[name]\"]',
    'input[autocomplete=\"cc-name\"]',
    'input[placeholder*=\"name\"]',
  ], 'Live Logistics Test');

  stageLog('UI_CLICK hosted_pay_or_continue');
  const payClicked = await clickIfVisible(page, frame, [
    'button:has-text(\"Pay\")',
    'button:has-text(\"Continue\")',
    'button[type=\"submit\"]',
  ]);
  stageLog('UI_RESULT hosted_pay_click=' + (payClicked ? 'true' : 'false'));

  const bankPage = page;
  await bankPage.waitForLoadState('domcontentloaded', {{ timeout: 30000 }}).catch(() => {{}});
  stageLog('UI_WAIT possible_3ds_or_bank_sim url=' + bankPage.url().substring(0, 160));
  const okBank = await clickIfVisible(bankPage, bankPage.mainFrame(), [
    'button:has-text(\"Success\")',
    'input[value=\"Success\"]',
    'text=/Success/i',
    'text=/Authorize/i',
  ]);
  stageLog('UI_RESULT test_bank_success_click=' + (okBank ? 'true' : 'false'));
  stageLog('UI_DONE completeTestModePayment_script_steps_finished');
}}

(async () => {{
  let browser;
  let server;
  let rejectCallback;
  const CALLBACK_MS = 180000;
  try {{
    const callback = new Promise((resolve, reject) => {{
      rejectCallback = reject;
      const timer = setTimeout(() => {{
        stageLog('CALLBACK_TIMEOUT no_callback_request_within_ms=' + CALLBACK_MS);
        reject(new Error('callback_timeout'));
      }}, CALLBACK_MS);
      const finish = (payload) => {{
        clearTimeout(timer);
        resolve(payload);
      }};

      server = http.createServer((req, res) => {{
        const rawUrl = req.url || '';
        const pathOnly = rawUrl.split('?')[0];
        if (req.method === 'GET' && pathOnly === '/') {{
          res.writeHead(200, {{ 'content-type': 'text/html; charset=utf-8' }});
          res.end(html(server.address().port));
          return;
        }}
        if (pathOnly === '/callback') {{
          stageLog('CALLBACK_SERVER_HIT method=' + req.method + ' path=' + pathOnly + ' raw_url_len=' + rawUrl.length);
          if (req.method === 'GET') {{
            const payload = parseCallbackPayload('GET', rawUrl, '', '');
            logPayloadPresence('CALLBACK_PARSED_GET', payload);
            fs.writeFileSync(outputPath, JSON.stringify(payload), 'utf8');
            res.writeHead(200, {{ 'content-type': 'text/html; charset=utf-8' }});
            res.end('<html><body>Payment captured</body></html>');
            finish(payload);
            return;
          }}
          if (req.method === 'POST') {{
            let body = '';
            req.on('data', (chunk) => {{ body += chunk.toString('utf8'); }});
            req.on('end', () => {{
              const ct = req.headers['content-type'] || '';
              stageLog('CALLBACK_POST body_len=' + body.length + ' content_type=' + ct.substring(0, 80));
              const payload = parseCallbackPayload('POST', rawUrl, body, ct);
              logPayloadPresence('CALLBACK_PARSED_POST', payload);
              fs.writeFileSync(outputPath, JSON.stringify(payload), 'utf8');
              res.writeHead(200, {{ 'content-type': 'text/html; charset=utf-8' }});
              res.end('<html><body>Payment captured</body></html>');
              finish(payload);
            }});
            req.on('error', rejectCallback);
            return;
          }}
          stageLog('CALLBACK_UNSUPPORTED_METHOD method=' + req.method);
          res.writeHead(405, {{ 'content-type': 'text/plain; charset=utf-8' }});
          res.end('method not allowed for /callback');
          return;
        }}
        stageLog('HTTP_UNHANDLED method=' + req.method + ' url_prefix=' + rawUrl.substring(0, 160));
        res.writeHead(404);
        res.end('not found');
      }});
      server.on('error', rejectCallback);
      server.listen(0, '127.0.0.1');
    }});

    await new Promise((resolve) => server.once('listening', resolve));
    stageLog('CALLBACK_SERVER_LISTENING port=' + server.address().port);
    browser = await chromium.launch({{
      headless: false,
      args: ['--disable-popup-blocking'],
    }});
    const page = await browser.newPage();
    page.on('console', (msg) => console.error(`[page-console] ${{msg.type()}} ${{msg.text()}}`));
    page.on('pageerror', (err) => console.error(`[page-error] ${{err?.stack || err}}`));
    page.on('requestfailed', (req) => console.error(`[request-failed] ${{req.url()}} :: ${{req.failure()?.errorText || 'unknown'}}`));
    page.on('framenavigated', (f) => {{
      try {{
        const u = f.url();
        if (u.includes('127.0.0.1') && u.includes('/callback')) {{
          stageLog('BROWSER_NAV_TO_CALLBACK url=' + u.substring(0, 260));
        }}
      }} catch (_e) {{}}
    }});
    await page.goto(`http://127.0.0.1:${{server.address().port}}/`, {{ waitUntil: 'domcontentloaded' }});
    stageLog('RUN_starting_playwright_payment_flow');
    await completeTestModePayment(page);
    stageLog('RUN_waiting_for_callback_promise');
    const payload = await callback;
    stageLog('RUN_callback_promise_resolved');
    if (!payload.razorpay_payment_id || !payload.razorpay_order_id || !payload.razorpay_signature) {{
      logPayloadPresence('CALLBACK_VALIDATION_FAIL', payload);
      throw new Error('Missing Razorpay callback payload fields');
    }}
    logPayloadPresence('CALLBACK_VALIDATION_OK', payload);
  }} catch (error) {{
    console.error(String(error?.stack || error));
    process.exitCode = 1;
  }} finally {{
    if (browser) {{
      await browser.close().catch(() => {{}});
    }}
    if (server) {{
      await new Promise((resolve) => server.close(resolve));
    }}
  }}
}})();
"#
    );

    fs::write(&script_path, script).map_err(|e| e.to_string())?;
    eprintln!(
        "[live-checkout-stage] rust_spawn_node_checkout tag={tag} (stderr lines prefixed [live-checkout-stage] trace UI + callback)"
    );
    let output = Command::new("node")
        .arg(&script_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .map_err(|e| format!("failed to launch live checkout helper: {e}"))?;
    let _ = fs::remove_file(&script_path);
    if !output.status.success() {
        let _ = fs::remove_file(&output_path);
        return Err(format!(
            "live checkout helper failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let payload = fs::read_to_string(&output_path)
        .map_err(|e| format!("failed to read live checkout payload: {e}"))?;
    let _ = fs::remove_file(&output_path);
    let json: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|e| format!("invalid live checkout payload: {e}"))?;
    let payment_id = json
        .get("razorpay_payment_id")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "missing razorpay_payment_id from live checkout payload".to_string())?;
    let order_id = json
        .get("razorpay_order_id")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "missing razorpay_order_id from live checkout payload".to_string())?;
    let signature = json
        .get("razorpay_signature")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "missing razorpay_signature from live checkout payload".to_string())?;

    Ok(LiveCheckoutPayment {
        payment_id: payment_id.to_string(),
        order_id: order_id.to_string(),
        signature: signature.to_string(),
    })
}

async fn shipment_meta(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<sea_orm::QueryResult, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT shiprocket_order_id, shiprocket_external_order_id, awb_code, pickup_scheduled_for,
                      logistics_status, razorpay_refund_id, refund_status
               FROM Shipments WHERE order_id = ? ORDER BY shipment_id DESC LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing shipment row".to_string())?;
    txn.rollback().await.ok();
    Ok(row)
}

async fn payment_intent_meta(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<payment_intents::Model, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent".to_string())?;
    txn.rollback().await.ok();
    Ok(intent)
}

async fn inventory_quantity(db: &DatabaseConnection, order_id: i64) -> Result<i64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT i.quantity_available
               FROM Inventory i
               JOIN OrderDetails od ON od.VariantID = i.VariantID
               WHERE od.OrderID = ?
               ORDER BY i.InventoryID DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing inventory row".to_string())?;
    let quantity = row
        .try_get::<i32>("", "quantity_available")
        .map_err(|e| e.to_string())? as i64;
    txn.rollback().await.ok();
    Ok(quantity)
}

async fn order_status_name(db: &DatabaseConnection, order_id: i64) -> Result<String, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT s.StatusName
               FROM Orders o
               JOIN OrderStatus s ON s.StatusID = o.StatusID
               WHERE o.OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing order status".to_string())?;
    let status = row
        .try_get::<String>("", "StatusName")
        .map_err(|e| e.to_string())?;
    txn.rollback().await.ok();
    Ok(status)
}

async fn cleanup_live_order(
    db: &DatabaseConnection,
    order_id: i64,
    user_id: i64,
) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    delete_order(
        &txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tokio::test]
#[ignore = "opt-in live logistics verification; creates a real Shiprocket order and cancels it before exit"]
async fn live_payment_success_auto_books_shiprocket_and_cleans_up() {
    let ctx = match live_context() {
        Ok(ctx) => ctx,
        Err(reason) => {
            print_live_skip_message(&reason);
            return;
        }
    };
    let db = Database::connect(&ctx.db_url).await.expect("connect");
    let tag = unique_tag();

    let mut cleanup: Option<(i64, i64)> = None;
    let outcome: Result<(), String> = async {
        let (order_id, user_id) = place_and_pay_live_order(&db, tag).await?;
        cleanup = Some((order_id, user_id));

        let shipment = shipment_meta(&db, order_id).await?;
        let intent = payment_intent_meta(&db, order_id).await?;
        let shipment_id: String = shipment
            .try_get("", "shiprocket_order_id")
            .map_err(|e| e.to_string())?;
        let external_order_id: String = shipment
            .try_get("", "shiprocket_external_order_id")
            .map_err(|e| e.to_string())?;
        let awb_code: String = shipment
            .try_get("", "awb_code")
            .map_err(|e| e.to_string())?;
        let logistics_status: String = shipment
            .try_get("", "logistics_status")
            .map_err(|e| e.to_string())?;
        assert!(!shipment_id.trim().is_empty());
        assert!(!external_order_id.trim().is_empty());
        assert!(!awb_code.trim().is_empty());
        assert!(matches!(
            logistics_status.as_str(),
            "ready_to_ship" | "pickup_scheduled"
        ));
        assert_eq!(intent.status, PaymentIntentStatus::Processed);
        assert!(
            intent
                .razorpay_payment_id
                .as_deref()
                .is_some_and(|value| value.starts_with("pay_")),
            "expected a real Razorpay payment id to be persisted"
        );
        eprintln!(
            "[live-verify] shiprocket shiprocket_order_id={shipment_id} external_order_id={external_order_id} awb_code={awb_code} logistics_status={logistics_status} internal_order_id={order_id} razorpay_payment_id={}",
            intent
                .razorpay_payment_id
                .as_deref()
                .unwrap_or("")
        );
        Ok(())
    }
    .await;

    if let Some((order_id, user_id)) = cleanup {
        if let Err(err) = cleanup_live_order(&db, order_id, user_id).await {
            panic!("live cleanup failed for order {order_id}: {err}");
        }
    }
    if let Err(err) = outcome {
        panic!("{err}");
    }
}

#[tokio::test]
#[ignore = "opt-in live logistics verification; exercises real Razorpay test-mode refund and cancels the Shiprocket order"]
async fn live_pre_pickup_cancel_refunds_once_and_is_idempotent() {
    let ctx = match live_context() {
        Ok(ctx) => ctx,
        Err(reason) => {
            print_live_skip_message(&reason);
            return;
        }
    };
    let db = Database::connect(&ctx.db_url).await.expect("connect");
    let tag = unique_tag();

    let (order_id, user_id) = place_and_pay_live_order(&db, tag)
        .await
        .expect("place and pay live order");

    let first = cleanup_live_order(&db, order_id, user_id).await;
    if let Err(err) = first {
        panic!("live cleanup failed for order {order_id}: {err}");
    }

    let replay = cleanup_live_order(&db, order_id, user_id).await;
    if let Err(err) = replay {
        panic!("live cancel replay failed for order {order_id}: {err}");
    }

    let shipment = shipment_meta(&db, order_id).await.expect("shipment");
    let intent = payment_intent_meta(&db, order_id).await.expect("intent");
    let final_status = order_status_name(&db, order_id)
        .await
        .expect("order status");
    let final_inventory = inventory_quantity(&db, order_id)
        .await
        .expect("inventory quantity");
    let refund_id: String = shipment
        .try_get("", "razorpay_refund_id")
        .expect("refund id");
    let refund_status: String = shipment
        .try_get("", "refund_status")
        .expect("refund status");
    assert!(!refund_id.trim().is_empty());
    assert!(!refund_status.trim().is_empty());
    assert!(
        intent
            .razorpay_payment_id
            .as_deref()
            .is_some_and(|value| value.starts_with("pay_")),
        "expected a real Razorpay payment id to be persisted before refund"
    );
    assert!(
        matches!(refund_status.as_str(), "pending" | "processed"),
        "unexpected refund status {refund_status}"
    );
    assert!(matches!(final_status.as_str(), "cancelled" | "refunded"));
    assert_eq!(
        final_inventory, 3,
        "inventory should be restored exactly once"
    );

    let refunds_count = {
        let txn = db.begin().await.expect("refund count txn");
        let count = core_db_entities::entity::refunds::Entity::find()
            .filter(core_db_entities::entity::refunds::Column::OrderId.eq(order_id))
            .count(&txn)
            .await
            .expect("count refunds");
        txn.rollback().await.ok();
        count
    };
    assert_eq!(refunds_count, 1);

    eprintln!(
        "[live-verify] after_cancel_duplicate_retry internal_order_id={order_id} razorpay_refund_id={refund_id} refund_status={refund_status} final_order_status={final_status} inventory_quantity_available={final_inventory} refunds_table_rows={refunds_count}"
    );
}
