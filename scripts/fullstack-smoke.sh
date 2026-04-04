#!/usr/bin/env bash
set -euo pipefail

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:8080}"
FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:3000}"

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

check_url "$BACKEND_URL/ready" "backend-ready" "200"
check_url "$FRONTEND_URL/" "home-page" "200"
check_url "$FRONTEND_URL/imtheboss/login" "admin-login-page" "200"

FRONTEND_URL="$FRONTEND_URL" GRAPHQL_SESSION_ID="${GRAPHQL_SESSION_ID:-ci-smoke-session}" \
  "$(cd "$(dirname "$0")" && pwd)/route-contract-checks.sh"

echo "Full-stack smoke checks passed."
