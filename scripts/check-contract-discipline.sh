#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
NODE_BIN="$(command -v node || true)"
if [[ -z "$NODE_BIN" ]]; then
  NODE_BIN="$(command -v node.exe || true)"
fi
if [[ -z "$NODE_BIN" ]]; then
  echo "FAIL Node.js is required but was not found in PATH (node/node.exe)." >&2
  exit 1
fi

cd "$ROOT_DIR"
"$NODE_BIN" "./scripts/check-contract-discipline.mjs"
