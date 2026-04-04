# Stack Architecture Diagram

Last Updated: 2026-04-04

## High-Level Flow

```text
[Browser]
   |
   | HTTPS
   v
[Next.js Frontend (App Router)]
   |\
   | \-- UI routes: /, /product/*, /bag, /checkout, /profile, /imtheboss
   |
   +--> [Next.js API Routes (/api/*)]  (BFF boundary)
          |
          | server-to-server HTTP
          v
       [GraphQL Service (Rust, warp + juniper)]
          |\
          | +-- /v2 (GraphQL endpoint)
          | +-- /ready (readiness)
          | +-- /session/guest
          | +-- /webhook/{provider}
          | +-- /metrics
          |
          +--> [gRPC: core_operations (Rust, tonic)]
                 |
                 +--> [MySQL] (SeaORM data access)
                 +--> [Redis] (session/rate-limit/cache)
                 +--> [Razorpay/Twilio/R2 integrations]
```

## Repository Layout

- `frontend/` -> Next.js 16 + React 19 + TypeScript
- `backend/graphql/` -> HTTP GraphQL gateway
- `backend/core_operations/` -> gRPC business operations
- `backend/core_db_entities/` -> SeaORM entities
- `backend/protos/` -> protobuf contracts
- `backend/database/` -> schema + migrations + DB docker image

## Runtime Surface

### Frontend
- User routes: `/`, `/product/*`, `/bag`, `/checkout`, `/profile`, `/imtheboss/*`
- API routes (BFF): `/api/account/*`, `/api/admin/*`, `/api/checkout/*`, `/api/auth/*`, `/api/products/*`, etc.

### Backend GraphQL Service
- `POST /v2` (GraphQL)
- `GET /ready`
- `POST /session/guest`
- `POST /webhook/{provider}`
- `GET /metrics`
- `GET /robots.txt`, `GET /sitemap.xml`

### Backend gRPC Service
- gRPC server on `GRPC_SERVER` (default `0.0.0.0:50051`)
- Prometheus metrics endpoint (default `0.0.0.0:9090/metrics`)

## Data + State

- Primary DB: MySQL (`SUDATTAS` schema)
- Ephemeral/state infra: Redis
- Auth/session modes used in backend:
  - JWT auth
  - Session ID (`GRAPHQL_SESSION_ID`) for guest/session flows
  - Internal service auth (`INTERNAL_API_SECRET`) for trusted server paths

## Integrations

- Payments: Razorpay (intent/capture/verify/webhook)
- OTP: Twilio Verify
- Media storage: Cloudflare R2 (S3-compatible)

## Local Development Topology

`docker-compose.yml` starts:
- `mysql` (3306)
- `redis` (6379)
- `core_operations` (50051)
- `graphql` (8080)

Frontend usually runs separately via `npm run dev` on `3000`.

## CI Topology (GitHub Actions)

- `frontend-ci.yml`: lint + unit tests + build + perf/contract checks
- `backend-ci.yml`: fmt/clippy + unit/integration/e2e + coverage + security + build matrix
- `fullstack-smoke.yml`: boots backend + frontend and runs smoke checks
- `env-safety.yml`: blocks privileged `NEXT_PUBLIC_*` leaks

## Core Environment Variables (Cross-Cutting)

- Backend infra:
  - `DATABASE_URL`
  - `REDIS_URL`
  - `GRPC_SERVER`
  - `GRPC_URL`
  - `GRAPHQL_LISTEN_ADDR`
- Auth/security:
  - `INTERNAL_API_SECRET`
  - `OAUTH_DOMAIN`
  - `OAUTH_AUDIENCE`
  - `ALLOWED_ORIGINS`
- Frontend:
  - `NEXT_PUBLIC_GRAPHQL_URL`
  - `GRAPHQL_URL` (server-only override)
  - NextAuth vars (`AUTH_SECRET`, provider creds)

## Notes

- Current repo has Vitest tests for frontend, but no Playwright config committed yet.
- GraphQL auth precedence and CSRF/rate-limit behavior materially affect E2E outcomes.
