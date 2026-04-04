# Frontend Telemetry

Structured client error telemetry is captured in the frontend and stored server-side for admin querying.

## What is captured

- `route` and `action`
- `userMode` (`public`, `account`, `admin`)
- `errorClass` (`unauthorized`, `validation`, `retryable`, `network`, `fatal`, `boundary`)
- backend-facing `errorCode` when available
- `status`, `message`, `requestId`
- device/network context (`online`, `userAgent`, `effectiveType`, `downlink`, `rtt`)

## Ingestion endpoint

- `POST /api/telemetry/events`
- Payload is validated by schema in:
  - `frontend/src/app/api/telemetry/events/route.ts`

## Query endpoint (admin-only)

- `GET /api/telemetry/events?limit=100&mode=admin&errorClass=retryable&q=/api/checkout`
- Requires admin session (`ADMIN_ALLOWED_EMAILS` policy).

## Summary endpoint (admin-only)

- `GET /api/telemetry/summary`
- Returns 24-hour operational aggregates:
  - login failure rate
  - cart conversion drop-off
  - checkout failure rate
  - payment mismatch rate
  - admin action failure rate
  - release confidence score
  - webhook processing latency (from Rust backend metrics)

The summary route also reads backend Prometheus metrics from GraphQL and core operations
to combine frontend + backend signals for checkout/payment/admin reliability.

Supported query params:

- `limit` (1-500)
- `mode` (`public|account|admin`)
- `errorClass` (`unauthorized|validation|retryable|network|fatal|boundary`)
- `errorCode` (exact match)
- `q` (substring search across route/pageRoute/action)

## Storage

- File: `frontend/.telemetry/client-events.ndjson`
- One JSON event per line.
- Intended for single-node deployments and operational triage.
