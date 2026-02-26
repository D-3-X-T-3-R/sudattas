# Resilience & Operations

## Rate limiting (GraphQL)

- **Per-IP** rate limiting is applied to GraphQL and webhook routes (not to GET `/`, GET `/ready`, or OPTIONS).
- Configure via env: `RATE_LIMIT_PER_MINUTE` (default `60`). Set to `0` to disable.
- When exceeded: HTTP **429 Too Many Requests**.

## gRPC client (GraphQL → gRPC)

- **Timeout:** Each gRPC request has a timeout (default 30s). Configure via `GRPC_REQUEST_TIMEOUT_SEC`.
- **Connect timeout:** Connection attempt timeout (default 10s). Configure via `GRPC_CONNECT_TIMEOUT_SEC`.
- **Retry:** On connection failure, the client retries up to `GRPC_CONNECT_RETRIES` (default 2) with 1s backoff.
- **Circuit breaker:** After repeated gRPC connection failures, the client stops attempting for a cooldown period (default 30s). Configure via `GRPC_CIRCUIT_BREAKER_FAILURES` (default 5) and `GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC` (default 30).

## Metrics (Prometheus)

- **Endpoint:** `GET /metrics` — Prometheus scrape format.
- **Metrics:** `graphql_requests_total` (counter by method, path, status); `graphql_request_duration_seconds` (histogram).
- Health and readiness endpoints are excluded from request metrics.

## Request / trace IDs

- Each request gets a `request_id` (UUID) in the tracing span. Use it for log correlation.
- When distributed tracing is enabled, the same ID can be propagated to gRPC metadata.

## Webhooks

- **Receiver:** POST `/webhook/:provider` (e.g. `razorpay`). Signature verification (e.g. `x-razorpay-signature`) when `RAZORPAY_WEBHOOK_SECRET` is set.
- **Idempotency:** By `webhook_id` (derived from payload); duplicate deliveries return stored response.
- **Storage:** All events stored in `webhook_events`; payment callbacks (e.g. `payment.captured`) trigger capture and status update.

## Idempotency (mutations)

- **place_order** and **capture_payment** accept optional header `Idempotency-Key`. When set, the key is stored with the response; repeated requests with the same key within the configured window return the cached response.
- Window: 24 hours (configurable via `IDEMPOTENCY_WINDOW_HOURS`).

## Structured logging

- Logs are JSON (tracing-subscriber). Include `request_id` and span context for production aggregation.
