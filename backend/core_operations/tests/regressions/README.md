# Commerce regression tests (written, not executed)

These tests extend `integration_launch_concurrency.rs` and reuse its SeaORM/MySQL
seed helpers. The subsequent commerce production patch is present but has not been
built, tested, migrated, or deployed. `graphql` is a workspace-local
dev dependency so F1 calls the actual gateway Redis helper rather than reimplementing
it. No dependency versions were changed and no dependencies were installed. The
repository ignores Cargo.lock; it was left with its original dependency entries.

## Deployment safety and future execution

Before editing, the committed backend Compose file, both Dockerfiles, startup
scripts, and watcher configuration were inspected. Backend containers run copied
release binaries, with no backend source bind mounts or Compose watch rules. The
Caddyfile bind mount does not include tests/manifests. Frontend files are untouched.
This establishes static edit safety, not an inspection of Docker's live state.

All new tests are ignored by default. No builds, tests, formatting, migrations,
provider requests, database/Redis connections, or service commands were performed.

For a separately authorized future run:

- Local runs read **`backend/.env:TEST_DATABASE_URL`**. When that file is absent
  under GitHub Actions, the tests accept CI's `TEST_DATABASE_URL` environment
  variable only with `COMMERCE_TEST_ISOLATED_STORAGE=1` and the expected local
  ephemeral `SUDATTAS` schema. There is never a `DATABASE_URL` fallback.
  URLs/credentials are not printed.
- Require `COMMERCE_TEST_ISOLATED_STORAGE=1`. The schema must already be migrated,
  dedicated to these tests, and have a different name from the live schema. The
  flag attests that no deployed service/worker uses it. The suite does not create,
  migrate, truncate, or drop a database. Synthetic, uniquely tagged rows remain
  for inspection; use disposable isolated storage. Do not use a live schema copy
  containing customer data or runnable background jobs.
- F1 also requires `TEST_REDIS_URL` pointing to a dedicated local Redis fixture and
  `COMMERCE_TEST_ISOLATED_REDIS=1`. Only UUID-scoped keys are created/deleted; there
  is no FLUSHDB/FLUSHALL. Never point it at the running application's Redis.
- Use one test thread. New cases serialize process-local provider environment
  overrides and restore them on drop; legacy tests do not share this lock. Run
  only the new module or the specifically extended overlap test, not every legacy
  ignored test. Legacy helpers still have broader environment/database defaults.
- Deterministic suite selector: `commerce_regressions`, excluding
  `razorpay_sandbox`. Concurrent overlap retains the existing selector
  `integration_verify_and_webhook_overlap_finalize_once`.
- CI runs those selectors in a dedicated job with ephemeral MySQL and Redis
  services. The broad integration job skips this module so its real Test Mode
  cases and worker fixtures cannot run against the shared integration database.
- Fakes bind ephemeral loopback ports only when tests are eventually executed.
  They do not contact either real provider. The two acceptance/timeout cases use
  the real HTTP client's timeout (approximately 15 seconds each), not a 504 stub.
- The halfway failure case needs TRIGGER privilege. Its trigger applies to one
  seeded order and is dropped before behavioural assertions. A session marker
  proves that invoice insertion was actually reached. An interrupted future run
  may leave `regression_invoice_<order-id>` in the isolated schema; inspect there
  before reuse. No trigger was created during this task.
- Refund-worker cases refuse unrelated runnable refund attempts. They never
  neutralize other rows to make a global worker test pass.

## Existing coverage and additions

| Scenario | Existing coverage gap | New/extended coverage | Static expectation |
|---|---|---|---|
| F1 customer/key collision | No actual Redis cache isolation test in the inspected Rust fixtures | `f1_gateway_cache_never_crosses_customer_boundary`: A's cache entry is explicitly verified, B cannot receive it; `f1_changed_payload_conflicts_even_on_cache_hit`: same customer/different payment mode must conflict and leave one order | Expected to expose current cache defects |
| F5 response loss | Atomicity test rolls back failed detail inserts; does not retry a successfully committed, cart-clearing request | `f5_lost_response_retry_after_cart_deletion`: discard response, recover ID from DB, bypass cache, retry; one order/provider submission, stock 5 → 4 | Expected to expose cart-before-idempotency lookup |
| F2 uncaptured signature | Payment fixtures traditionally treated a valid signature as capture | `f2_authorised_signature_is_not_capture`: provider says authorized, real callback, no ledger/invoice/notification, booking eligibility rejected even after its time window; signed capture then duplicate delivery completes once | Expected to expose premature capture |
| F2/F3 overlap | Existing `integration_verify_and_webhook_overlap_finalize_once` checked final intent/order state, omitted ledger/invoice/outbox counts, and supplied no raw webhook signature | Existing test now delegates to shared signed fixture with a barrier; added both serial arrival orders; event must be processed, one ledger effect totaling INR 10, one invoice, one PaymentCaptured outbox row | Serial cases preserve convergence; concurrent case may expose race |
| F3 halfway failure | `integration_webhooks.rs` tests retry after missing intent; atomicity test injects before OrderDetails; logistics refund-trigger test omitted raw signature | `f3_halfway_failure_rolls_back_and_replay_converges`: invoice trigger after payment-intent/order updates, commit semantics match gRPC, durable failed event on success acknowledgement, no partial financial state, exact signed redelivery twice | Expected to expose partial commit |
| F3 tampered replay | Existing replay-by-ID test starts from a failed event, not a signature-invalid raw request | `f3_tampered_event_cannot_be_blessed_by_replay`: sign then append whitespace; rejection/no effects, invoke actual replay, still no effects | Ingress rejection preserves a safeguard; replay assertion expected to expose trust escalation |
| F4 ordering | `handler_refunds_resolve::create_refund_idempotent_returns_existing` starts with processed refund on a confirmed order; integration refund tests repeat a partial amount, not full-refund terminal state | Separate pending→processed, processed→stale pending, duplicate full refund, and two distinct partial refund cases; every event must process and totals/counts stay exact | First three expected to expose defects; two partials preserves deduplication |
| F4 worker pending response | Existing logistics fake returns processed or fails before acceptance | `f4_worker_pending_id_waits_for_processed_webhook`: worker must preserve pending status and gateway ID; processed webhook converges without another external submission | Expected to expose hard-coded processed state |
| Accepted request times out | Existing logistics retry checks equal idempotency keys after a provider rejection; rejection does not establish prior acceptance | `accepted_order_timeout_is_recovered_or_explicitly_unresolved`: one external submission, same recovered gateway ID or durable NeedsReview; `accepted_refund_timeout_requires_reconciliation_without_resubmission`: one accepted external submission, one local attempt, no local refund until authenticated recovery; `ambiguous_refund_recovers_by_webhook_without_resubmitting`: signed webhook converges without another POST | Explicit unresolved state or authenticated recovery, never a blind second submission |
| F7 free shipping | No serviceability rejection above free-shipping threshold in inspected checkout tests | `f7_free_shipping_still_requires_serviceable_pin`: INR 10 cart over INR 9.99 threshold, fake rejects destination, no Razorpay submission/order/stock mutation | Expected to expose free-shipping bypass |

Expected outcomes are based on code inspection, **not test results**. An assertion
failure stops that test; later assertions describe the guarantee after remediation.
The production patch now implements the intended safeguards, subject to static-only
review. The timeout regression was updated from its obsolete two-POST expectation
to the stronger one-POST guard: an unknown outcome must remain unresolved until
authenticated provider evidence permits local settlement.

## Razorpay Test Mode: two-phase manual/automated workflow

This is an explicitly ignored suite, not part of the default or deterministic run.
It is designed for a future authorized sandbox session, not execution against the
currently running site. Required process-local settings:

- `RUN_RAZORPAY_SANDBOX=1`
- `COMMERCE_TEST_ISOLATED_STORAGE=1`, with the isolated MySQL URL already in
  `backend/.env:TEST_DATABASE_URL`
- External `SANDBOX_RAZORPAY_KEY_ID` beginning `rzp_test_`, plus
  `SANDBOX_RAZORPAY_KEY_SECRET`
- External `SANDBOX_RAZORPAY_WEBHOOK_SECRET`, different from the checkout secret,
  the deterministic fixture secret, and the deployed webhook secret
- `RAZORPAY_SANDBOX_ARTIFACT_DIR`: a fresh, private, existing absolute directory
  outside this checkout. Keep artifacts out of version control; real webhook
  bodies may contain contact data even in Test Mode.

1. In that future session, select only
   `razorpay_sandbox::prepare_test_orders` with ignored tests enabled and one test
   thread. It creates three actual INR 10 Test Mode orders through the production
   server-authoritative order resolver, saves synthetic fixture IDs in
   `orders.json`, and asserts that each internal order is unpaid. It persists each
   accepted operation immediately and refuses an existing manifest, avoiding
   silent duplicate order creation after a partial preparation failure.
2. On a dedicated sandbox Checkout page, supply each generated gateway order ID
   and the Test Mode public key. Use the provider's Test Mode successful-payment
   interaction for `callback_and_capture` and `webhook_only`; use its declined
   payment interaction for `declined`. Capture genuine provider results; do not
   invent payment IDs or call undocumented payment-simulation endpoints. The
   current suite does not automate the browser Checkout interaction.
3. Save the genuine successful callback object for `callback_and_capture` as
   `callback.json` (`razorpay_order_id`, `razorpay_payment_id`,
   `razorpay_signature`). Do **not** forward it to the running application's API.
   For `webhook_only`, discard the callback entirely (or close the browser after
   completing payment). Configure capture in the isolated Test Mode account as
   needed; an authorized-only payment is not capture evidence.
4. Configure a dedicated, publicly reachable **sandbox event collector** with the
   dedicated test webhook secret. It must retain raw bodies and headers without
   forwarding to production or mutating these test fixtures. Subscribe to
   `payment.captured` and `payment.failed`. For each case save the exact raw event
   body as `<case>.body.json`, `X-Razorpay-Signature` as `<case>.signature.txt`, and
   `X-Razorpay-Event-Id` as `<case>.event-id.txt`. Do not reformat JSON. Provisioning
   the collector or changing any account configuration is outside this task.
5. Select only `razorpay_sandbox::assert_manual_checkout_outcomes`. It fetches each
   payment from the real Test Mode API and checks order, amount, currency, and
   failed/captured status; validates the original raw-body signature; processes
   the callback only for its designated case; delivers each signed event twice;
   then asserts unpaid/no invoice/no shipment for decline, or one ledger effect,
   invoice and notification for success, including browser-callback omission.
   This phase requires fresh unprocessed fixtures and is not intended as a
   rerunnable reconciler after partial assertion failure.

## Remaining limits / seams

- No required deterministic scenario is blocked on a new production seam. The
  existing API-base override, public handlers/replay/worker, and test-only MySQL
  trigger provide the injection points. The new fake follows the existing Warp
  provider-fake convention but does not assume provider rejection equals timeout.
- F1 exercises the real Redis helper plus real core checkout, not JWT transport or
  full GraphQL serialization. A future actor-aware cache API will require updating
  this harness to pass its new actor/payload parameters.
- Webhook tests reproduce the core gRPC transaction boundary. They assert durable
  retry state when that boundary returns success. Actual Warp HTTP response codes,
  proxy delivery, and worker scheduling still require end-to-end verification;
  these tests must not be reported as proof of HTTP acknowledgement behaviour.
- The barrier coordinates handler starts; it does not force a particular SQL
  interleaving. Repeated multi-connection stress and production isolation settings
  remain unverified. Each handler obtains a separate transaction from the MySQL pool.
- The refund fake guarantees provider deduplication only when given the same
  `X-Refund-Idempotency` key. Real provider behaviour after acceptance and network
  loss remains an external contract to verify; the Test Mode suite does not refund.
- Sandbox browser work and public event collection require a human/isolated
  endpoint. Automated assertions are supplied, but those steps have not occurred.
- The suite's compilation and all assertions remain unverified: **written only;
  neither built nor run**. No service, container, frontend process, tunnel, live
  configuration, credential, database record, or Redis cache was changed.
