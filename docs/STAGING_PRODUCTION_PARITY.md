# Staging/Production Parity Rules

This project requires staging to enforce the same auth/payment/origin security behavior as production.

## Why

- Prevent false confidence from permissive staging behavior.
- Catch auth/origin/payment misconfiguration before production rollout.
- Keep E2E and smoke validations meaningful.

## Scope

Parity checks focus on security-sensitive config, not hostnames or credentials.

### Frontend/Next.js parity keys

- `GRAPHQL_URL`
- `STOREFRONT_ORIGIN`
- `ADMIN_ALLOWED_EMAILS`

### Backend GraphQL parity keys

- `ALLOWED_ORIGINS`
- `RATE_LIMIT_TRUST_PROXY_HEADERS`
- `RATE_LIMIT_PER_MINUTE`
- `RATE_LIMIT_WEBHOOK_PER_MINUTE`
- `GRAPHQL_MAX_QUERY_DEPTH`
- `GRAPHQL_MAX_QUERY_COMPLEXITY`

### Backend Core Operations parity keys

- `GRPC_AUTH_TOKEN` (must be set in both)

### Presence-only secrets (must exist in both envs)

- `AUTH_SECRET`
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `RAZORPAY_KEY_ID`
- `RAZORPAY_KEY_SECRET`
- `RAZORPAY_WEBHOOK_SECRET`

## Automated check

Use scripts:

- Bash: `scripts/check-staging-prod-parity.sh`
- PowerShell: `scripts/check-staging-prod-parity.ps1`

Default env files:

- `backend/.env.staging`
- `backend/.env.production`
- `frontend/.env.staging`
- `frontend/.env.production`

Override paths via script arguments.

## Release gate

Before promoting staging -> production:

1. Run parity script.
2. Ensure zero parity failures.
3. Ensure staging smoke tests pass under staging config.

