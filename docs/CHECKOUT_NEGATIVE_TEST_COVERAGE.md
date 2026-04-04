# Checkout Negative Test Coverage

This maps Section 6.4 checklist items to automated tests currently in the repository.

## 6.4 coverage map

1. Test invalid signature
- `backend/core_operations/tests/handler_payment_intents.rs`
  - `verify_razorpay_payment_invalid_signature_returns_false_and_does_not_update`
- `backend/core_operations/tests/integration_payments.rs` (ignored integration)
  - `integration_verify_razorpay_payment_invalid_signature_no_update`
- `backend/graphql/src/webhooks/mod.rs`
  - `webhook_signature_validation_when_secret_set`

2. Test duplicate verification attempt
- `backend/core_operations/tests/handler_payment_intents.rs`
  - `verify_razorpay_payment_duplicate_attempt_same_payload_is_idempotent`

3. Test payment capture mismatch
- `backend/core_operations/tests/integration_webhooks.rs` (ignored integration)
  - `integration_webhook_amount_mismatch_marked_needs_review_not_paid`
  - `integration_webhook_currency_mismatch_marked_needs_review_not_paid`

4. Test webhook delayed
- `backend/core_operations/tests/integration_webhooks.rs` (ignored integration)
  - `integration_webhook_out_of_order_same_payment_second_idempotent`

5. Test payment marked `needs_review`
- `backend/core_operations/tests/integration_webhooks.rs` (ignored integration)
  - `integration_webhook_amount_mismatch_marked_needs_review_not_paid`
  - `integration_webhook_currency_mismatch_marked_needs_review_not_paid`

6. Test order created but payment unresolved
- `backend/core_operations/tests/integration_payments.rs` (ignored integration)
  - `integration_place_order_creates_payment_intent` (asserts `pending` payment intent right after place order)

## Notes

1. Integration tests are intentionally `#[ignore]` and require `TEST_DATABASE_URL` plus migrated schema.
2. Unit tests run in CI-like local runs without database and cover signature and duplicate verification behavior.
