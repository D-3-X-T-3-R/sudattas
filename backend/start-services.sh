#!/usr/bin/env bash
#
# Start backend services with DB-first bootstrap and explicit DB mode.
# 1) Backup current DB (if container is running)
# 2) Stop existing sudattas* containers
# 3) Start MySQL + Redis only
# 4) DB mode:
#    - --preserve-data (default): restore latest backup, then apply forward migrations
#    - --fresh: load schema from database/sql_dump/01_schema.sql, then apply forward migrations
# 5) Regenerate SeaORM entities
# 6) Start Core Operations + GraphQL
#
# Usage (from backend/):
#   ./start-services.sh
#   ./start-services.sh --preserve-data
#   ./start-services.sh --fresh

set -e
BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$BACKEND_ROOT/.env"
SCHEMA_FILE="$BACKEND_ROOT/database/sql_dump/01_schema.sql"
DB_CONTAINER_NAME="sudattas-mysql"
DB_MODE="preserve-data"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fresh)
      DB_MODE="fresh"
      shift
      ;;
    --preserve-data)
      DB_MODE="preserve-data"
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Usage: ./start-services.sh [--fresh|--preserve-data]" >&2
      exit 1
      ;;
  esac
done

read_db_credentials() {
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "ERROR: .env not found. Cannot resolve DATABASE_URL." >&2
    exit 1
  fi
  local database_line url
  database_line="$(grep -E '^[[:space:]]*DATABASE_URL=' "$ENV_FILE" | head -1)"
  if [[ -z "$database_line" ]]; then
    echo "ERROR: DATABASE_URL not found in .env." >&2
    exit 1
  fi
  url="${database_line#*=}"
  url="${url//[[:space:]]/}"
  if [[ ! "$url" =~ ^mysql://([^:]+):([^@]+)@[^/]+/([^?]+) ]]; then
    echo "ERROR: DATABASE_URL format not recognized." >&2
    exit 1
  fi
  DB_USER="${BASH_REMATCH[1]}"
  DB_PASSWORD="${BASH_REMATCH[2]}"
}

wait_for_mysql() {
  local max_attempts=45 attempt=0
  echo "Waiting for MySQL to accept connections..."
  while [[ $attempt -lt $max_attempts ]]; do
    attempt=$((attempt + 1))
    if docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
      sh -c "mysql -h127.0.0.1 -u$DB_USER -e 'select 1'" &>/dev/null; then
      return 0
    fi
    sleep 2
  done
  echo "ERROR: MySQL did not become ready in time." >&2
  return 1
}

load_fresh_schema() {
  if [[ ! -f "$SCHEMA_FILE" ]]; then
    echo "ERROR: schema file not found: $SCHEMA_FILE" >&2
    exit 1
  fi
  read_db_credentials
  wait_for_mysql
  echo "Loading fresh schema from 01_schema.sql..."
  docker cp "$SCHEMA_FILE" "${DB_CONTAINER_NAME}:/tmp/01_schema.sql"
  docker exec -e MYSQL_PWD="$DB_PASSWORD" "$DB_CONTAINER_NAME" \
    sh -c "mysql -h127.0.0.1 -u$DB_USER < /tmp/01_schema.sql"
  echo "Fresh schema loaded."
}

# 1) Backup DB before tearing down containers
echo "Backing up DB (if running)..."
if ! "$BACKEND_ROOT/backup-db.sh" 2>/dev/null; then
  echo "Backup skipped or failed (DB container may not be running)."
fi

echo "Stopping existing sudattas containers..."
CONTAINERS="$(docker ps -a --format '{{.Names}}' 2>/dev/null | grep -E '^sudattas' || true)"
if [[ -n "$CONTAINERS" ]]; then
  for c in $CONTAINERS; do docker rm -f "$c" 2>/dev/null || true; done
  echo "Removed: $CONTAINERS"
else
  echo "No existing sudattas containers found."
fi

COMPOSE_CMD="docker compose"
if ! docker compose version &>/dev/null; then
  COMPOSE_CMD="docker-compose"
fi

echo "Starting database dependencies (MySQL, Redis)..."
(cd "$BACKEND_ROOT" && $COMPOSE_CMD up -d mysql redis) || { echo "Docker Compose (mysql/redis) failed." >&2; exit 1; }

if [[ "$DB_MODE" == "fresh" ]]; then
  load_fresh_schema
else
  echo "Restoring DB from latest backup (if any)..."
  sleep 3
  if ! "$BACKEND_ROOT/restore-db.sh" 2>/dev/null; then
    echo "Restore skipped or failed (no backup file or DB not ready)."
  fi
fi

echo "Applying forward DB migrations..."
bash "$BACKEND_ROOT/apply-db-migrations.sh"

# 5) Regenerate SeaORM entities from current DB schema
echo "Regenerating SeaORM entities..."
ENTITY_GENERATE_SCRIPT="$BACKEND_ROOT/core_db_entities/src/entity/generate.sh"
if [[ ! -f "$ENTITY_GENERATE_SCRIPT" ]]; then
  echo "Entity generation script not found: $ENTITY_GENERATE_SCRIPT"
else
  (
    cd "$(dirname "$ENTITY_GENERATE_SCRIPT")"
    sh "$ENTITY_GENERATE_SCRIPT"
  ) || { echo "Entity regeneration failed." >&2; exit 1; }
fi

# 6) Build and start app services after DB + entities are ready
echo "Building and starting app services (Core Operations, GraphQL)..."
(cd "$BACKEND_ROOT" && $COMPOSE_CMD up -d --build core_operations graphql) || { echo "Docker Compose (core services) failed." >&2; exit 1; }

# Keep only the latest 5 DB dumps under database/db-backups
echo "Pruning old DB backups (keeping latest 5)..."
BACKUP_PRUNE_DIR="$BACKEND_ROOT/database/db-backups"
if [[ -d "$BACKUP_PRUNE_DIR" ]]; then
  _i=0
  while IFS= read -r f; do
    [[ -f "$f" ]] || continue
    _i=$((_i + 1))
    if ((_i > 5)); then
      rm -f "$f"
      echo "  Removed: $(basename "$f")"
    fi
  done < <(ls -t "$BACKUP_PRUNE_DIR"/db-backup-*.sql 2>/dev/null)
fi

echo ""
echo "Done. All services running in Docker:"
echo "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)"
echo "  DB mode: $DB_MODE"
echo "  SeaORM entities refreshed from current DB schema"
echo "  Stop with: docker compose down  (or docker-compose down)"
