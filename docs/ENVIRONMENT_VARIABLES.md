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

### Validation

- Validated at startup via `backend/core_operations/src/startup_config.rs`.
- Invalid values fail startup with explicit error messages.

## Production vs development

- Production: explicitly set all server-only auth/security variables and bind addresses.
- Development: defaults are allowed for local endpoints and ports where noted.
