#!/usr/bin/env bash
set -euo pipefail

# Start database and regenerate entities.
# 1. Stop any running container using an image whose name starts with sudattas_
# 2. Run database/run_database.sh (build and start container)
# 3. Run core_db_entities/src/entity/generate.sh to regenerate SeaORM entities

BACKEND_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATABASE_DIR="$BACKEND_ROOT/database"
ENTITY_DIR="$BACKEND_ROOT/core_db_entities/src/entity"

running="$(docker ps --format '{{.ID}} {{.Image}}' 2>/dev/null || true)"
if [[ -n "${running}" ]]; then
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    id="${line%% *}"
    image="${line#* }"
    if [[ "${image}" == sudattas_* ]]; then
      echo "Stopping container ${id} (image: ${image})..."
      docker stop "${id}" >/dev/null 2>&1 || true
    fi
  done <<< "${running}"
fi

run_db_script="$DATABASE_DIR/run_database.sh"
if [[ ! -f "${run_db_script}" ]]; then
  echo "Database start script not found: ${run_db_script}" >&2
  exit 1
fi

echo "Starting database..."
(
  cd "${DATABASE_DIR}"
  bash "${run_db_script}"
)

echo "Waiting for MySQL to be ready..."
sleep 15

generate_script="$ENTITY_DIR/generate.sh"
if [[ ! -f "${generate_script}" ]]; then
  echo "Generate script not found: ${generate_script}" >&2
  exit 1
fi

echo "Regenerating entities..."
(
  cd "${ENTITY_DIR}"
  bash "${generate_script}"
)

echo
echo "Done. Database is running and entities were regenerated."

