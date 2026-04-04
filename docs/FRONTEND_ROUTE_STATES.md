# Frontend Route States

This document defines the canonical UI states for critical route families.

## Public pages

- `loading`: data request in flight.
- `empty`: request succeeded but list/detail has no content.
- `unauthorized`: session missing/expired where guest or session auth was expected.
- `stale`: stale guest/session token detected; service layer attempts recovery.
- `retryable_error`: transient network/429/5xx failure; show retry guidance.
- `fatal_error`: non-retryable failure after normalization.

## Account pages

- `loading`: profile/account fetch in progress.
- `empty`: valid user with no addresses/orders yet.
- `unauthorized`: signed-out/expired auth; prompt sign-in.
- `stale`: session/account token stale; prompt sign-in refresh.
- `retryable_error`: temporary service issue; show retry guidance.
- `fatal_error`: non-retryable account error.

## Admin pages

- `loading`: admin query in progress.
- `empty`: no rows for current filters.
- `unauthorized`: admin session/permissions invalid.
- `stale`: admin session stale/expired.
- `retryable_error`: temporary upstream issue.
- `fatal_error`: non-retryable admin error.

## Rule

- Components render state UI only.
- Service-layer helpers own retry, rate-limit backoff, stale-session recovery, and error normalization.
- Do not render raw backend/internal error strings directly to users on critical pages.

## Degraded-state UX actions

- `retryable_error`: show a retry action (`Retry`) on affected route cards/banners.
- `unauthorized` or `stale`: show a re-auth action (`Sign in again`) that routes through the existing login flow.
- `fatal_error`: show a stable fallback message without exposing internal backend details.
