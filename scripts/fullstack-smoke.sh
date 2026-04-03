#!/usr/bin/env bash
set -euo pipefail

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:8080}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:3000}"
SESSION_ID="${GRAPHQL_SESSION_ID:-ci-smoke-session}"

echo "== Full-stack smoke checks =="
echo "Backend:  $BACKEND_URL"
echo "Frontend: $FRONTEND_URL"

check_url() {
  local url="$1"
  local label="$2"
  local expected="${3:-200}"
  local code
  code="$(curl -sS -o /tmp/sudattas_smoke_body.txt -w "%{http_code}" "$url")"
  if [[ "$code" != "$expected" ]]; then
    echo "FAIL $label ($url) expected $expected got $code" >&2
    cat /tmp/sudattas_smoke_body.txt || true
    exit 1
  fi
  echo "OK   $label ($code)"
}

check_body_contains() {
  local file="$1"
  local expected="$2"
  local label="$3"
  if ! grep -q "$expected" "$file"; then
    echo "FAIL $label missing expected pattern: $expected" >&2
    cat "$file" || true
    exit 1
  fi
  echo "OK   $label"
}

check_url "$BACKEND_URL/ready" "backend-ready" "200"
check_url "$FRONTEND_URL/" "home-page" "200"
check_url "$FRONTEND_URL/imtheboss/login" "admin-login-page" "200"

code="$(curl -sS -o /tmp/sudattas_smoke_products.json -w "%{http_code}" \
  -H "x-session-id: $SESSION_ID" \
  "$FRONTEND_URL/api/products")"
if [[ "$code" != "200" ]]; then
  echo "FAIL storefront-products expected 200 got $code" >&2
  cat /tmp/sudattas_smoke_products.json || true
  exit 1
fi
echo "OK   storefront-products (200)"

code="$(curl -sS -o /tmp/sudattas_smoke_filters.json -w "%{http_code}" \
  -H "x-session-id: $SESSION_ID" \
  "$FRONTEND_URL/api/storefront-filters")"
if [[ "$code" != "200" ]]; then
  echo "FAIL storefront-filters expected 200 got $code" >&2
  cat /tmp/sudattas_smoke_filters.json || true
  exit 1
fi
echo "OK   storefront-filters (200)"

code="$(curl -sS -o /tmp/sudattas_smoke_account_orders.json -w "%{http_code}" \
  "$FRONTEND_URL/api/account/orders")"
if [[ "$code" != "401" ]]; then
  echo "FAIL account-orders-unauthorized expected 401 got $code" >&2
  cat /tmp/sudattas_smoke_account_orders.json || true
  exit 1
fi
echo "OK   account-orders-unauthorized (401)"
check_body_contains /tmp/sudattas_smoke_account_orders.json '"ok":false' "account-orders-envelope"
check_body_contains /tmp/sudattas_smoke_account_orders.json '"errorCode":"UNAUTHORIZED"' "account-orders-error-code"

code="$(curl -sS -o /tmp/sudattas_smoke_admin_products.json -w "%{http_code}" \
  -H "content-type: application/json" \
  --data '{"query":"query SmokeAdmin { searchProduct(search: { limit: \"1\" }) { productId } }"}' \
  "$FRONTEND_URL/api/admin/products")"
if [[ "$code" != "401" ]]; then
  echo "FAIL admin-products-unauthorized expected 401 got $code" >&2
  cat /tmp/sudattas_smoke_admin_products.json || true
  exit 1
fi
echo "OK   admin-products-unauthorized (401)"
check_body_contains /tmp/sudattas_smoke_admin_products.json '"ok":false' "admin-products-envelope"
check_body_contains /tmp/sudattas_smoke_admin_products.json '"errorCode":"UNAUTHORIZED"' "admin-products-error-code"

echo "Full-stack smoke checks passed."
