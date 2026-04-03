#!/usr/bin/env bash
set -euo pipefail

# Apply forward SQL migrations in lexical order and track them in schema_migrations.
#
# Usage:
#   ./backend/scripts/apply_sql_migrations.sh HOST PORT USER PASSWORD DB_NAME MIGRATIONS_DIR [--verify-idempotent]

if [[ $# -lt 6 ]]; then
  echo "Usage: $0 HOST PORT USER PASSWORD DB_NAME MIGRATIONS_DIR [--verify-idempotent]" >&2
  exit 1
fi

HOST="$1"
PORT="$2"
USER="$3"
PASSWORD="$4"
DB_NAME="$5"
MIGRATIONS_DIR="$6"
VERIFY_IDEMPOTENT="${7:-}"

if [[ ! -d "$MIGRATIONS_DIR" ]]; then
  echo "Migrations directory not found: $MIGRATIONS_DIR" >&2
  exit 1
fi

MYSQL_BASE=(mysql -h "$HOST" -P "$PORT" -u "$USER" "-p$PASSWORD")

apply_once() {
  "${MYSQL_BASE[@]}" "$DB_NAME" -e \
    "CREATE TABLE IF NOT EXISTS schema_migrations (file_name VARCHAR(255) PRIMARY KEY, applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP);"

  shopt -s nullglob
  local files=("$MIGRATIONS_DIR"/*.sql)
  local applied=0
  local skipped=0
  for migration in "${files[@]}"; do
    local file_name
    file_name="$(basename "$migration")"
    local safe_file_name="${file_name//\'/\'\'}"
    local already
    already="$("${MYSQL_BASE[@]}" -N -s "$DB_NAME" -e "SELECT 1 FROM schema_migrations WHERE file_name='${safe_file_name}' LIMIT 1;")"
    if [[ "$already" == "1" ]]; then
      echo "Skipping already applied migration: $file_name"
      skipped=$((skipped + 1))
      continue
    fi

    echo "Applying migration: $file_name"
    "${MYSQL_BASE[@]}" "$DB_NAME" < "$migration"
    "${MYSQL_BASE[@]}" "$DB_NAME" -e "INSERT INTO schema_migrations (file_name) VALUES ('${safe_file_name}');"
    applied=$((applied + 1))
  done

  echo "Migration apply pass complete. Applied: $applied, skipped: $skipped."
}

echo "Applying forward migrations from: $MIGRATIONS_DIR"
apply_once

if [[ "$VERIFY_IDEMPOTENT" == "--verify-idempotent" ]]; then
  echo "Re-running migration apply pass for idempotency verification..."
  apply_once
fi
