# Review of External Remarks

Assessment of the critical remarks against the current codebase. **Valid** = accurate and still an issue; **Addressed** = fixed or partially fixed; **Action** = what to do.

---

## 1) Money math (f64)

**Verdict: Addressed**

- **Was:** place_order used f64 for gross/total; `amount_paise = (total_amount * 100.0) as i64`; coupon path used paise→f64→store.
- **Now:** Place order and order/payment logic use integer paise (minor units) end-to-end; money helpers in `money.rs`; order snapshot and create_order use paise-derived totals; no float math in core pricing path (Phase 1).

---

## 2) Inventory (race / no decrement)

**Verdict: Addressed**

- **Was:** Read-then-check stock; no atomic update; concurrent checkouts could oversell.
- **Now:** Atomic decrement in place_order: `UPDATE ... WHERE quantity_available >= ?`; check affected rows before creating order; inventory update in same transaction as order creation. Concurrency tests: two checkouts for last unit → exactly one succeeds (Phase 3).

---

## 3) Coupons (usage_count not incremented at checkout)

**Verdict: Addressed**

- **Was:** place_order only called check_coupon; usage_count never incremented; usage_limit not enforced.
- **Now:** Coupon snapshot stored on order at place_order (applied_coupon_id, applied_coupon_code, applied_discount_paise). `usage_count` incremented only on **verified payment capture** (in ingest_webhook after capture_payment), in same transaction; atomic UPDATE with usage_limit check. Tests: coupon not incremented by place_order alone; limit enforced under concurrency (Phase 4).

---

## 4) Payments only half-integrated

**Verdict: Addressed**

- **Was:** capture_payment only updated payment_intents; orders and order_events unchanged; no amount/currency verification.
- **Now:** On verified payment.captured (ingest_webhook → process_payment_captured): (1) amount/currency verified (webhook vs intent; if order linked, vs order grand_total_minor); mismatch → order/intent marked NeedsReview, not paid; (2) orders.payment_status and orders.status_id updated via **order state machine** (transition to Paid or NeedsReview); (3) order_events emitted (payment_captured / payment_mismatch); (4) inventory already decremented at place_order (Model A). All handlers (place_order, ingest_webhook, update_order) use the central state machine (Phases 5, 7).

---

## 5) Webhook endpoint abusable

**Verdict: Addressed**

- **Was:** GraphQL always called gRPC and stored webhooks; invalid/missing signature still persisted.
- **Now:** If `RAZORPAY_WEBHOOK_SECRET` is set and signature is missing or invalid, GraphQL webhook handler returns **400/401** and **does not** call gRPC (does not persist). Replay protection via provider_event_id uniqueness. Tests: valid vs invalid/missing signature, replay attempts (Phase 6).

---

## Already addressed (since the remarks)

| Item | Status |
|------|--------|
| CI (unit + integration tests, migrations) | Green. |
| Rate limiting (GraphQL + webhooks) | Per-IP rate limit applied; 429 when exceeded. |
| Idempotency for place_order / capture_payment | Idempotency-Key header; Redis cache; duplicate requests return cached response. |
| Timeouts + retry for GraphQL→gRPC | gRPC client has request/connect timeouts, connect retries, circuit breaker (RESILIENCE.md). |
| Request_id / trace_id | Request ID in Context; propagated to gRPC for place_order and capture_payment. |

---

## Still missing (from "commerce-grade" list)

| Item | Status |
|------|--------|
| GraphQL query depth/complexity limiting | Not implemented. |
| Reject invalid webhook signatures at HTTP layer | **Done.** 400/401 at HTTP layer when secret set and signature invalid/missing; no gRPC call, no persist. |

---

## Top 10 fixes (prioritised)

1. **Money:** ~~Replace f64 with integer paise~~ **Done.** Paise/minor units end-to-end; money helpers (Phase 1).
2. **Inventory:** ~~Atomic decrement~~ **Done.** UPDATE … WHERE quantity_available >= ? in same txn as order (Phase 3).
3. **Order snapshot:** ~~Store immutable line-level and coupon snapshot~~ **Done.** Line-level + order-level snapshot; coupon on order (Phase 4).
4. **Payment capture:** ~~Update orders, order_events, verify amount/currency~~ **Done.** Via state machine; mismatch → NeedsReview (Phases 5, 7).
5. **Coupons:** ~~Increment usage_count; store snapshot~~ **Done.** Snapshot at place_order; usage_count only on verified capture (Phase 4).
6. **Webhooks:** ~~Reject invalid signature at HTTP layer~~ **Done.** 400/401, no gRPC/persist (Phase 6).
7. **Idempotency:** Done for place_order and capture_payment (durable DB keys).
8. **GraphQL depth/complexity:** Not done. Add limits and pagination for list fields.
9. **gRPC timeouts/retry:** Done.
10. **Metrics / trace:** Request_id done; optional metrics (graphql_requests_total, duration).

---

*Summary: Remarks 1–5 are **addressed** (Phases 1–7: money, inventory, order snapshot, coupons, payment capture integration, webhook security, order state machine). Remaining: GraphQL depth/complexity and pagination. Idempotency, rate limiting, gRPC timeouts/retry, and request_id propagation remain in place.*
