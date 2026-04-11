#!/usr/bin/env bash
set -euo pipefail

# Run all backend tests:
# 1) non-ignored workspace tests + doc tests
# 2) ignored core_operations DB/integration tests
# 3) ignored graphql e2e tests
#
# Usage:
#   ./backend/scripts/run-all-tests.sh

BACKEND_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$BACKEND_ROOT"

export TEST_DATABASE_URL="${TEST_DATABASE_URL:-mysql://root:12345678@127.0.0.1:3306/SUDATTAS}"
export GRAPHQL_URL="${GRAPHQL_URL:-http://127.0.0.1:8080}"
export GRAPHQL_SESSION_ID="${GRAPHQL_SESSION_ID:-ci-e2e-session}"
export ALLOWED_ORIGINS="${ALLOWED_ORIGINS:-http://127.0.0.1:3000}"
export INTERNAL_API_SECRET="${INTERNAL_API_SECRET:-ci-e2e-internal-secret}"
export GRAPHQL_E2E_CUSTOMER_USER_ID="${GRAPHQL_E2E_CUSTOMER_USER_ID:-2}"
export GRAPHQL_E2E_ADMIN_USER_ID="${GRAPHQL_E2E_ADMIN_USER_ID:-1}"
export ADMIN_ALLOWED_USER_IDS="${ADMIN_ALLOWED_USER_IDS:-1}"

echo "== Environment =="
echo "TEST_DATABASE_URL=$TEST_DATABASE_URL"
echo "GRAPHQL_URL=$GRAPHQL_URL"
echo "ALLOWED_ORIGINS=$ALLOWED_ORIGINS"

echo "== Readiness checks =="
"$BACKEND_ROOT/scripts/wait_for_http_ready.sh" "$GRAPHQL_URL/ready" 90 2

echo "== 1/3 Non-ignored workspace tests =="
cargo test --all-features --workspace --no-fail-fast -- --skip ignored
cargo test --doc --all-features --workspace

echo "== 2/3 Ignored core_operations DB/integration tests =="
for test_binary in \
  integration_abandoned_cart_outbox \
  integration_cart \
  integration_coupons \
  integration_order_state \
  integration_payments \
  integration_products \
  integration_refunds \
  integration_reviews \
  integration_shipping \
  integration_users \
  integration_users_carts_orders_products \
  integration_webhooks \
  integration_wishlist
do
  cargo test -p core_operations --test "$test_binary" --all-features --no-fail-fast -- --ignored --test-threads=1
done

for test_binary in handler_outbox handler_p2 handler_refunds_resolve handler_security
do
  cargo test -p core_operations --test "$test_binary" --all-features --no-fail-fast -- --ignored --test-threads=1
done

# Redis-backed ignored lib test (session lifecycle)
cargo test -p core_operations --lib --all-features -- --ignored --test-threads=1 test_session_lifecycle

echo "== 3/3 Ignored graphql e2e tests =="
cargo test -p graphql --test e2e_tests --all-features -- --ignored
cargo test -p graphql --test e2e_all_graphql_operations --all-features -- --ignored
cargo test -p graphql --test e2e_business_flows --all-features -- --ignored

echo "All backend tests completed."
