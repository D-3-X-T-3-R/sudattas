#!/usr/bin/env bash
#
# Rehearse schema migration + rollback on local Docker MySQL.
#
# Usage (from backend/):
#   ./scripts/rehearse-migration-rollback.sh --apply-schema
#   ./scripts/rehearse-migration-rollback.sh --apply-schema --schema-file ./database/sql_dump/01_schema.sql
#
# What it does:
#   1) Creates a backup (backup-db.sh)
#   2) Captures pre-change table snapshot (name + row estimate)
#   3) Optionally applies schema SQL
#   4) Runs basic smoke checks
#   5) Restores backup (restore-db.sh)
#   6) Captures post-restore snapshot and compares with pre-change snapshot

set -euo pipefail

BACKEND_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DB_CONTAINER_NAME="sudattas-mysql"
DEFAULT_SCHEMA="$BACKEND_ROOT/database/sql_dump/01_schema.sql"
SCHEMA_FILE="$DEFAULT_SCHEMA"
APPLY_SCHEMA=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply-schema)
      APPLY_SCHEMA=1
      shift
      ;;
    --schema-file)
      SCHEMA_FILE="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

parse_database_url() {
  local env_file="$1"
  local line url
  [[ -f "$env_file" ]] || { echo "missing .env at $env_file" >&2; return 1; }
  line="$(grep -E '^[[:space:]]*DATABASE_URL=' "$env_file" | head -1 || true)"
  [[ -n "$line" ]] || { echo "DATABASE_URL missing in $env_file" >&2; return 1; }
  url="${line#*=}"
  url="${url//[[:space:]]/}"
  if [[ ! "$url" =~ ^mysql://([^:]+):([^@]+)@[^/]+/([^?]+) ]]; then
    echo "invalid DATABASE_URL format" >&2
    return 1
  fi
  DB_USER="${BASH_REMATCH[1]}"
  DB_PASSWORD="${BASH_REMATCH[2]}"
  DB_NAME="${BASH_REMATCH[3]}"
}

ensure_db_container_running() {
  if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER_NAME"; then
    echo "DB container '$DB_CONTAINER_NAME' is not running." >&2
    exit 1
  fi
}

table_snapshot() {
  docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
    mysql -h127.0.0.1 -u"$DB_USER" -N -e \
    "SELECT TABLE_NAME, TABLE_ROWS FROM information_schema.tables WHERE table_schema='$DB_NAME' ORDER BY TABLE_NAME;"
}

run_smoke_checks() {
  local required=(users products orders order_details payment_intents)
  local t count
  for t in "${required[@]}"; do
    count="$(docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
      mysql -h127.0.0.1 -u"$DB_USER" -N -e \
      "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$DB_NAME' AND table_name='$t';")"
    if [[ "${count:-0}" -lt 1 ]]; then
      echo "Smoke check failed: missing required table '$t'" >&2
      exit 1
    fi
  done
}

echo "== Migration + Rollback Rehearsal =="
ensure_db_container_running
parse_database_url "$BACKEND_ROOT/.env"

echo "Step 1/6: creating backup..."
"$BACKEND_ROOT/backup-db.sh"

BACKUP_DIR="$BACKEND_ROOT/database/db-backups"
BACKUP_PATH="$(ls -t "$BACKUP_DIR"/db-backup-*.sql | head -1)"
[[ -n "${BACKUP_PATH:-}" ]] || { echo "Could not locate backup file."; exit 1; }
echo "Backup: $BACKUP_PATH"

TS="$(date +%Y%m%d-%H%M%S)"
REHEARSAL_DIR="$BACKUP_DIR/rehearsal"
mkdir -p "$REHEARSAL_DIR"
PRE_SNAPSHOT="$REHEARSAL_DIR/snapshot-pre-$TS.txt"
POST_SNAPSHOT="$REHEARSAL_DIR/snapshot-post-$TS.txt"

echo "Step 2/6: capturing pre-change snapshot..."
table_snapshot > "$PRE_SNAPSHOT"

if [[ "$APPLY_SCHEMA" -eq 1 ]]; then
  [[ -f "$SCHEMA_FILE" ]] || { echo "Schema file not found: $SCHEMA_FILE" >&2; exit 1; }
  echo "Step 3/6: applying schema from '$SCHEMA_FILE'..."
  docker cp "$SCHEMA_FILE" "$DB_CONTAINER_NAME:/tmp/rehearsal-schema.sql"
  docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
    sh -c "mysql -h127.0.0.1 -u$DB_USER $DB_NAME < /tmp/rehearsal-schema.sql"
else
  echo "Step 3/6: schema apply skipped (pass --apply-schema to run full rehearsal)."
fi

echo "Step 4/6: running smoke checks..."
run_smoke_checks

echo "Step 5/6: restoring backup..."
"$BACKEND_ROOT/restore-db.sh" "$BACKUP_PATH"

echo "Step 6/6: capturing post-restore snapshot..."
table_snapshot > "$POST_SNAPSHOT"

if ! diff -u "$PRE_SNAPSHOT" "$POST_SNAPSHOT" >/dev/null; then
  echo "Rollback rehearsal completed with snapshot differences."
  echo "Review:"
  echo "  $PRE_SNAPSHOT"
  echo "  $POST_SNAPSHOT"
  exit 2
fi

echo "Rollback rehearsal completed successfully (pre/post snapshots match)."
echo "Artifacts:"
echo "  $BACKUP_PATH"
echo "  $PRE_SNAPSHOT"
echo "  $POST_SNAPSHOT"

