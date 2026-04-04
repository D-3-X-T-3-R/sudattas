#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

declare -a BLOCKED_PATTERNS=(
  "NEXT_PUBLIC_ADMIN_API_KEY"
  "NEXT_PUBLIC_GRPC_AUTH_TOKEN"
  "NEXT_PUBLIC_RAZORPAY_KEY_SECRET"
  "NEXT_PUBLIC_GOOGLE_CLIENT_SECRET"
  "NEXT_PUBLIC_AUTH_SECRET"
)

failures=0

search_hits() {
  local pattern="$1"
  local files
  files="$(git -C "$ROOT_DIR" ls-files frontend backend | grep -E '(\.ts|\.tsx|\.js|\.mjs|\.cjs|/\.env|\.env\.)' || true)"
  [[ -n "$files" ]] || return 0
  if command -v rg >/dev/null 2>&1; then
    # shellcheck disable=SC2086
    rg -n "$pattern" $files || true
  else
    # shellcheck disable=SC2086
    grep -nE "$pattern" $files || true
  fi
}

for pattern in "${BLOCKED_PATTERNS[@]}"; do
  hits="$(search_hits "$pattern" || true)"
  if [[ -n "$hits" ]]; then
    echo "FAIL blocked public env pattern found: $pattern"
    echo "$hits"
    failures=$((failures + 1))
  else
    echo "OK   pattern not found: $pattern"
  fi
done

if [[ "$failures" -gt 0 ]]; then
  echo
  echo "Blocked NEXT_PUBLIC privileged env variables detected."
  exit 1
fi

echo
echo "No blocked public env patterns detected."
