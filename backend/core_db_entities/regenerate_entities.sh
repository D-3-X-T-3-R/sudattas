#!/usr/bin/env bash
set -euo pipefail

echo "=================================================="
echo "SeaORM Entity Regeneration Script"
echo "=================================================="
echo

echo "[1/6] Checking Docker..."
if ! docker version >/dev/null 2>&1; then
  echo "[FAIL] Docker not found. Please start Docker." >&2
  exit 1
fi
echo "[OK] Docker is available"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
backend_root="$(cd "${script_dir}/.." && pwd)"

echo
echo "[2/6] Starting Docker services..."
(
  cd "${backend_root}"
  docker compose up -d
)
echo "[OK] Docker services started"

echo
echo "[3/6] Waiting for MySQL to be ready (30 seconds)..."
sleep 30

connected=0
for retry in 1 2 3 4 5; do
  if docker exec sudattas-mysql mysql -u root -p12345678 SUDATTAS -e "SELECT 1;" >/dev/null 2>&1; then
    connected=1
    echo "[OK] MySQL is ready"
    break
  fi
  echo "  Retry ${retry}/5..."
  sleep 5
done

if [[ "${connected}" -ne 1 ]]; then
  echo "[FAIL] Could not connect to MySQL" >&2
  echo "  Try: docker logs sudattas-mysql" >&2
  exit 1
fi

echo
echo "[4/6] Checking database tables..."
table_output="$(docker exec sudattas-mysql mysql -u root -p12345678 SUDATTAS -e "SHOW TABLES;" 2>/dev/null || true)"
table_count="$(printf "%s\n" "${table_output}" | wc -l | awk '{print $1-1}')"
echo "[OK] Found ${table_count} tables"

phase1_tables=(sessions payment_intents shipments coupons order_events webhook_events)
for table in "${phase1_tables[@]}"; do
  if docker exec sudattas-mysql mysql -u root -p12345678 SUDATTAS -e "SHOW TABLES LIKE '${table}';" 2>/dev/null | grep -q "${table}"; then
    echo "  [OK] ${table} exists"
  else
    echo "  [WARN] ${table} NOT found"
  fi
done

echo
echo "[5/6] Removing old entity files..."
entity_dir="${script_dir}/src/entity"
(
  cd "${entity_dir}"
  rm -f ./*.rs
)
echo "[OK] Old entity files removed"

echo
echo "[6/6] Generating new entity files..."
if ! command -v sea-orm-cli >/dev/null 2>&1; then
  echo "[FAIL] sea-orm-cli not found. Install with: cargo install sea-orm-cli" >&2
  exit 1
fi

(
  cd "${script_dir}"
  sea-orm-cli generate entity \
    -u "mysql://root:12345678@localhost:3306/SUDATTAS" \
    -o src/entity \
    --with-serde both \
    --date-time-crate chrono \
    --max-connections 1
)
echo "[OK] Entity generation complete"

echo
echo "=================================================="
echo "Summary"
echo "=================================================="

new_files_count="$(find "${script_dir}/src/entity" -maxdepth 1 -type f -name "*.rs" | wc -l | tr -d ' ')"
echo "Total entity files: ${new_files_count}"

echo
echo "Phase 1 entity files:"
find "${script_dir}/src/entity" -maxdepth 1 -type f -name "*.rs" \
  | grep -E "session|payment|shipment|coupon|order_event|webhook" \
  | xargs -r -n1 basename \
  | sed 's/^/  [OK] /'

echo
echo "=================================================="
echo "[SUCCESS] Entity regeneration complete!"
echo "=================================================="

