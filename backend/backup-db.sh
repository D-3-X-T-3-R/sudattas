#!/usr/bin/env bash
#
# Backup the MySQL database running in the sudattas-mysql container.
#
# Usage (from backend/):
#   ./backup-db.sh
#
# Reads DATABASE_URL from .env. Writes timestamped .sql to database/db-backups/.

set -e
BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"
DB_CONTAINER_NAME="sudattas-mysql"
BACKUP_DIR="$BACKEND_ROOT/database/db-backups"
ENV_FILE="$BACKEND_ROOT/.env"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "ERROR: .env file not found at '$ENV_FILE'. Cannot resolve DATABASE_URL." >&2
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
  echo "ERROR: DATABASE_URL is not in expected mysql://user:pass@host:port/db format." >&2
  exit 1
fi

DB_USER="${BASH_REMATCH[1]}"
DB_PASSWORD="${BASH_REMATCH[2]}"
DB_NAME="${BASH_REMATCH[3]}"
MYSQL_HOST="127.0.0.1"

echo "Using DB credentials from DATABASE_URL: user='$DB_USER', db='$DB_NAME'."

mkdir -p "$BACKUP_DIR"

echo "Checking if DB container '$DB_CONTAINER_NAME' is running..."
if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER_NAME"; then
  echo "ERROR: DB container '$DB_CONTAINER_NAME' is not running. Start services first." >&2
  exit 1
fi

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP_FILE="$BACKUP_DIR/db-backup-$TIMESTAMP.sql"

echo "Creating backup from container '$DB_CONTAINER_NAME' into '$BACKUP_FILE'..."
if docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
  mysqldump --set-gtid-purged=OFF -h"$MYSQL_HOST" -u"$DB_USER" "$DB_NAME" > "$BACKUP_FILE" 2>&1; then
  if [[ -s "$BACKUP_FILE" ]]; then
    echo "Backup created successfully: '$BACKUP_FILE'."
    exit 0
  fi
fi

echo "ERROR: Failed to create DB backup." >&2
exit 1
