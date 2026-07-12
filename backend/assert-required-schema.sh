#!/usr/bin/env bash
#
# Validate that migration-created critical tables are present in the active DB.
#
# Usage:
#   ./assert-required-schema.sh

set -euo pipefail

BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$BACKEND_ROOT/.env"
DB_CONTAINER_NAME="sudattas-mysql"
MYSQL_HOST="127.0.0.1"

REQUIRED_TABLES=(
  "RefundAttempts"
  "ReturnRequests"
  "ReturnRequestItems"
  "OrderInventoryRestores"
  "OrderInventoryRestoreItems"
  "Invoices"
  "SchemaMigrations"
)

normalize_table_name() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9'
}

if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER_NAME"; then
  echo "ERROR: DB container '$DB_CONTAINER_NAME' is not running." >&2
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

echo "Validating DB connectivity before schema assertion..."
MAX_ATTEMPTS=45
ATTEMPT=0
PROBE_OK=false
while [[ $ATTEMPT -lt $MAX_ATTEMPTS ]]; do
  ATTEMPT=$((ATTEMPT + 1))
  if docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
    sh -c "mysql -h$MYSQL_HOST -u$DB_USER -e 'select 1'" &>/dev/null; then
    PROBE_OK=true
    break
  fi
  sleep 2
done
if [[ "$PROBE_OK" != "true" ]]; then
  echo "ERROR: MySQL did not become ready in time." >&2
  exit 1
fi

FOUND_ROWS="$(docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" sh -c \
  "mysql -N -s -h$MYSQL_HOST -u$DB_USER -e \"SELECT table_name FROM information_schema.tables WHERE LOWER(table_schema) = LOWER('${DB_NAME}');\"")"

# Avoids bash 4+ associative arrays (stock macOS ships bash 3.2) by tracking
# normalized found-table names as lines in a temp file instead of a hash map.
NORMALIZED_FOUND_FILE="$(mktemp)"
trap 'rm -f "$NORMALIZED_FOUND_FILE"' EXIT

while IFS= read -r row; do
  [[ -z "$row" ]] && continue
  normalize_table_name "$row" >> "$NORMALIZED_FOUND_FILE"
  printf '\n' >> "$NORMALIZED_FOUND_FILE"
done <<< "$FOUND_ROWS"

MISSING=()
for table in "${REQUIRED_TABLES[@]}"; do
  normalized="$(normalize_table_name "$table")"
  if ! grep -Fxq "$normalized" "$NORMALIZED_FOUND_FILE"; then
    MISSING+=("$table")
  fi
done

if [[ ${#MISSING[@]} -gt 0 ]]; then
  IFS=', '
  echo "ERROR: Missing required migration table(s): ${MISSING[*]}. Database is NOT ready." >&2
  exit 1
fi

echo "Schema validation passed. Required migration tables are present."
