#!/usr/bin/env bash
set -euo pipefail

# Run E2E tests locally. Requires:
# - MySQL with SUDATTAS schema (e.g. docker-compose up -d from backend/)
# - Redis (optional; set REDIS_URL or leave unset for session-disabled)
# Run from backend/: ./run_e2e_local.sh

backend="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${backend}"

export DATABASE_URL="${DATABASE_URL:-mysql://root:12345678@127.0.0.1:3306/SUDATTAS}"
export GRPC_SERVER="${GRPC_SERVER:-0.0.0.0:50051}"
export GRPC_URL="${GRPC_URL:-http://127.0.0.1:50051}"
export GRAPHQL_URL="${GRAPHQL_URL:-http://127.0.0.1:8080}"
export GRAPHQL_SESSION_ID="${GRAPHQL_SESSION_ID:-ci-e2e-session}"
export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"
export OAUTH_DOMAIN="${OAUTH_DOMAIN:-https://accounts.google.com/}"
export OAUTH_AUDIENCE="${OAUTH_AUDIENCE:-https://www.googleapis.com/oauth2/v3/tokeninfo}"
export RATE_LIMIT_PER_MINUTE="${RATE_LIMIT_PER_MINUTE:-1000}"
export INTERNAL_API_SECRET="${INTERNAL_API_SECRET:-e2e-internal-secret}"
export GRAPHQL_E2E_CUSTOMER_USER_ID="${GRAPHQL_E2E_CUSTOMER_USER_ID:-2}"
export GRAPHQL_E2E_ADMIN_USER_ID="${GRAPHQL_E2E_ADMIN_USER_ID:-1}"
export ADMIN_ALLOWED_USER_IDS="${ADMIN_ALLOWED_USER_IDS:-${GRAPHQL_E2E_ADMIN_USER_ID}}"

if command -v redis-cli >/dev/null 2>&1; then
  redis-cli -h 127.0.0.1 SET session:ci-e2e-session 1 >/dev/null 2>&1 || true
fi

echo "Building core_operations, graphql, and E2E test binaries..."
cargo build -p core_operations -p graphql --all-features --tests

echo "Starting core_operations (gRPC) on 50051..."
./target/debug/core_operations >/tmp/sudattas_core_operations.log 2>&1 &
grpc_pid=$!
sleep 5

echo "Starting graphql on 8080..."
./target/debug/graphql >/tmp/sudattas_graphql.log 2>&1 &
gql_pid=$!

cleanup() {
  kill "${grpc_pid}" "${gql_pid}" >/dev/null 2>&1 || true
  wait "${grpc_pid}" "${gql_pid}" >/dev/null 2>&1 || true
  echo "Stopped servers."
}
trap cleanup EXIT

max=30
for ((i=0; i<max; i++)); do
  if curl -fsS "http://127.0.0.1:8080/" >/dev/null 2>&1; then
    echo "GraphQL server ready."
    break
  fi
  if (( i == max - 1 )); then
    echo "GraphQL server did not become ready."
    exit 1
  fi
  sleep 2
done

echo "Running E2E tests (using pre-built test binaries)..."
e2e_tests_exe="$(find "${backend}/target/debug/deps" -maxdepth 1 -type f -name 'e2e_tests-*' ! -name '*.d' | head -n 1)"
e2e_all_exe="$(find "${backend}/target/debug/deps" -maxdepth 1 -type f -name 'e2e_all_graphql_operations-*' ! -name '*.d' | head -n 1)"
e2e_flows_exe="$(find "${backend}/target/debug/deps" -maxdepth 1 -type f -name 'e2e_business_flows-*' ! -name '*.d' | head -n 1)"

if [[ -z "${e2e_tests_exe}" || -z "${e2e_all_exe}" || -z "${e2e_flows_exe}" ]]; then
  echo "Test executables not found. Run: cargo build -p graphql --all-features --tests" >&2
  exit 1
fi

"${e2e_tests_exe}" --ignored
"${e2e_all_exe}" --ignored
"${e2e_flows_exe}" --ignored

echo "E2E tests passed."

