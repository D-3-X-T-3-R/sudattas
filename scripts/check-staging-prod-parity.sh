#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

BACKEND_STAGING="${1:-$ROOT_DIR/backend/.env.staging}"
BACKEND_PROD="${2:-$ROOT_DIR/backend/.env.production}"
FRONTEND_STAGING="${3:-$ROOT_DIR/frontend/.env.staging}"
FRONTEND_PROD="${4:-$ROOT_DIR/frontend/.env.production}"

read_env_value() {
  local file="$1"
  local key="$2"
  [[ -f "$file" ]] || { echo "__MISSING_FILE__"; return 0; }
  local line
  line="$(grep -E "^[[:space:]]*(export[[:space:]]+)?${key}=" "$file" | tail -1 || true)"
  [[ -n "$line" ]] || { echo "__MISSING_KEY__"; return 0; }
  line="${line#export }"
  local value="${line#*=}"
  value="${value%$'\r'}"
  value="${value%\"}"
  value="${value#\"}"
  value="${value%\'}"
  value="${value#\'}"
  echo "$value"
}

check_equal_key() {
  local staging_file="$1"
  local prod_file="$2"
  local key="$3"
  local sv pv
  sv="$(read_env_value "$staging_file" "$key")"
  pv="$(read_env_value "$prod_file" "$key")"
  if [[ "$sv" == "__MISSING_FILE__" || "$pv" == "__MISSING_FILE__" ]]; then
    echo "FAIL $key: missing env file (staging='$staging_file' prod='$prod_file')"
    return 1
  fi
  if [[ "$sv" == "__MISSING_KEY__" || "$pv" == "__MISSING_KEY__" ]]; then
    echo "FAIL $key: key missing in one env file"
    return 1
  fi
  if [[ "$sv" != "$pv" ]]; then
    echo "FAIL $key: staging != production"
    return 1
  fi
  echo "OK   $key"
}

check_presence_key() {
  local staging_file="$1"
  local prod_file="$2"
  local key="$3"
  local sv pv
  sv="$(read_env_value "$staging_file" "$key")"
  pv="$(read_env_value "$prod_file" "$key")"
  if [[ "$sv" == "__MISSING_FILE__" || "$pv" == "__MISSING_FILE__" ]]; then
    echo "FAIL $key: missing env file (staging='$staging_file' prod='$prod_file')"
    return 1
  fi
  if [[ "$sv" == "__MISSING_KEY__" || "$pv" == "__MISSING_KEY__" ]]; then
    echo "FAIL $key: key missing in one env file"
    return 1
  fi
  if [[ -z "$sv" || -z "$pv" ]]; then
    echo "FAIL $key: key present but empty"
    return 1
  fi
  echo "OK   $key (present in both)"
}

failures=0

echo "Checking frontend parity..."
for key in GRAPHQL_URL STOREFRONT_ORIGIN ADMIN_ALLOWED_EMAILS; do
  if ! check_equal_key "$FRONTEND_STAGING" "$FRONTEND_PROD" "$key"; then
    failures=$((failures + 1))
  fi
done

echo "Checking backend GraphQL parity..."
for key in ALLOWED_ORIGINS RATE_LIMIT_TRUST_PROXY_HEADERS RATE_LIMIT_PER_MINUTE RATE_LIMIT_WEBHOOK_PER_MINUTE GRAPHQL_MAX_QUERY_DEPTH GRAPHQL_MAX_QUERY_COMPLEXITY; do
  if ! check_equal_key "$BACKEND_STAGING" "$BACKEND_PROD" "$key"; then
    failures=$((failures + 1))
  fi
done

echo "Checking backend core parity..."
for key in GRPC_AUTH_TOKEN; do
  if ! check_presence_key "$BACKEND_STAGING" "$BACKEND_PROD" "$key"; then
    failures=$((failures + 1))
  fi
done

echo "Checking required secret presence..."
for key in AUTH_SECRET GOOGLE_CLIENT_ID GOOGLE_CLIENT_SECRET RAZORPAY_KEY_ID RAZORPAY_KEY_SECRET RAZORPAY_WEBHOOK_SECRET; do
  if ! check_presence_key "$BACKEND_STAGING" "$BACKEND_PROD" "$key"; then
    failures=$((failures + 1))
  fi
done

if [[ "$failures" -gt 0 ]]; then
  echo
  echo "Parity check failed with $failures issue(s)."
  exit 1
fi

echo
echo "Parity check passed."

