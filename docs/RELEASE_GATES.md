# Release Gates

This project uses explicit scripts to enforce core release gates before production promotion.

## Scripts

- Bash: `scripts/run-release-gates.sh`
- PowerShell: `scripts/run-release-gates.ps1`

### Included gates

1. No exposed privileged public env vars:
   - runs `scripts/check-no-privileged-public-env.(sh|ps1)`
2. Backend readiness endpoint is healthy:
   - default URL: `http://127.0.0.1:8080/ready`
3. Admin authorization tests pass:
   - `graphql_tests::test_admin_mutation_requires_admin_authorization`
   - `graphql_tests::test_search_user_requires_admin_authorization`
4. Payment negative-path tests pass:
   - `handler_payment_intents::verify_razorpay_payment_missing_fields_returns_invalid_argument`
   - `handler_payment_intents::verify_razorpay_payment_not_configured_returns_failed_precondition`

## CI enforcement

- `.github/workflows/env-safety.yml` enforces privileged `NEXT_PUBLIC_*` leak checks on frontend/backend/script changes.
- `.github/workflows/backend-ci.yml` also includes a `public-env-safety` job for backend CI runs.
- `.github/workflows/frontend-ci.yml` enforces frontend lint + build on frontend changes.
- `.github/workflows/fullstack-smoke.yml` boots backend + frontend and runs `scripts/fullstack-smoke.sh` to fail CI on broken critical user paths.
  - `scripts/fullstack-smoke.sh` now includes `scripts/route-contract-checks.sh` for Next `/api/admin/*` and `/api/account/*` contract assertions.

## Usage

```bash
./scripts/run-release-gates.sh
```

```powershell
.\scripts\run-release-gates.ps1
```

## Notes

- These scripts do not perform schema mutation or rollback.
- Migration safety/rollback rehearsal is handled separately in:
  - `backend/scripts/rehearse-migration-rollback.ps1`
  - `backend/scripts/rehearse-migration-rollback.sh`
  - `backend/docs/MIGRATION_ROLLBACK_REHEARSAL.md`
