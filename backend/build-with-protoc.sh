#!/usr/bin/env bash
set -euo pipefail

# Build script with protoc environment setup (Linux equivalent).

echo "Setting up build environment..."

if command -v protoc >/dev/null 2>&1; then
  export PROTOC="${PROTOC:-$(command -v protoc)}"
  echo "Found protoc at: ${PROTOC}"
else
  echo "WARNING: protoc not found in PATH. Build may fail."
  echo "Trying to build anyway..."
fi

echo
echo "Building core_operations..."
echo "========================================"

cargo build --package core_operations

echo "========================================"
echo "[SUCCESS] Build completed successfully!"

