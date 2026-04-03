#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
READY_URL="${READY_URL:-http://127.0.0.1:8080/ready}"

echo "== Release Gates =="

echo "Gate: no exposed privileged NEXT_PUBLIC env vars"
"$ROOT_DIR/scripts/check-no-privileged-public-env.sh"

echo "Gate: frontend/backend validation limits stay aligned"
"$ROOT_DIR/scripts/check-validation-parity.sh"

echo "Gate: frontend performance budgets stay within limits"
"$ROOT_DIR/scripts/check-frontend-performance-budgets.sh"

echo "Gate: contract/documentation discipline is enforced"
"$ROOT_DIR/scripts/check-contract-discipline.sh"

echo "Gate: backend health check green ($READY_URL)"
if ! curl -fsS "$READY_URL" >/dev/null; then
  echo "FAIL backend readiness endpoint not healthy: $READY_URL" >&2
  exit 1
fi
echo "OK   backend readiness healthy"

echo "Gate: admin authorization tests green"
(
  cd "$BACKEND_DIR"
  cargo test -p graphql --test graphql_tests test_admin_mutation_requires_admin_authorization -- --exact
  cargo test -p graphql --test graphql_tests test_search_user_requires_admin_authorization -- --exact
)

echo "Gate: payment negative-path tests green"
(
  cd "$BACKEND_DIR"
  cargo test -p core_operations --test handler_payment_intents verify_razorpay_payment_missing_fields_returns_invalid_argument -- --exact
  cargo test -p core_operations --test handler_payment_intents verify_razorpay_payment_not_configured_returns_failed_precondition -- --exact
)

echo "All configured release gates passed."
