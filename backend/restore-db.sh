#!/usr/bin/env bash
#
# Restore the MySQL database from a backup file.
#
# Usage (from backend/):
#   ./restore-db.sh                    # restore from latest in database/db-backups/
#   ./restore-db.sh path/to/backup.sql # restore from specific file
#
# Reads DATABASE_URL from .env. Waits for MySQL to accept connections, then imports.

set -e
BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"
DB_CONTAINER_NAME="sudattas-mysql"
BACKUP_DIR="$BACKEND_ROOT/database/db-backups"
ENV_FILE="$BACKEND_ROOT/.env"
CONTAINER_PATH="/tmp/db-restore.sql"
MYSQL_HOST="127.0.0.1"

# Resolve backup file
if [[ -n "${1:-}" ]]; then
  if [[ ! -f "$1" ]]; then
    echo "ERROR: Backup file not found: '$1'." >&2
    exit 1
  fi
  BACKUP_PATH="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
else
  if [[ ! -d "$BACKUP_DIR" ]]; then
    echo "ERROR: Backup directory not found: '$BACKUP_DIR'. Run backup-db.sh first or pass a file." >&2
    exit 1
  fi
  LATEST="$(ls -t "$BACKUP_DIR"/db-backup-*.sql 2>/dev/null | head -1)"
  if [[ -z "$LATEST" || ! -f "$LATEST" ]]; then
    echo "ERROR: No db-backup-*.sql files in '$BACKUP_DIR'. Run backup-db.sh first or pass -BackupFile." >&2
    exit 1
  fi
  BACKUP_PATH="$LATEST"
fi

echo "Using backup file: $BACKUP_PATH"

# Credentials from .env
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

# Container must be running
if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER_NAME"; then
  echo "ERROR: DB container '$DB_CONTAINER_NAME' is not running. Run start-services.sh first." >&2
  exit 1
fi

# Wait for MySQL to accept TCP connections
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

echo "Copying backup into container..."
if ! docker cp "$BACKUP_PATH" "$DB_CONTAINER_NAME:$CONTAINER_PATH" 2>/dev/null; then
  echo "ERROR: Failed to copy backup into container." >&2
  exit 1
fi

echo "Restoring into database '$DB_NAME'..."
if docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
  sh -c "mysql -h$MYSQL_HOST -u$DB_USER $DB_NAME < $CONTAINER_PATH" 2>/dev/null; then
  echo "Restore completed successfully."
  exit 0
fi

echo "ERROR: Restore failed." >&2
exit 1
