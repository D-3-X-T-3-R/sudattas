# Configuration & Deployment

## Environment variables

See [.env.example](.env.example) for the full list of required and optional variables (database, gRPC, OAuth, Redis, R2, GraphQL bind address, logging).

**Production:** Do not use example or default secrets in production. Override all sensitive values via environment variables or a secret manager (e.g. Kubernetes secrets, AWS Secrets Manager). The values in `.env.example` are placeholders only.

### Admin authorization (GraphQL resolver layer)

- `ADMIN_ALLOWED_USER_IDS`: comma-separated JWT user ids allowed to execute admin-only GraphQL operations.
- Empty or unset means no backend admin privileges are granted.
- This is an explicit temporary allowlist control until role-based backend authorization is fully wired through `user_roles`.

## Health endpoints (GraphQL service)

For orchestrators (e.g. Kubernetes):

- **Liveness — `GET /`**  
  Returns 200 if the process is running. Use for restart decisions (e.g. `livenessProbe`). No dependency checks.

- **Readiness — `GET /ready`**  
  Returns 200 if the service can serve traffic: checks gRPC backend (and thus DB, via the gRPC Readiness RPC) and, if `REDIS_URL` is set, Redis. Returns 503 if any configured check fails. Use for traffic routing (e.g. `readinessProbe`) so the pod is not sent requests until dependencies are up.

### Rollout gating order (backend -> frontend)

- Deploy or restart backend services first (`core_operations`, `graphql`, Redis/MySQL dependencies).
- Gate traffic shift on `GET /ready` returning 200 for GraphQL.
- Only after backend readiness is green, deploy/shift frontend (Next.js) traffic.
- If backend readiness is red, block frontend rollout and keep prior frontend serving.

## Bind addresses

- **GraphQL:** `GRAPHQL_LISTEN_ADDR` (default `0.0.0.0:8080`).
- **gRPC (core_operations):** `GRPC_SERVER` (default `0.0.0.0:50051`).

## Metrics and resilience

- **Prometheus:** `GET /metrics` — scrape endpoint for request counts and latency. See [RESILIENCE.md](RESILIENCE.md) for rate limiting, gRPC timeouts/retries/circuit breaker, and webhooks.
- If GraphQL is behind a trusted proxy/app server, set `RATE_LIMIT_TRUST_PROXY_HEADERS=true` so rate limiting keys on end-user IP (`X-Forwarded-For` / `X-Real-Ip`) instead of the proxy socket IP.

## Webhook endpoints (GraphQL service)

- **Razorpay webhook:** `POST /wheresthemoney/razorpay`  
  Requires valid `x-razorpay-signature` when `RAZORPAY_WEBHOOK_SECRET` is set.
- **Shiprocket webhook:** `POST /blastoff/parcelupdate`  
  Set `SHIPROCKET_WEBHOOK_SECRET` and send it as `x-shiprocket-token`.

## Cross-layer route contract

Frontend and backend boundary rules are documented in [`../docs/CROSS_LAYER_CONTRACT.md`](../docs/CROSS_LAYER_CONTRACT.md).
Staging/production security parity rules are documented in [`../docs/STAGING_PRODUCTION_PARITY.md`](../docs/STAGING_PRODUCTION_PARITY.md).

This includes:

- Route families (`public storefront`, `authenticated customer`, `admin`)
- Header ownership and forbidden browser credential patterns
- Idempotency requirements for money-moving flows
- Expected frontend-consumed error shape targets
