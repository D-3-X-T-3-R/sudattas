#!/usr/bin/env bash
#
# Apply forward-only SQL migrations from database/migrations against SUDATTAS.
# Tracks applied files in schema_migrations(file_name, applied_at).
#
# Usage:
#   ./apply-db-migrations.sh

set -euo pipefail

BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$BACKEND_ROOT/.env"
MIGRATIONS_DIR="$BACKEND_ROOT/database/migrations"
DB_CONTAINER_NAME="sudattas-mysql"
MYSQL_HOST="127.0.0.1"

if [[ ! -d "$MIGRATIONS_DIR" ]]; then
  echo "Migrations directory not found: $MIGRATIONS_DIR" >&2
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "ERROR: .env not found. Cannot resolve DATABASE_URL." >&2
  exit 1
fi

DATABASE_LINE="$(grep -E '^[[:space:]]*DATABASE_URL=' "$ENV_FILE" | head -1)"
if [[ -z "$DATABASE_LINE" ]]; then
  echo "ERROR: DATABASE_URL not found in .env." >&2
  exit 1
fi

URL="${DATABASE_LINE#*=}"
URL="${URL//[[:space:]]/}"
if [[ ! "$URL" =~ ^mysql://([^:]+):([^@]+)@[^/]+/([^?]+) ]]; then
  echo "ERROR: DATABASE_URL format not recognized." >&2
  exit 1
fi

DB_USER="${BASH_REMATCH[1]}"
DB_PASSWORD="${BASH_REMATCH[2]}"
DB_NAME="${BASH_REMATCH[3]}"

if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER_NAME"; then
  echo "ERROR: DB container '$DB_CONTAINER_NAME' is not running." >&2
  exit 1
fi

MAX_ATTEMPTS=45
ATTEMPT=0
echo "Waiting for MySQL to accept connections..."
while [[ $ATTEMPT -lt $MAX_ATTEMPTS ]]; do
  ATTEMPT=$((ATTEMPT + 1))
  if docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
    sh -c "mysql -h$MYSQL_HOST -u$DB_USER -e 'select 1'" &>/dev/null; then
    break
  fi
  sleep 2
done

if ! docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
  sh -c "mysql -h$MYSQL_HOST -u$DB_USER -e 'select 1'" &>/dev/null; then
  echo "ERROR: MySQL did not become ready in time." >&2
  exit 1
fi

docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" sh -c \
  "mysql -h$MYSQL_HOST -u$DB_USER $DB_NAME -e \"CREATE TABLE IF NOT EXISTS schema_migrations (file_name VARCHAR(255) PRIMARY KEY, applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP);\"" \
  >/dev/null

shopt -s nullglob
migration_files=("$MIGRATIONS_DIR"/*.sql)
if [[ ${#migration_files[@]} -eq 0 ]]; then
  echo "No migration files found in $MIGRATIONS_DIR. Nothing to apply."
  exit 0
fi

applied_count=0
skipped_count=0
for migration in "${migration_files[@]}"; do
  file_name="$(basename "$migration")"
  exists="$(docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" sh -c \
    "mysql -N -s -h$MYSQL_HOST -u$DB_USER $DB_NAME -e \"SELECT 1 FROM schema_migrations WHERE file_name='${file_name}' LIMIT 1;\"")"
  if [[ "$exists" == "1" ]]; then
    echo "Skipping already applied migration: $file_name"
    skipped_count=$((skipped_count + 1))
    continue
  fi

  container_sql="/tmp/migration-${file_name}"
  docker cp "$migration" "${DB_CONTAINER_NAME}:${container_sql}"
  echo "Applying migration: $file_name"
  docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" sh -c \
    "mysql -h$MYSQL_HOST -u$DB_USER $DB_NAME < ${container_sql}"
  docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" sh -c \
    "mysql -h$MYSQL_HOST -u$DB_USER $DB_NAME -e \"INSERT INTO schema_migrations (file_name) VALUES ('${file_name}');\""
  applied_count=$((applied_count + 1))
done

echo "Migration apply complete. Applied: ${applied_count}, skipped: ${skipped_count}."
