#!/usr/bin/env bash
#
# Start all backend services: MySQL, Redis, Core Operations (gRPC), GraphQL (Docker).
# 1) Backup current DB (if container is running)
# 2) Stop existing sudattas* containers, then start fresh via docker-compose
# 3) Restore DB from latest backup (if any)
#
# Usage (from backend/):
#   ./start-services.sh

set -e
BACKEND_ROOT="$(cd "$(dirname "$0")" && pwd)"

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

echo "Building and starting all services (MySQL, Redis, Core Operations, GraphQL)..."
COMPOSE_CMD="docker compose"
if ! docker compose version &>/dev/null; then
  COMPOSE_CMD="docker-compose"
fi
(cd "$BACKEND_ROOT" && $COMPOSE_CMD up -d --build) || { echo "Docker Compose failed." >&2; exit 1; }

# 3) Restore DB from latest backup (if any)
echo "Restoring DB from latest backup (if any)..."
sleep 3
if ! "$BACKEND_ROOT/restore-db.sh" 2>/dev/null; then
  echo "Restore skipped or failed (no backup file or DB not ready)."
fi

echo ""
echo "Done. All services running in Docker:"
echo "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)"
echo "  Stop with: docker compose down  (or docker-compose down)"
