# User Journey Catalog (Frontend + Backend)

Last Updated: 2026-04-04
Status: Source-of-truth blueprint

## How to read this blueprint

Each journey item is path-oriented and should be interpreted as:
- Entry point
- User path through UI (click/submit/navigation flow)
- Backend/API/GraphQL path touched
- Expected outcome
- Edge/failure outcome

---

## 1) Entry And Discovery

- Land on home page as first-time user.
- Land on home page as returning user with guest session cookie.
- Home page with slow network and skeletons visible.
- Home page with backend unavailable (graceful fallback/error UI).
- Navigate via top nav links.
- Navigate via footer links.
- Open site directly on category/deep link URL.
- Open non-existent route (404 page UX).
- Browser refresh on deep page (state/session preserved).
- Back/forward navigation behavior across pages.
- The Direct-to-WhatsApp Flow: Premium consultation entry point.


## 2) Catalog Browsing

- View product listing default sort/filter.
- Apply one filter.
- Apply multiple filters.
- Clear filters.
- Pagination next/prev.
- Pagination with out-of-range page.
- Empty result set UX.
- Search with normal keyword.
- Search with typo/special chars.
- Search with very long query.
- Search with unicode/emoji.
- Product card rendering with missing optional data.
- Product card rendering with long names/prices.
- Product listing API 500/timeout path.
- Product listing partial data path.
- Collection/category page notFound path for unknown slug/id.


## 3) Product Details

- Open PDP from listing.
- Open PDP direct URL.
- Variant selection (size/color) valid.
- Variant out-of-stock state.
- No-variant product state.
- Quantity selector min boundary.
- Quantity selector max boundary.
- Invalid quantity input (0, negative, non-number).
- Add-to-cart success from PDP.
- Add-to-cart when session missing (session mint flow).
- Add-to-cart when backend rejects validation.
- Add-to-cart when rate limited (429 UX).
- Image gallery interactions.
- Missing image fallback.
- Related products click-through.
- The Fabric Deep-Dive: User interacts with high-res zoom or fabric video.

- PDP notFound path for invalid product id.


## 4) Guest Session Lifecycle

- First request mints guest session.
- Session reused across pages.
- Session expires while browsing.
- Session invalid/corrupted ID.
- Session recreation after invalidation.
- Guest session merge prompt after login.
- Multiple tabs share same guest session.
- New private window gets independent session.
- Single-flight guest session mint under concurrent first-load requests.


## 5) Cart And Bag

- Open empty cart.
- Open cart with items.
- Update quantity success.
- Update quantity exceeds inventory.
- Remove one item.
- Remove all items.
- Cart persists after refresh.
- Cart persists across restart/browser reopen.
- Cart API unauthorized flow.
- Cart API validation error envelope.
- Cart API timeout/retry behavior.
- Concurrent cart edits from two tabs.
- Cart totals recalc after mutation.
- Cart with stale variant/product removed from catalog.
- Cart with price changed since add.
- Guest cart merge into authenticated cart after login.

- Guest cart merge failure when guestSessionId is missing/invalid.


## 6) Checkout Basics

- Checkout blocked for empty cart.
- Checkout as guest if allowed by policy.
- Checkout requiring login (redirect path).
- Select existing shipping address.
- Add new shipping address during checkout.
- Invalid address validation (postal code/required fields).
- Duplicate/default address logic.
- Checkout summary accuracy (subtotal/shipping/tax/discount/total).
- Idempotent order placement (double-click Place Order).
- Refresh on confirmation page.
- Network loss after clicking Place Order.
- Backend validation failure on Place Order.
- Inventory changed between cart and place order.
- Coupon application success.
- Coupon invalid/expired/min-order/not-eligible.
- Coupon removed/changed after apply.
- Retry place order after transient failure.
- Post-order cart clear behavior.
- The Hybrid Payment (COD + Online): If you offer partial advance for custom orders.


## 7) Payments

- Create payment intent success.
- Payment gateway popup success path.
- User cancels payment popup.
- Payment verification success.
- Payment verification signature mismatch.
- Duplicate verification idempotency.
- Async payment status event arrives before client verify (mock-safe reconciliation).
- Duplicate async payment status event replay (mock-safe idempotency check).
- Async payment amount mismatch state path (mock-safe reconciliation check).
- Async payment currency mismatch state path (mock-safe reconciliation check).
- Needs-review status path UX.
- Payment success but order state not updated (recovery path).
- Payment failure and retry.
- Timeout from gateway.
- Refund initiation success.
- Partial refund path.
- Full refund path.
- Refund in non-refundable state.

## 8) Authentication And Identity

- Login with valid credentials.
- Login with invalid credentials.
- OAuth login success.
- OAuth callback failure.
- Session expiry while logged in.
- Logout and protected route access after logout.
- Route requiring auth redirects correctly.
- Access account APIs without auth (401 envelope).
- CSRF disallowed origin rejection.
- CSRF allowed origin success.
- JWT malformed/expired token behavior.
- Account takeover-resistant error messaging (no user enumeration).
- JWT Handover: Ensure the JWT minted by Rust is correctly handled by NextAuth in the frontend.

- Phone OTP request success via login dialog. [Deferred - do not implement now]

- Phone OTP request failure (invalid phone/upstream OTP error). [Deferred - do not implement now]

- Phone OTP verification success creates authenticated session. [Deferred - do not implement now]

- Phone OTP verification failure (wrong/expired code). [Deferred - do not implement now]

- Auth capabilities endpoint reports guest/customer/admin mode correctly.


## 9) Account Area

- View profile.
- Edit profile success.
- Edit profile validation errors.
- View order history.
- View order details access control (own order only).
- View empty order history.
- Save address from account.
- Update/delete address from account.
- Wishlist add/remove/view.
- Wishlist persists across sessions.
- Account API partial failure UX.
- The Custom Measurements Vault: User saves specific body dimensions for tailoring.

- Stale/unauthorized account route shows re-auth CTA and safe fallback.


## 10) Admin Journeys

- Admin login success.
- Non-admin user blocked from admin routes.
- Admin product create/update/delete.
- Admin product with invalid payload.
- Admin category CRUD.
- Admin inventory adjust + inventory logs.
- Admin order search/filter.
- Admin mark shipped.
- Admin mark delivered.
- Illegal order state transition blocked.
- Admin review moderation.
- Admin coupon create/update/deactivate.
- Admin user role changes.
- Admin transaction/refund visibility.
- Admin API unauthorized and forbidden envelopes.
- No browser direct privileged GraphQL from client components.
- The Lookbook/Editorial CMS Path: Admin updating high-fashion imagery.

- Admin middleware redirect: unauthorized user is redirected to /imtheboss/login?error=AccessDenied.

- Admin telemetry summary visible to admin role.

- Admin telemetry endpoints reject non-admin access with 401 envelope.


## 11) Security And Abuse Paths

- Missing auth header on protected mutations.
- Forged admin header attempts.
- Privileged env leakage audit (NEXT_PUBLIC_*).
- Rate limit normal user behavior.
- Rate limit burst abuse.
- Rate limit behind proxy headers.
- Input injection-like payloads.
- Large payload rejection.
- GraphQL depth/complexity limit rejection.
- Unknown field/invalid argument error shape consistency.
- Idempotency-key replay same payload.
- Idempotency-key replay different payload conflict.
- gRPC Timeout Recovery: Backend is slow; Next.js BFF must show a specific "Retry" UI.

- Server-only admin GraphQL proxy prevents INTERNAL_API_SECRET leakage to browser.


## 12) Reliability And Failure Recovery

- GraphQL server restart during active session.
- Core operations restart during checkout.
- Redis unavailable.
- MySQL unavailable.
- Slow DB responses.
- Partial dependency failure (one service down).
- Retry behavior and user-visible messaging.
- Background outbox processing lag.
- Webhook delayed delivery.
- Duplicate events handling.
- Stuck "processing" order recovery procedure.
- Graceful degradation for storefront reads.
- BFF-to-GraphQL/gRPC contract mismatch: Next.js API route sends payload that no longer matches backend contract and must return controlled validation error UX.

- Telemetry summary degrades gracefully when backend Prometheus metrics are unavailable.

- 429/network retry path via BFF resilience wrapper without duplicate side effects.


## 13) Performance UX

- First load performance on mobile.
- Revisit/cached navigation speed.
- Large catalog page performance.
- Image-heavy PDP performance.
- Interaction latency for filter/sort.
- API waterfall avoidance in critical paths.
- Layout shift checks on load.
- Slow 3G behavior for key journeys.

## 14) Accessibility

- Keyboard-only navigation for all primary flows.
- Focus management on modals/drawers.
- Focus visible styles.
- Form labels/aria validity.
- Screen-reader meaningful announcements for errors/loading.
- Color contrast checks.
- Error states accessible messaging.
- Skip links and heading structure.
- Mobile zoom/readability.

## 15) SEO/Content Surface

- Indexable storefront pages.
- Correct canonical tags.
- robots/sitemap endpoints.
- Structured metadata for products.
- 404/500 pages index handling.
- Share preview metadata on PDP.

## 16) Device/Browser Matrix

- Chrome desktop.
- Safari desktop.
- Firefox desktop.
- Edge desktop.
- iOS Safari.
- Android Chrome.
- Narrow viewport (320px).
- Tablet viewport.
- Dark/light mode if supported.
- Locale/currency formatting consistency.

## 17) Site Path Matrix (Per Journey)

| ID | Section | Journey | Site Path User Takes | Primary API/GraphQL Endpoint(s) |
|---|---|---|---|---|
| UJ-001 | 1 | Land on home page as first-time user. | `/` | `GET /ready`, `/api/products` |
| UJ-002 | 1 | Land on home page as returning user with guest session cookie. | `/` (with existing guest session cookie) | `GET /ready`, `/api/products` |
| UJ-003 | 1 | Home page with slow network and skeletons visible. | `/` (throttled network) | `GET /ready`, `/api/products` |
| UJ-004 | 1 | Home page with backend unavailable (graceful fallback/error UI). | `/` (backend-down fallback path) | `GET /ready`, `/api/products` |
| UJ-005 | 1 | Navigate via top nav links. | `/` -> top-nav routes (`/`, `/products`, `/account/orders`, `/admin/login`) | `GET /ready`, `/api/products` |
| UJ-006 | 1 | Navigate via footer links. | `/` -> footer-linked pages (policy/help/contact routes) | `GET /ready`, `/api/products` |
| UJ-007 | 1 | Open site directly on category/deep link URL. | Direct open: `/categories/[slug]` or `/products/[slug]` | `GET /api/products`, GraphQL product query |
| UJ-008 | 1 | Open non-existent route (404 page UX). | `/does-not-exist` | `GET /<unknown-route>` (frontend 404) |
| UJ-009 | 1 | Browser refresh on deep page (state/session preserved). | `/products/[slug]` (refresh on same URL) | `GET /ready`, `/api/products` |
| UJ-010 | 1 | Back/forward navigation behavior across pages. | `/` -> `/products/[slug]` -> `/cart` (browser back/forward) | `GET /ready`, `/api/products` |
| UJ-011 | 2 | View product listing default sort/filter. | `/products` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-012 | 2 | Apply one filter. | `/products?filter=<value>` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-013 | 2 | Apply multiple filters. | `/products?f1=...&f2=...` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-014 | 2 | Clear filters. | `/products` (query reset) | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-015 | 2 | Pagination next/prev. | `/products?page=n` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-016 | 2 | Pagination with out-of-range page. | `/products?page=9999` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-017 | 2 | Empty result set UX. | `/products?filter=no-match` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-018 | 2 | Search with normal keyword. | `/products?query=<term>` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-019 | 2 | Search with typo/special chars. | `/products?query=<term>` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-020 | 2 | Search with very long query. | `/products?query=<term>` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-021 | 2 | Search with unicode/emoji. | `/products?query=<term>` | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-022 | 2 | Product card rendering with missing optional data. | `/products` (card render path) | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-023 | 2 | Product card rendering with long names/prices. | `/products` (card render path) | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-024 | 2 | Product listing API 500/timeout path. | `/products` (listing request failure path) | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-025 | 2 | Product listing partial data path. | `/products` (card render path) | `/api/products` and/or GraphQL `searchProduct` query |
| UJ-026 | 3 | Open PDP from listing. | `/products` -> `/products/[slug]` | GraphQL product detail query, storefront product API/query |
| UJ-027 | 3 | Open PDP direct URL. | `/products/[slug]` | GraphQL product detail query, storefront product API/query |
| UJ-028 | 3 | Variant selection (size/color) valid. | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-029 | 3 | Variant out-of-stock state. | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-030 | 3 | No-variant product state. | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-031 | 3 | Quantity selector min boundary. | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-032 | 3 | Quantity selector max boundary. | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-033 | 3 | Invalid quantity input (0, negative, non-number). | `/products/[slug]` (option/qty controls) | GraphQL product detail query, storefront product API/query |
| UJ-034 | 3 | Add-to-cart success from PDP. | `/products/[slug]` -> `/cart` | GraphQL `addCartItem` mutation, `POST /session/guest` (if no session) |
| UJ-035 | 3 | Add-to-cart when session missing (session mint flow). | `/products/[slug]` -> `/cart` | GraphQL `addCartItem` mutation, `POST /session/guest` (if no session) |
| UJ-036 | 3 | Add-to-cart when backend rejects validation. | `/products/[slug]` -> `/cart` | GraphQL `addCartItem` mutation, `POST /session/guest` (if no session) |
| UJ-037 | 3 | Add-to-cart when rate limited (429 UX). | `/products/[slug]` -> `/cart` | GraphQL `addCartItem` mutation, `POST /session/guest` (if no session) |
| UJ-038 | 3 | Image gallery interactions. | `/products/[slug]` (media gallery area) | GraphQL product detail query, storefront product API/query |
| UJ-039 | 3 | Missing image fallback. | `/products/[slug]` (media gallery area) | GraphQL product detail query, storefront product API/query |
| UJ-040 | 3 | Related products click-through. | `/products/[slug]` -> `/products/[related-slug]` | GraphQL `getRelatedProducts` query |
| UJ-041 | 4 | First request mints guest session. | `/` or `/products/[slug]` (first stateful action) | `POST /session/guest` |
| UJ-042 | 4 | Session reused across pages. | `/` -> `/products/[slug]` -> `/cart` -> `/checkout` | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-043 | 4 | Session expires while browsing. | Any stateful route (`/cart`, `/checkout`, `/account/*`) during session check | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-044 | 4 | Session invalid/corrupted ID. | Any stateful route (`/cart`, `/checkout`, `/account/*`) during session check | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-045 | 4 | Session recreation after invalidation. | Any stateful route (`/cart`, `/checkout`, `/account/*`) during session check | `POST /session/guest` |
| UJ-046 | 4 | Guest session merge prompt after login. | `/cart` (guest) -> `/login` -> `/cart` (merged) | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-047 | 4 | Multiple tabs share same guest session. | Tab A `/products/[slug]` + Tab B `/cart` | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-048 | 4 | New private window gets independent session. | Normal window `/cart` vs private window `/cart` (independent sessions) | Session-backed GraphQL requests (`GRAPHQL_SESSION_ID`), `POST /session/guest` fallback |
| UJ-049 | 5 | Open empty cart. | `/cart` | GraphQL `getCartItems` query |
| UJ-050 | 5 | Open cart with items. | `/cart` | GraphQL `getCartItems` query |
| UJ-051 | 5 | Update quantity success. | `/cart` (line-item actions) | GraphQL `updateCartItem` mutation |
| UJ-052 | 5 | Update quantity exceeds inventory. | `/cart` (line-item actions) | GraphQL `updateCartItem` mutation |
| UJ-053 | 5 | Remove one item. | `/cart` (line-item actions) | GraphQL `deleteCartItem` mutation |
| UJ-054 | 5 | Remove all items. | `/cart` (line-item actions) | GraphQL `deleteCartItem` mutation |
| UJ-055 | 5 | Cart persists after refresh. | `/cart` (reload/browser restart) | GraphQL cart endpoints |
| UJ-056 | 5 | Cart persists across restart/browser reopen. | `/cart` (reload/browser restart) | GraphQL cart endpoints |
| UJ-057 | 5 | Cart API unauthorized flow. | `/cart` (API error handling path) | GraphQL cart endpoints with auth/error envelopes |
| UJ-058 | 5 | Cart API validation error envelope. | `/cart` (API error handling path) | GraphQL cart endpoints with auth/error envelopes |
| UJ-059 | 5 | Cart API timeout/retry behavior. | `/cart` (API error handling path) | GraphQL cart endpoints with auth/error envelopes |
| UJ-060 | 5 | Concurrent cart edits from two tabs. | Tab A `/cart` + Tab B `/cart` (same account/session) | GraphQL cart endpoints with auth/error envelopes |
| UJ-061 | 5 | Cart totals recalc after mutation. | `/cart` (line-item actions) | GraphQL cart queries/mutations (`getCartItems`, `updateCartItem`) |
| UJ-062 | 5 | Cart with stale variant/product removed from catalog. | `/cart` (line-item actions) | GraphQL `deleteCartItem` mutation |
| UJ-063 | 5 | Cart with price changed since add. | `/cart` (line-item actions) | GraphQL cart queries/mutations (`getCartItems`, `updateCartItem`) |
| UJ-064 | 6 | Checkout blocked for empty cart. | `/cart` -> `/checkout` (blocked redirect) | Auth guard + cart/account APIs |
| UJ-065 | 6 | Checkout as guest if allowed by policy. | `/cart` -> `/checkout` (guest mode) | GraphQL checkout/cart/order endpoints |
| UJ-066 | 6 | Checkout requiring login (redirect path). | `/checkout` -> `/login` -> `/checkout` | Auth guard + cart/account APIs |
| UJ-067 | 6 | Select existing shipping address. | `/checkout/address` | GraphQL `createShippingAddress`/`updateShippingAddress`/`getShippingAddresses` |
| UJ-068 | 6 | Add new shipping address during checkout. | `/checkout/address` | GraphQL `createShippingAddress`/`updateShippingAddress`/`getShippingAddresses` |
| UJ-069 | 6 | Invalid address validation (postal code/required fields). | `/checkout/*` | GraphQL checkout/cart/order endpoints |
| UJ-070 | 6 | Duplicate/default address logic. | `/checkout/address` | GraphQL checkout/cart/order endpoints |
| UJ-071 | 6 | Checkout summary accuracy (subtotal/shipping/tax/discount/total). | `/checkout/review` | GraphQL cart/order preview queries |
| UJ-072 | 6 | Idempotent order placement (double-click Place Order). | `/checkout/review` -> place-order submit path | GraphQL `placeOrder` mutation (+ idempotency headers if used) |
| UJ-073 | 6 | Refresh on confirmation page. | `/checkout/confirmation` (refresh same route) | GraphQL checkout/cart/order endpoints |
| UJ-074 | 6 | Network loss after clicking Place Order. | `/checkout/review` -> place-order submit path | GraphQL `placeOrder` mutation (+ idempotency headers if used) |
| UJ-075 | 6 | Backend validation failure on Place Order. | `/checkout/review` -> place-order submit path | GraphQL `placeOrder` mutation (+ idempotency headers if used) |
| UJ-076 | 6 | Inventory changed between cart and place order. | `/checkout/review` -> place-order submit path | GraphQL `placeOrder` mutation (+ idempotency headers if used) |
| UJ-077 | 6 | Coupon application success. | `/checkout/review` | GraphQL `validateCoupon` query, `applyCoupon` mutation |
| UJ-078 | 6 | Coupon invalid/expired/min-order/not-eligible. | `/checkout/review` | GraphQL `validateCoupon` query, `applyCoupon` mutation |
| UJ-079 | 6 | Coupon removed/changed after apply. | `/checkout/review` | GraphQL `validateCoupon` query, `applyCoupon` mutation |
| UJ-080 | 6 | Retry place order after transient failure. | `/checkout/review` -> place-order submit path | GraphQL `placeOrder` mutation (+ idempotency headers if used) |
| UJ-081 | 6 | Post-order cart clear behavior. | `/checkout/confirmation` -> `/cart` (should be empty) | GraphQL `placeOrder`, `getCartItems` |
| UJ-082 | 7 | Create payment intent success. | `/checkout/payment` | GraphQL `createPaymentIntent` mutation |
| UJ-083 | 7 | Payment gateway popup success path. | `/checkout/payment` (gateway popup flow) | GraphQL payment + webhook endpoints |
| UJ-084 | 7 | User cancels payment popup. | `/checkout/payment` (gateway popup flow) | GraphQL payment + webhook endpoints |
| UJ-085 | 7 | Payment verification success. | `/checkout/payment` -> verify callback path | GraphQL `verifyRazorpayPayment` mutation |
| UJ-086 | 7 | Payment verification signature mismatch. | `/checkout/payment` -> verify callback path | GraphQL `verifyRazorpayPayment` mutation |
| UJ-087 | 7 | Duplicate verification idempotency. | `/checkout/payment` -> verify callback path | GraphQL `verifyRazorpayPayment` mutation |
| UJ-088 | 7 | Simulated async payment status arrives before client verify (mock-safe ordering). | `/checkout/payment` -> status refresh/reconciliation UI | GraphQL `verifyRazorpayPayment` mutation + order status query |
| UJ-089 | 7 | Simulated duplicate async payment status replay (mock-safe idempotency). | `/checkout/payment` -> status refresh/reconciliation UI | GraphQL `verifyRazorpayPayment` mutation + order status query |
| UJ-090 | 7 | Simulated payment amount mismatch state path (mock-safe). | `/checkout/payment` -> pending/needs-review messaging path | GraphQL `verifyRazorpayPayment` mutation + order status query |
| UJ-091 | 7 | Simulated payment currency mismatch state path (mock-safe). | `/checkout/payment` -> pending/needs-review messaging path | GraphQL `verifyRazorpayPayment` mutation + order status query |
| UJ-092 | 7 | Needs-review status path UX. | `/checkout/confirmation` or order status UI in `/account/orders/[id]` | GraphQL payment mutations + order status queries |
| UJ-093 | 7 | Payment success but order state not updated (recovery path). | `/checkout/confirmation` + `/account/orders/[id]` recovery path | GraphQL payment mutations + order status queries |
| UJ-094 | 7 | Payment failure and retry. | `/checkout/payment` (retry flow) | GraphQL payment mutations + order status queries |
| UJ-095 | 7 | Timeout from gateway. | `/checkout/payment` (retry flow) | GraphQL payment mutations + order status queries |
| UJ-096 | 7 | Refund initiation success. | `/account/orders/[id]` and/or `/admin/orders/[id]` | GraphQL/admin refund operations + core payment handlers |
| UJ-097 | 7 | Partial refund path. | `/account/orders/[id]` and/or `/admin/orders/[id]` | GraphQL/admin refund operations + core payment handlers |
| UJ-098 | 7 | Full refund path. | `/account/orders/[id]` and/or `/admin/orders/[id]` | GraphQL/admin refund operations + core payment handlers |
| UJ-099 | 7 | Refund in non-refundable state. | `/account/orders/[id]` and/or `/admin/orders/[id]` | GraphQL/admin refund operations + core payment handlers |
| UJ-100 | 8 | Login with valid credentials. | `/login` | Auth endpoints/middleware + account GraphQL/API routes |
| UJ-101 | 8 | Login with invalid credentials. | `/login` | Auth endpoints/middleware + account GraphQL/API routes |
| UJ-102 | 8 | OAuth login success. | `/login` -> `/auth/callback` -> target route | OAuth callback endpoints + GraphQL `authInfo`/identity query |
| UJ-103 | 8 | OAuth callback failure. | `/login` -> `/auth/callback` -> target route | OAuth callback endpoints + GraphQL `authInfo`/identity query |
| UJ-104 | 8 | Session expiry while logged in. | `/account/profile` or `/account/orders` (then redirect/login) | Auth endpoints/middleware + account GraphQL/API routes |
| UJ-105 | 8 | Logout and protected route access after logout. | `/account/profile` or `/account/orders` (then redirect/login) | Auth endpoints/middleware + account GraphQL/API routes |
| UJ-106 | 8 | Route requiring auth redirects correctly. | Protected route (`/account/*` or `/checkout`) -> `/login` | Auth endpoints/middleware + account GraphQL/API routes |
| UJ-107 | 8 | Access account APIs without auth (401 envelope). | `/account/*` (unauthorized API envelope path) | Account `/api/*` and/or GraphQL protected queries (401) |
| UJ-108 | 8 | CSRF disallowed origin rejection. | Browser mutation routes from allowed/disallowed origin (e.g., `/cart`, `/checkout`) | GraphQL mutation path with origin checks (`ALLOWED_ORIGINS`) |
| UJ-109 | 8 | CSRF allowed origin success. | Browser mutation routes from allowed/disallowed origin (e.g., `/cart`, `/checkout`) | GraphQL mutation path with origin checks (`ALLOWED_ORIGINS`) |
| UJ-110 | 8 | JWT malformed/expired token behavior. | Protected route/API with invalid token context | Auth middleware token verification path |
| UJ-111 | 8 | Account takeover-resistant error messaging (no user enumeration). | `/login` and password/account recovery surfaces | Auth + protected API/GraphQL endpoints |
| UJ-112 | 9 | View profile. | `/account/profile` | GraphQL `authInfo`/profile query+mutation (or account API equivalents) |
| UJ-113 | 9 | Edit profile success. | `/account/profile` | GraphQL `authInfo`/profile query+mutation (or account API equivalents) |
| UJ-114 | 9 | Edit profile validation errors. | `/account/profile` | GraphQL `authInfo`/profile query+mutation (or account API equivalents) |
| UJ-115 | 9 | View order history. | `/account/orders` | GraphQL `searchOrder`/orders query |
| UJ-116 | 9 | View order details access control (own order only). | `/account/orders/[orderId]` | GraphQL order detail + events queries |
| UJ-117 | 9 | View empty order history. | `/account/orders` | GraphQL `searchOrder`/orders query |
| UJ-118 | 9 | Save address from account. | `/account/addresses` | GraphQL shipping address queries/mutations |
| UJ-119 | 9 | Update/delete address from account. | `/account/addresses` | GraphQL shipping address queries/mutations |
| UJ-120 | 9 | Wishlist add/remove/view. | `/account/wishlist` (and product pages for add/remove) | GraphQL wishlist queries/mutations |
| UJ-121 | 9 | Wishlist persists across sessions. | `/account/wishlist` (and product pages for add/remove) | GraphQL wishlist queries/mutations |
| UJ-122 | 9 | Account API partial failure UX. | `/account/*` (degraded sections on partial API failures) | Account GraphQL/API endpoints |
| UJ-123 | 10 | Admin login success. | `/admin/login` | Admin GraphQL/API endpoints |
| UJ-124 | 10 | Non-admin user blocked from admin routes. | `/admin/*` as non-admin user | Admin GraphQL/API endpoints |
| UJ-125 | 10 | Admin product create/update/delete. | `/admin/products` and `/admin/products/[id]` | GraphQL admin product CRUD mutations/queries |
| UJ-126 | 10 | Admin product with invalid payload. | `/admin/products` and `/admin/products/[id]` | GraphQL admin product CRUD mutations/queries |
| UJ-127 | 10 | Admin category CRUD. | `/admin/categories` | GraphQL admin category CRUD (if schema-supported) |
| UJ-128 | 10 | Admin inventory adjust + inventory logs. | `/admin/inventory` and `/admin/inventory/logs` | GraphQL inventory item/log mutations/queries |
| UJ-129 | 10 | Admin order search/filter. | `/admin/orders` and `/admin/orders/[id]` | GraphQL admin order queries/mutations/events |
| UJ-130 | 10 | Admin mark shipped. | `/admin/orders` and `/admin/orders/[id]` | GraphQL admin order queries/mutations/events |
| UJ-131 | 10 | Admin mark delivered. | `/admin/orders` and `/admin/orders/[id]` | GraphQL admin order queries/mutations/events |
| UJ-132 | 10 | Illegal order state transition blocked. | `/admin/orders` and `/admin/orders/[id]` | GraphQL admin order queries/mutations/events |
| UJ-133 | 10 | Admin review moderation. | `/admin/reviews` | GraphQL review moderation operations |
| UJ-134 | 10 | Admin coupon create/update/deactivate. | `/admin/coupons` | GraphQL coupon operations |
| UJ-135 | 10 | Admin user role changes. | `/admin/users` | GraphQL/admin user role operations |
| UJ-136 | 10 | Admin transaction/refund visibility. | `/admin/transactions` and `/admin/refunds` | GraphQL/admin transaction/refund operations |
| UJ-137 | 10 | Admin API unauthorized and forbidden envelopes. | `/admin/*` API/GraphQL error envelope paths | Protected admin GraphQL/API envelopes (401/403) |
| UJ-138 | 10 | No browser direct privileged GraphQL from client components. | Client-side routes (`/admin/*`) must call server APIs, not privileged direct GraphQL | Frontend server API proxy routes (no direct privileged GraphQL from browser) |
| UJ-139 | 11 | Missing auth header on protected mutations. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Security checks on protected GraphQL/API endpoints |
| UJ-140 | 11 | Forged admin header attempts. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Security checks on protected GraphQL/API endpoints |
| UJ-141 | 11 | Privileged env leakage audit (NEXT_PUBLIC_*). | Frontend build/runtime surfaces (`/`, `/admin/*`) and bundled client env inspection | Frontend bundle/env inspection (`NEXT_PUBLIC_*` only) |
| UJ-142 | 11 | Rate limit normal user behavior. | Burst traffic against `/graphql` and high-traffic UI actions (`/products`, `/cart`, `/checkout`) | Global rate-limiter on `/graphql` and relevant `/api/*` routes |
| UJ-143 | 11 | Rate limit burst abuse. | Burst traffic against `/graphql` and high-traffic UI actions (`/products`, `/cart`, `/checkout`) | Global rate-limiter on `/graphql` and relevant `/api/*` routes |
| UJ-144 | 11 | Rate limit behind proxy headers. | Burst traffic against `/graphql` and high-traffic UI actions (`/products`, `/cart`, `/checkout`) | Global rate-limiter on `/graphql` and relevant `/api/*` routes |
| UJ-145 | 11 | Input injection-like payloads. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Security checks on protected GraphQL/API endpoints |
| UJ-146 | 11 | Large payload rejection. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Security checks on protected GraphQL/API endpoints |
| UJ-147 | 11 | GraphQL depth/complexity limit rejection. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | GraphQL depth/complexity guard on `/graphql` |
| UJ-148 | 11 | Unknown field/invalid argument error shape consistency. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Security checks on protected GraphQL/API endpoints |
| UJ-149 | 11 | Idempotency-key replay same payload. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Mutation endpoints honoring `Idempotency-Key` semantics |
| UJ-150 | 11 | Idempotency-key replay different payload conflict. | Security test paths on protected endpoints (`/graphql`, `/api/*`), usually from `/cart`, `/checkout`, `/admin/*` actions | Mutation endpoints honoring `Idempotency-Key` semantics |
| UJ-151 | 12 | GraphQL server restart during active session. | Active user flow routes (`/cart` -> `/checkout`) during service restart | `/graphql` availability/recovery path |
| UJ-152 | 12 | Core operations restart during checkout. | Active user flow routes (`/cart` -> `/checkout`) during service restart | `/graphql` upstream core_ops dependency path |
| UJ-153 | 12 | Redis unavailable. | Any data-driven route (`/`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) | Session/rate-limit/cache paths via `REDIS_URL` |
| UJ-154 | 12 | MySQL unavailable. | Any data-driven route (`/`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) | DB-backed core operations and GraphQL resolvers |
| UJ-155 | 12 | Slow DB responses. | Any data-driven route (`/`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) | DB-backed core operations and GraphQL resolvers |
| UJ-156 | 12 | Partial dependency failure (one service down). | Any data-driven route (`/`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) | Resilience across `/ready`, `/graphql`, and dependent services |
| UJ-157 | 12 | Retry behavior and user-visible messaging. | Storefront and checkout routes with retry/degraded UI states | Resilience across `/ready`, `/graphql`, and dependent services |
| UJ-158 | 12 | Background outbox processing lag. | `/checkout/confirmation`, `/account/orders/[id]`, `/admin/orders/[id]` (eventual consistency/recovery) | Webhook endpoints + outbox processors + order status queries |
| UJ-159 | 12 | Webhook delayed delivery. | `/checkout/confirmation`, `/account/orders/[id]`, `/admin/orders/[id]` (eventual consistency/recovery) | Webhook endpoints + outbox processors + order status queries |
| UJ-160 | 12 | Duplicate events handling. | `/checkout/confirmation`, `/account/orders/[id]`, `/admin/orders/[id]` (eventual consistency/recovery) | Webhook endpoints + outbox processors + order status queries |
| UJ-161 | 12 | Stuck "processing" order recovery procedure. | `/checkout/confirmation`, `/account/orders/[id]`, `/admin/orders/[id]` (eventual consistency/recovery) | Webhook endpoints + outbox processors + order status queries |
| UJ-162 | 12 | Graceful degradation for storefront reads. | Storefront and checkout routes with retry/degraded UI states | Resilience across `/ready`, `/graphql`, and dependent services |
| UJ-163 | 13 | First load performance on mobile. | `/` and critical journey routes (`/products/[slug]`, `/cart`, `/checkout`) | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-164 | 13 | Revisit/cached navigation speed. | `/` and critical journey routes (`/products/[slug]`, `/cart`, `/checkout`) | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-165 | 13 | Large catalog page performance. | `/products` | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-166 | 13 | Image-heavy PDP performance. | `/products/[slug]` | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-167 | 13 | Interaction latency for filter/sort. | `/products` | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-168 | 13 | API waterfall avoidance in critical paths. | Critical paths: `/`, `/products/[slug]`, `/checkout` | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-169 | 13 | Layout shift checks on load. | `/` and critical journey routes (`/products/[slug]`, `/cart`, `/checkout`) | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-170 | 13 | Slow 3G behavior for key journeys. | `/` and critical journey routes (`/products/[slug]`, `/cart`, `/checkout`) | Critical route requests: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, and GraphQL/API calls behind them |
| UJ-171 | 14 | Keyboard-only navigation for all primary flows. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-172 | 14 | Focus management on modals/drawers. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-173 | 14 | Focus visible styles. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-174 | 14 | Form labels/aria validity. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-175 | 14 | Screen-reader meaningful announcements for errors/loading. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-176 | 14 | Color contrast checks. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-177 | 14 | Error states accessible messaging. | Accessibility coverage across all primary flows | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-178 | 14 | Skip links and heading structure. | All primary routes: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-179 | 14 | Mobile zoom/readability. | Mobile viewport on primary routes | All user-facing routes (`/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*`) |
| UJ-180 | 15 | Indexable storefront pages. | `/`, `/products`, `/products/[slug]`, category pages | Storefront route rendering + metadata endpoints |
| UJ-181 | 15 | Correct canonical tags. | Storefront indexable pages (`/products`, `/products/[slug]`, category pages) | Route metadata generation on storefront/PDP routes |
| UJ-182 | 15 | robots/sitemap endpoints. | `/robots.txt`, `/sitemap.xml` | `GET /robots.txt`, `GET /sitemap.xml` |
| UJ-183 | 15 | Structured metadata for products. | `/products/[slug]` | Route metadata generation on storefront/PDP routes |
| UJ-184 | 15 | 404/500 pages index handling. | `/404`, `/500` (or framework equivalents) | Storefront route rendering + metadata endpoints |
| UJ-185 | 15 | Share preview metadata on PDP. | `/products/[slug]` | Route metadata generation on storefront/PDP routes |
| UJ-186 | 16 | Chrome desktop. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-187 | 16 | Safari desktop. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-188 | 16 | Firefox desktop. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-189 | 16 | Edge desktop. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-190 | 16 | iOS Safari. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-191 | 16 | Android Chrome. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-192 | 16 | Narrow viewport (320px). | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-193 | 16 | Tablet viewport. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-194 | 16 | Dark/light mode if supported. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |
| UJ-195 | 16 | Locale/currency formatting consistency. | Run the same core paths on each device/browser: `/`, `/products`, `/products/[slug]`, `/cart`, `/checkout`, `/account/*`, `/admin/*` | Same endpoints as core flows, validated across browser/device matrix |

| UJ-196 | 3 | The Fabric Deep-Dive: User interacts with high-res zoom or fabric video. | `/products/[slug]` (interaction with media zoom/video components) | Product detail query + media asset delivery/CDN endpoints (image/video) |
| UJ-197 | 9 | The Custom Measurements Vault: User saves specific body dimensions for tailoring. | `/account/measurements` | Account profile/measurements API or GraphQL mutation path (tailoring profile data) |
| UJ-198 | 6 | The Hybrid Payment (COD + Online): If you offer partial advance for custom orders. | `/checkout/payment` (split-logic path) | Checkout payment orchestration endpoints (create intent + COD split business rules) |
| UJ-199 | 1 | The Direct-to-WhatsApp Flow: Premium consultation entry point. | `/` or `/products/[slug]` -> external WhatsApp redirect | Frontend CTA route + external WhatsApp deep-link redirect (no privileged backend required) |
| UJ-200 | 10 | The Lookbook/Editorial CMS Path: Admin updating high-fashion imagery. | `/admin/lookbooks/new` or `/admin/lookbooks/[id]` | Admin media/content CRUD endpoints (lookbook/editorial assets) |
| UJ-201 | 11 | gRPC Timeout Recovery: Backend is slow; Next.js BFF must show a specific "Retry" UI. | `/products/[slug]` (simulate backend delay path) | Next.js BFF route -> GraphQL `/v2` -> gRPC core_operations (timeout + retry UX path) |
| UJ-202 | 8 | JWT Handover: Ensure the JWT minted by Rust is correctly handled by NextAuth in the frontend. | `/login` -> `/profile` (token persistence check) | NextAuth callbacks/session routes + GraphQL `authInfo`/protected account endpoints |
| UJ-203 | 12 | BFF-to-GraphQL/gRPC contract mismatch handling (frontend does not call protobuf directly). | `/api/checkout` (validation error path) | Next.js `/api/checkout/*` -> GraphQL `/v2`/gRPC contract validation and error envelope |
| UJ-204 | 8 | Phone OTP request success via login dialog. [Deferred - do not implement now] | Login dialog -> `/api/auth/phone-otp/request` | Next.js OTP request route -> GraphQL `/auth/phone-otp/request` |
| UJ-205 | 8 | Phone OTP request failure (invalid phone/upstream OTP error). [Deferred - do not implement now] | Login dialog -> `/api/auth/phone-otp/request` (error UI) | Next.js OTP request route 400/5xx envelope + inline error messaging |
| UJ-206 | 8 | Phone OTP verification success creates authenticated session. [Deferred - do not implement now] | Login dialog -> NextAuth credentials (`phone-otp`) -> callback -> return route | NextAuth `[...nextauth]` + backend OTP verify + customer identity sync |
| UJ-207 | 8 | Phone OTP verification failure (wrong/expired code). [Deferred - do not implement now] | Login dialog verify action (stay on same modal) | NextAuth credentials authorize failure -> UI alert + telemetry failure event |
| UJ-208 | 8 | Auth capabilities endpoint reports guest/customer/admin mode correctly. | Any route checking capabilities -> `/api/auth/capabilities` | Capabilities route + NextAuth session + admin allowlist evaluation |
| UJ-209 | 10 | Admin middleware redirect: unauthorized user is redirected to /imtheboss/login?error=AccessDenied. | `/imtheboss/*` as non-admin -> redirected to `/imtheboss/login?error=AccessDenied` | `src/middleware.ts` + NextAuth JWT + admin allowlist |
| UJ-210 | 10 | Admin telemetry summary visible to admin role. | `/imtheboss` dashboard observability widgets | `/api/telemetry/summary` (authorized) + metrics fan-in from GraphQL/core_ops |
| UJ-211 | 10 | Admin telemetry endpoints reject non-admin access with 401 envelope. | Non-admin call to `/api/telemetry/events` or `/api/telemetry/summary` | Telemetry routes with `getAdminSession()` guard -> 401 envelope |
| UJ-212 | 12 | Telemetry summary degrades gracefully when backend Prometheus metrics are unavailable. | `/imtheboss` observability panel with backend metrics down | `/api/telemetry/summary` fallback path returns available=false but usable payload |
| UJ-213 | 11 | Server-only admin GraphQL proxy prevents INTERNAL_API_SECRET leakage to browser. | Browser admin action -> `/api/admin/graphql` only | Server-only `admin-graphql-server` path; secret remains server-side |
| UJ-214 | 5 | Guest cart merge into authenticated cart after login. | Auth sync flow -> `/api/account/cart/merge` | Merge route reads guest+customer cart, merges quantities, deletes guest rows |
| UJ-215 | 5 | Guest cart merge idempotency replay is safe. | Repeat `/api/account/cart/merge` with same idempotency key | Merge route uses deterministic/replayed keys for add/update/delete operations |
| UJ-216 | 5 | Guest cart merge failure when guestSessionId is missing/invalid. | `/api/account/cart/merge` with missing/invalid body | Merge route returns 400 validation envelope; no partial cart corruption |
| UJ-217 | 3 | PDP notFound path for invalid product id. | Direct open `/product/[id]` with invalid id | Product page `notFound()` path + 404 UX |
| UJ-218 | 2 | Collection/category page notFound path for unknown slug/id. | Direct open `/collections/[slug]` or `/category/[categoryId]` invalid | Collection/category loaders -> `notFound()` path + 404 UX |
| UJ-219 | 9 | Stale/unauthorized account route shows re-auth CTA and safe fallback. | `/profile` or `/checkout/address` while auth stale | Route failure mapping (`toRouteFailureUi`) + login prompt path |
| UJ-220 | 12 | 429/network retry path via BFF resilience wrapper without duplicate side effects. | Data actions through `/api/*` under transient failures | `fetchWithResilience` retry behavior + idempotent checkout/cart write safeguards |
Total journey rows in matrix: 220
