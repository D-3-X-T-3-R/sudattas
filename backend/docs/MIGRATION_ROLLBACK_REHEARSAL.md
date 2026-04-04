# Migration + Rollback Rehearsal

This project currently maintains schema via SQL (`database/sql_dump/01_schema.sql`), so release safety requires explicit backup/restore rehearsal before production changes.

## Purpose

- Enforce reversible migration discipline.
- Prove rollback path works before deployment.
- Produce artifacts (backup + snapshots) for release review.

## Scripts

- PowerShell: `backend/scripts/rehearse-migration-rollback.ps1`
- Bash: `backend/scripts/rehearse-migration-rollback.sh`

Both scripts:

1. Create DB backup (`backup-db.*`).
2. Capture pre-change table snapshot (`TABLE_NAME`, `TABLE_ROWS`).
3. Optionally apply schema SQL.
4. Run smoke checks for critical tables.
5. Restore backup (`restore-db.*`).
6. Capture post-restore snapshot and compare against pre-change.

## Usage

From `backend/`:

```powershell
.\scripts\rehearse-migration-rollback.ps1 -ApplySchema
```

```bash
./scripts/rehearse-migration-rollback.sh --apply-schema
```

Optional schema file override:

```powershell
.\scripts\rehearse-migration-rollback.ps1 -ApplySchema -SchemaFile .\database\sql_dump\01_schema.sql
```

```bash
./scripts/rehearse-migration-rollback.sh --apply-schema --schema-file ./database/sql_dump/01_schema.sql
```

## Preconditions

- MySQL Docker container `sudattas-mysql` is running.
- `backend/.env` contains valid `DATABASE_URL`.
- `backup-db.*` and `restore-db.*` scripts are working locally.

## Release Gate Expectation

For schema-impacting releases:

- Rehearsal must complete with no snapshot diff (`pre == post` after restore).
- Keep generated artifacts in `database/db-backups/rehearsal/` for audit.
- If rehearsal fails, block release until root cause is fixed and rehearsal passes.

