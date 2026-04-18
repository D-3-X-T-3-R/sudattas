# Environment Variables Contract

This document defines public-safe vs server-only variables and startup validation rules across frontend and backend services.

## Frontend (Next.js)

### Public-safe (`NEXT_PUBLIC_*`)

- `NEXT_PUBLIC_GRAPHQL_URL` (default `http://localhost:8080/v2`)
- `NEXT_PUBLIC_SITE_URL` (optional; canonical metadata/sitemap)
- `NEXT_PUBLIC_STORE_URL` (optional; admin "Back to store" link)
- `NEXT_PUBLIC_IMAGE_HOST` (optional; Next image remote host)
- `NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID` (optional numeric)
- `NEXT_PUBLIC_PHONE_OTP_CHANNEL` (optional: `sms` or `whatsapp`)

### Server-only

- `GRAPHQL_URL` (optional server override for Next API/auth)
- `GRAPHQL_METRICS_URL` (optional override for GraphQL metrics scrape URL; default `${GRAPHQL_URL base}/metrics`)
- `CORE_OPS_METRICS_URL` (optional override for core operations metrics URL; default `http://127.0.0.1:9090/metrics`)
- `STOREFRONT_ORIGIN` (optional origin for session-auth GraphQL calls)
- `AUTH_SECRET` (required in production)
- `NEXTAUTH_URL` (required in production)
- `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` (required for Google sign-in)
- `ADMIN_ALLOWED_EMAILS` (required for restricted admin access)

### Validation

- Frontend environment is validated via:
  - `frontend/src/lib/env/public.ts`
  - `frontend/src/lib/env/server.ts`
- Invalid values fail fast at app startup/build.

## Backend: GraphQL service

### Runtime variables

- `GRAPHQL_LISTEN_ADDR` (default `0.0.0.0:8080`)
- `REDIS_URL` (optional; required for guest sessions)
- `ALLOWED_ORIGINS` (optional comma-separated allowlist)
- `RAZORPAY_WEBHOOK_SECRET` (required when webhook secret enforcement is enabled; validates `x-razorpay-signature` on `POST /wheresthemoney/razorpay`)
- `SHIPROCKET_WEBHOOK_SECRET` (required when webhook secret enforcement is enabled; validates `x-shiprocket-token` on `POST /blastoff/parcelupdate`)
- `REQUIRE_WEBHOOK_SECRETS` (default `true` when `APP_ENV`, `RUST_ENV`, or `NODE_ENV` is `production`, otherwise `false`; when enabled, GraphQL startup fails if webhook secrets are missing)
- `RATE_LIMIT_PER_MINUTE` (default `240`)
- `RATE_LIMIT_WEBHOOK_PER_MINUTE` (default `120`)
- `RATE_LIMIT_TRUST_PROXY_HEADERS` (default `false`)

### Validation

- Validated at startup via `backend/graphql/src/startup_config.rs`.
- Invalid values fail startup with explicit error messages.

## Backend: Core operations service

### Runtime variables

- `GRPC_SERVER` (default `0.0.0.0:50051`)
- `GRPC_METRICS_ADDR` (default `0.0.0.0:9090`)
- `SHIPROCKET_EMAIL` / `SHIPROCKET_PASSWORD` (required to book shipments via Shiprocket from admin mark-shipped flow)
- `SHIPROCKET_PICKUP_LOCATION` (optional; default `Primary`)
- `SHIPROCKET_PICKUP_POSTCODE` (required for checkout quote-based shipping calculation)
- `SHIPROCKET_DEFAULT_WEIGHT_KG`, `SHIPROCKET_ESTIMATED_UNIT_WEIGHT_KG`, `SHIPROCKET_PACKAGE_LENGTH_CM`, `SHIPROCKET_PACKAGE_BREADTH_CM`, `SHIPROCKET_PACKAGE_HEIGHT_CM` (optional package/weight defaults)
- `SHIPROCKET_COURIER_ID` (optional fixed courier override)

### Validation

- Validated at startup via `backend/core_operations/src/startup_config.rs`.
- Invalid values fail startup with explicit error messages.

## Production vs development

- Production: explicitly set all server-only auth/security variables and bind addresses.
- Development: defaults are allowed for local endpoints and ports where noted.
