#!/usr/bin/env bash
set -euo pipefail

# Linux build equivalent for build.ps1
# Builds core_operations with the current toolchain/environment.

echo "Building Rust project (core_operations)..."
echo "----------------------------------------"

cargo build --package core_operations

echo "----------------------------------------"
echo "Build completed successfully."

