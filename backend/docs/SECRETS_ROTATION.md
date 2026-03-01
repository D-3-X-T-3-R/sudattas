# P2 Security: Secrets rotation process and audit

This document describes how to rotate secrets used by the backend and how to record rotation in the audit log.

## Secrets to rotate

| Secret | Where used | Rotation steps |
|--------|------------|----------------|
| **DATABASE_URL** | core_operations, graphql (gRPC client) | Update env/secret in deployment; restart services. No application-level rotation. |
| **REDIS_URL** | graphql (sessions, idempotency), core_operations (session manager) | Update env; restart. Optionally drain sessions before switch. |
| **JWT / JWKS** | graphql (auth) | Update JWKS URL or keys in IdP; set `GRAPHQL_JWKS_URL` if needed; restart graphql. |
| **R2_*** (S3-compat)** | core_operations (image upload) | Rotate R2 API token in Cloudflare; set `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`; restart. |
| **Webhook signing secrets** | graphql (Razorpay etc.) | Update secret in provider and in env (e.g. `RAZORPAY_WEBHOOK_SECRET`); restart. |

## Audit after rotation

After rotating any of the above:

1. **Record the event** so there is an audit trail. Use the gRPC method `RecordSecurityAuditEvent` with:
   - `event_type`: e.g. `secrets_rotation`
   - `details`: optional short note (e.g. "REDIS_URL rotated", "R2 keys rotated").

2. **Where to call it**: From an admin script or CI that has gRPC access to core_operations, or by inserting a row into `security_audit_log` directly:
   ```sql
   INSERT INTO security_audit_log (event_type, details) VALUES ('secrets_rotation', 'REDIS_URL rotated');
   ```

3. **Review**: Periodically review `security_audit_log` (e.g. by `event_type`, `created_at`) to confirm rotations and access.

## Optional: restrict gRPC for audit

`RecordSecurityAuditEvent` is exposed on the same gRPC service as other operations. Restrict who can call it (e.g. admin-only or internal service account) via your gRPC auth/authorization.
