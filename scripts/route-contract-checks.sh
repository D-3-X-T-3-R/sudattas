#!/usr/bin/env bash
set -euo pipefail

FRONTEND_URL="${FRONTEND_URL:-http://127.0.0.1:3000}"
SESSION_ID="${GRAPHQL_SESSION_ID:-ci-contract-session}"

echo "== Next API route contract checks =="
echo "Frontend: $FRONTEND_URL"

request_json() {
  local method="$1"
  local path="$2"
  local body_file="$3"
  local status_var="$4"
  shift 4
  local code
  code="$(curl -sS -o "$body_file" -w "%{http_code}" -X "$method" "$@" "$FRONTEND_URL$path")"
  printf -v "$status_var" "%s" "$code"
}

assert_status() {
  local got="$1"
  local expected="$2"
  local label="$3"
  if [[ "$got" != "$expected" ]]; then
    echo "FAIL $label expected HTTP $expected got $got" >&2
    exit 1
  fi
  echo "OK   $label ($expected)"
}

assert_envelope_error() {
  local file="$1"
  local expected_code="$2"
  local label="$3"
  node - "$file" "$expected_code" "$label" <<'NODE'
const fs = require("fs");
const [file, expectedCode, label] = process.argv.slice(2);
const json = JSON.parse(fs.readFileSync(file, "utf8"));
if (json.ok !== false) throw new Error(`${label}: expected ok=false`);
if (json.errorCode !== expectedCode) {
  throw new Error(`${label}: expected errorCode=${expectedCode} got ${json.errorCode}`);
}
if (typeof json.message !== "string" || json.message.trim() === "") {
  throw new Error(`${label}: expected non-empty message`);
}
if (!Object.prototype.hasOwnProperty.call(json, "retryable")) {
  throw new Error(`${label}: missing retryable`);
}
NODE
  echo "OK   $label envelope"
}

assert_products_shape() {
  local file="$1"
  node - "$file" <<'NODE'
const fs = require("fs");
const json = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
if (!Array.isArray(json.products)) throw new Error("products must be an array");
if (json.error !== null) throw new Error("products.error must be null");
if (json.products.length > 0) {
  const p = json.products[0];
  if (typeof p.id !== "string") throw new Error("product.id must be string");
  if (typeof p.name !== "string") throw new Error("product.name must be string");
  if (typeof p.price !== "number") throw new Error("product.price must be number");
}
NODE
  echo "OK   /api/products response typing"
}

assert_filters_shape() {
  local file="$1"
  node - "$file" <<'NODE'
const fs = require("fs");
const json = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
if (!Array.isArray(json.categories)) throw new Error("categories must be an array");
if (!Array.isArray(json.occasions)) throw new Error("occasions must be an array");
if (!Array.isArray(json.moods)) throw new Error("moods must be an array");
if (json.error !== null) throw new Error("error must be null");
NODE
  echo "OK   /api/storefront-filters response typing"
}

status=""

# Auth expectations + normalized error envelope on account path.
request_json "GET" "/api/account/orders" "/tmp/sudattas_contract_account_orders.json" status
assert_status "$status" "401" "account-orders unauthorized"
assert_envelope_error "/tmp/sudattas_contract_account_orders.json" "UNAUTHORIZED" "account-orders unauthorized"

# Request-shape enforcement on admin path (missing query).
request_json "POST" "/api/admin/products" "/tmp/sudattas_contract_admin_missing_query.json" status \
  -H "content-type: application/json" \
  --data '{}'
assert_status "$status" "400" "admin-products missing query"
assert_envelope_error "/tmp/sudattas_contract_admin_missing_query.json" "BAD_REQUEST" "admin-products missing query"

# Request-shape enforcement on admin path (disallowed root).
request_json "POST" "/api/admin/products" "/tmp/sudattas_contract_admin_disallowed_root.json" status \
  -H "content-type: application/json" \
  --data '{"query":"query DisallowedRoot { searchOrder(search: { limit: \"1\" }) { orderId } }"}'
assert_status "$status" "400" "admin-products disallowed root"
assert_envelope_error "/tmp/sudattas_contract_admin_disallowed_root.json" "BAD_REQUEST" "admin-products disallowed root"

# Response typing checks for representative storefront APIs.
request_json "GET" "/api/products" "/tmp/sudattas_contract_products.json" status \
  -H "x-session-id: $SESSION_ID"
assert_status "$status" "200" "storefront products"
assert_products_shape "/tmp/sudattas_contract_products.json"

request_json "GET" "/api/storefront-filters" "/tmp/sudattas_contract_filters.json" status \
  -H "x-session-id: $SESSION_ID"
assert_status "$status" "200" "storefront filters"
assert_filters_shape "/tmp/sudattas_contract_filters.json"

echo "Next API route contract checks passed."
