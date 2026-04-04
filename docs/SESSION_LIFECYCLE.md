# Session Lifecycle Contract

This document defines a single cross-layer session lifecycle for guest and authenticated customers.

## 1) Guest session creation

- Trigger: first storefront visit without JWT/session token.
- Frontend call: `POST /session/guest` (through `frontend/src/lib/session.ts` or `frontend/src/lib/server-guest-session.ts`).
- Result: backend returns `session_id`; frontend stores it and sends `X-Session-Id` on storefront reads.

## 2) Guest session expiration

- Backend authority: Redis TTL and validation in backend session validator.
- Frontend behavior: if storefront/account call reports invalid/expired session, frontend clears cached guest session and mints a new one.
- No admin fallback is allowed for expired guest sessions.

## 3) Login conversion (guest -> customer)

- Trigger: successful OTP/Google sign-in.
- Frontend behavior: customer flows switch to JWT (`Authorization: Bearer ...`) as primary auth.
- Guest session remains non-authoritative for customer-owned resources.

## 4) Cart merge

- Policy: merge logic is backend-owned.
- Frontend sends customer-authenticated cart operations after login; backend resolves guest/customer cart consolidation rules.
- Frontend must not implement a conflicting merge algorithm in component state.

## 5) Logout

- Trigger: `signOut()` from NextAuth.
- Frontend behavior:
  - clear customer-authenticated UI state,
  - return to guest mode,
  - ensure guest session exists for storefront browsing.

## 6) Session refresh

- NextAuth handles token/session refresh for authenticated users.
- Frontend should treat refreshed session as same canonical customer identity and avoid auth-provider specific branching.

## 7) Non-ambiguous persistence rule

- Durable customer data (addresses, orders, account views) must be fetched from backend after auth state changes.
- Cart is backend-authoritative; frontend must not use local-only cart fallback as alternate persistence.
- UI must not depend on stale local-only state for identity-scoped data.
