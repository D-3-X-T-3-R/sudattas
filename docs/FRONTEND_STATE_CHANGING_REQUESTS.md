# Frontend State-Changing Requests

This inventory lists frontend write paths so CSRF/auth/idempotency hardening can be tracked in one place.

## Public / Storefront

- `POST /session/guest` (via `src/lib/session.ts` and `src/lib/server-guest-session.ts`)
  - Purpose: create/refresh guest session.
  - Auth mode: guest bootstrap.
- `POST /v2` GraphQL storefront mutations (via `src/lib/graphqlWithSession.ts` and `src/lib/graphqlClient.ts`)
  - Purpose: cart, checkout, and customer write operations.
  - Auth mode: bearer token or guest session id.

## Account (Next API routes)

- `POST /api/account/addresses`
  - Purpose: create shipping address.
  - Auth mode: server-resolved customer session token.
- `PATCH /api/account/addresses`
  - Purpose: update shipping address.
  - Auth mode: server-resolved customer session token.
- `DELETE /api/account/addresses`
  - Purpose: delete shipping address.
  - Auth mode: server-resolved customer session token.

## Admin (Next API routes, server-only auth)

- `POST /api/admin/products`
  - Purpose: admin product/catalog/image/inventory mutations and scoped reads.
- `POST /api/admin/orders`
  - Purpose: admin order reads and state transitions.
- `POST /api/admin/customers`
  - Purpose: admin customer/user operations.
- `POST /api/admin/reviews`
  - Purpose: review moderation operations.
- `POST /api/admin/shipments`
  - Purpose: shipment/order-fulfillment state changes.
- `POST /api/admin/graphql` (compatibility fallback)
  - Purpose: controlled migration fallback for admin operations.

All admin routes enforce session and admin allowlist on the server, then attach server-only credentials for backend GraphQL calls.

