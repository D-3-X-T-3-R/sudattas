# Sudattas Cross-Layer Contract (Architecture + Boundaries)

This document defines the frontend/backend boundary for auth, route families, idempotency, and error normalization.

## Backend layering reference

The backend architecture remains:

`DB -> operations/handlers -> GraphQL -> security -> payments/webhooks -> procedures/outbox`

Frontend integration must align with this layering and must not bypass backend security policy.

## Route families and auth expectations

There are exactly three request families.

1. Public storefront reads
- Frontend entrypoints: `/api/products`, `/api/products/[id]`, `/api/storefront-filters`
- Backend auth mode: guest session only (`X-Session-Id`)
- Allowed operations: read-only storefront queries
- Not allowed: admin mutations, privileged headers from browser

2. Authenticated customer actions
- Frontend entrypoints:
`/api/account/addresses`
`/api/account/orders`
`/api/account/orders/[orderId]`
`/api/account/wishlist`
`/api/account/profile`
plus tightly controlled direct GraphQL calls using `Authorization: Bearer <id_token|access_token>` from NextAuth session sync where proxy routes are not yet implemented.
- Backend auth mode: JWT (`AuthSource::Jwt`)
- Allowed operations: customer-scoped cart/checkout/profile/wishlist/order actions
- Not allowed: admin-only actions

3. Admin actions
- Frontend entrypoint: `/api/admin/graphql` only
- Backend auth mode: JWT from server-side admin proxy
- Admin authorization: NextAuth session + `ADMIN_ALLOWED_EMAILS` allowlist
- Backend resolver enforcement: JWT user id must be in `ADMIN_ALLOWED_USER_IDS`
- Not allowed: direct browser requests to backend privileged GraphQL using admin credentials

### Account order detail contract

1. `/api/account/orders/[orderId]` must return customer-scoped order data only.
2. Backend GraphQL order-adjacent reads (`getPaymentIntent`, `getShipment`, `getOrderEvents`, `searchOrderEvents`) enforce customer ownership or admin role.
3. The profile UI displays:
- canonical order status
- derived payment state
- derived fulfillment state
- line items, payment intents, and shipment rows when present.

## Header ownership contract

1. Browser-owned
- `X-Session-Id` for guest storefront reads
- `Authorization: Bearer ...` for authenticated customer actions

2. Server-owned
- Admin GraphQL forwarding credentials on `/api/admin/graphql`
- `Idempotency-Key` forwarding for money-moving mutations
- `X-Request-Id` forwarding for trace correlation
- `X-Client-Action` forwarding for route/action attribution
- `X-Guest-Session-Id` forwarding for guest-session correlation when present

3. Forbidden in browser bundles
- `NEXT_PUBLIC_ADMIN_API_KEY`
- `NEXT_PUBLIC_GRAPHQL_TOKEN`
- `NEXT_PUBLIC_GRAPHQL_SESSION_ID` as privileged fallback for admin

## CSRF and Origin enforcement (non-relaxation rule)

1. Backend CSRF/origin checks are security policy, not UX tuning knobs.
2. No production fix may disable, bypass, or weaken backend CSRF checks to make frontend mutations pass.
3. Frontend changes must align request `Origin`/`Referer` behavior with backend `ALLOWED_ORIGINS` expectations.
4. Security regressions must be prevented by tests, not convention-only review.

### CSRF/origin expectations by route family

1. Public storefront reads (`/api/products`, `/api/products/[id]`, `/api/storefront-filters`)
- Frontend -> Next route may include `x-session-id` for guest continuity.
- Next -> backend uses `X-Session-Id` and must provide Origin/Referer that matches `ALLOWED_ORIGINS` when that backend policy is enabled.

2. Authenticated customer actions (`/api/account/...`)
- Frontend -> Next route uses same-origin browser request.
- Next -> backend uses `Authorization: Bearer <jwt>`; CSRF origin check is not the auth boundary for this path (JWT is).

3. Admin actions (`/api/admin/...`)
- Frontend -> Next route uses same-origin browser request.
- Next -> backend uses server-only JWT forwarding after admin allowlist verification.
- Browser must never send privileged admin headers directly to backend GraphQL.

### Mutation-path CSRF regression coverage

Security regression tests verify disallowed-origin rejection for session-auth mutation paths used by storefront:

1. `addCartItem`
2. `updateCartItem`
3. `deleteCartItem`
4. `createPaymentIntent`
5. `verifyRazorpayPayment`
6. `createShippingAddress` (when attempted via session auth path)

## Guest session behavior

1. Guest session is minted from backend `POST /session/guest`.
2. Storefront read routes recover from stale guest sessions by minting a new one.
3. Public APIs never fall back to admin query clients.

## Error response shape consumed by frontend

Frontend-facing API routes should normalize to this envelope:

```json
{
  "ok": false,
  "data": null,
  "errorCode": "UNAUTHORIZED",
  "message": "Human-readable message",
  "fieldErrors": {},
  "retryable": false
}
```

Current admin GraphQL passthrough returns GraphQL `data/errors`; this envelope is the target for route-level normalization and must be applied consistently as follow-up hardening.

## Idempotency requirements for mutations

`Idempotency-Key` is required for duplicate-prone or money-moving mutations:

1. `placeOrder`
2. payment verification/capture flows
3. webhook replay-sensitive side effects

Retries without a stable key are non-compliant.

## Pagination and query-bound contract

1. Frontend must treat `50` as the effective max page size for GraphQL list/search requests.
2. Any analytics/dashboard flow needing more than one page must iterate using `offset` + bounded `limit`, not request very large one-shot limits.
3. Backend list resolvers must clamp requested limits via `graphql_limits::cap_page_size` (including related-products/search list paths).
4. Admin customer listing must use paged GraphQL inputs (`SearchUserInput.limit` + `offset`) and iterate pages client-side instead of list-all fetches.

## Validation parity rules

1. Address create/update must enforce:
- `road` required and max 500 chars
- `postalCode` exactly 6 digits
- non-empty `country`, `stateRegion`, `city`
2. User create/update must validate email format and phone format (optional, but if present requires at least 10 digits with valid phone characters).
3. Admin product create/update must validate:
- positive money amount (`pricePaise > 0`)
- `sku`/`slug` character set and length consistency.

## Money contract (paise/rupees)

1. Network contract for money is always integer paise (`amountPaise` / `pricePaise`) in API and GraphQL payloads.
2. Frontend display may use backend `formatted` money string or shared INR formatting from paise; avoid ad-hoc per-screen formatters.
3. Frontend rupee inputs must convert to paise using string-safe conversion (no float multiplication).
4. Product search money filters must be non-negative paise and `starting <= ending`.
5. Money amounts are bounded to backend-supported integer range (`<= 2147483647` paise) for validation parity.

## Major mutation contract matrix

| Mutation flow | Required auth | Idempotency | Validation owner | Expected error codes |
|---|---|---|---|---|
| `placeOrder` | Customer JWT | Required | Backend + frontend schema mirror | `UNAUTHORIZED`, `VALIDATION_ERROR`, `CONFLICT`, `PAYMENT_REQUIRED` |
| `createPaymentIntent` | Customer JWT | Required | Backend | `UNAUTHORIZED`, `VALIDATION_ERROR`, `CONFLICT` |
| `verify/capture payment` | Customer JWT or trusted server flow | Required | Backend signature checks | `UNAUTHORIZED`, `INVALID_SIGNATURE`, `CONFLICT`, `NEEDS_REVIEW` |
| `createShippingAddress` | Customer JWT | Recommended (required from checkout retries) | Backend + frontend | `UNAUTHORIZED`, `VALIDATION_ERROR` |
| `admin create/update/delete product` | Admin allowlisted session via `/api/admin/graphql` | Recommended | Backend + frontend admin form validation | `UNAUTHORIZED`, `FORBIDDEN`, `VALIDATION_ERROR`, `CONFLICT` |
| `admin order status updates` | Admin allowlisted session via `/api/admin/graphql` | Recommended | Backend state machine | `UNAUTHORIZED`, `FORBIDDEN`, `INVALID_STATE`, `CONFLICT` |

## Checkout and payment state mapping

Checkout and account UI must map directly to backend order/payment truth instead of invented labels.

### Payment state mapping

1. Backend payment intent `status` containing `captured` or `paid` -> UI `paid`
2. `verified` -> UI `verified`
3. `needs_review` -> UI `needs_review`
4. `failed` -> UI `failed`
5. `refunded` -> UI `refunded`
6. Unknown/empty -> UI `pending`

### Order status mapping

1. `status_id=1` -> UI `pending`
2. `status_id=2` -> UI `processing`
3. `status_id=3` -> UI `shipped`
4. `status_id=4` -> UI `delivered`
5. `status_id=5` -> UI `cancelled`
6. Any other value -> passthrough raw status id

### Webhook-aware UX rule

Frontend must not assume payment finality from the initial client callback alone. After verify, UI shows backend-derived payment/order state and treats non-final states as pending finalization until backend webhook processing converges.

## Enforcement summary implemented

1. Public API routes now use storefront session query paths only.
2. Admin browser actions now flow through `/api/admin/graphql` server proxy only.
3. Admin pages are server-protected by middleware with allowlist checks before render.
4. Frontend privileged `NEXT_PUBLIC_*` admin fallbacks were removed from GraphQL clients.
