#!/usr/bin/env bash
set -euo pipefail

URL="${1:-http://127.0.0.1:8080/ready}"
MAX_ATTEMPTS="${2:-60}"
SLEEP_SECONDS="${3:-2}"

echo "Waiting for readiness endpoint: ${URL}"
for i in $(seq 1 "${MAX_ATTEMPTS}"); do
  if curl -sf "${URL}" > /dev/null 2>&1; then
    echo "Ready: ${URL}"
    exit 0
  fi
  sleep "${SLEEP_SECONDS}"
done

echo "Readiness check failed after ${MAX_ATTEMPTS} attempts: ${URL}" >&2
exit 1
