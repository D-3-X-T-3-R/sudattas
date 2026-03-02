#!/bin/bash
# Start all backend services: MySQL, Redis, Core Operations (gRPC), GraphQL (all in Docker)
# Stops existing sudattas* containers, then starts fresh via docker-compose.
# Run from backend/: ./start-services.sh

set -e
BACKEND_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BACKEND_ROOT"

echo "Stopping existing sudattas containers..."
containers=$(docker ps -a --format '{{.Names}}' 2>/dev/null | grep '^sudattas' || true)
if [ -n "$containers" ]; then
  echo "$containers" | xargs docker rm -f 2>/dev/null || true
  echo "Removed: $containers"
else
  echo "No existing sudattas containers found."
fi

echo "Building and starting all services (MySQL, Redis, Core Operations, GraphQL)..."
docker-compose up -d --build

echo ""
echo "Done. All services running in Docker:"
echo "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)"
echo "  Stop with: docker-compose down"
