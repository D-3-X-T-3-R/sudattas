# Async Reconciliation in Checkout UI

This doc describes how frontend UI reconciles delayed backend side effects (webhooks/outbox-driven transitions).

## Why reconciliation is needed

- Payment verification from checkout can arrive before webhook-driven finalization is complete.
- Backend may temporarily report intermediate states (`verified`, `pending`) before final states.
- UI must not treat the first callback as final truth.

## Current frontend behavior

- `useRazorpayTest.runCheckout` calls:
  1. `/api/checkout/place-order`
  2. Razorpay checkout
  3. `/api/checkout/verify-payment`
- If verify returns a non-final payment state, frontend enters reconciliation mode:
  - Poll `/api/account/orders/:orderId` for a short bounded window.
  - Stop polling as soon as payment enters a final state (`paid`, `failed`, `refunded`, `needs_review`).
  - If still non-final after the polling window, show a clear message to refresh profile orders shortly.

## UX contract

- Success message must reflect backend-confirmed state, not client callback assumptions.
- Intermediate states are communicated as "processing/reconciling", not as failure.
- Errors shown to users are normalized through route-state mapping (no raw backend internals).

## Operational note

- This reconciliation is intentionally bounded (short polling window) to avoid aggressive client load.
- Source of truth remains backend order/payment state endpoints.
