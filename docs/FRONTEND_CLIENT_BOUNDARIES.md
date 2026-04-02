# Frontend Client Boundaries

This file defines which client to use for each traffic family.

## 1) Public reads

- Client: `src/lib/graphqlClient.ts` or storefront `/api/products` routes.
- Auth: guest session id only (or no auth for safe reads).
- Rule: never include privileged admin headers.

## 2) Authenticated customer actions

- Client: `src/lib/graphqlWithSession.ts` for browser calls with bearer/session, plus `/api/account/*` server routes where session ownership must be enforced.
- Auth: customer session token resolved from login state.
- Rule: customer writes should flow through account routes when identity scoping is required.

## 3) Admin operations

- Client: `src/lib/graphqlAdmin.ts`.
- Transport: browser always calls `/api/admin/*`; server routes call backend GraphQL.
- Auth: admin identity resolved server-side (`src/lib/admin-auth-server.ts` + middleware).
- Rule: no browser code constructs admin credentials directly.

