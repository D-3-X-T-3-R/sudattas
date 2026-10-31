#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_FILE="$ROOT_DIR/backend/graphql/src/validation.rs"
FRONTEND_FILE="$ROOT_DIR/frontend/src/lib/validation-schemas.ts"

extract_backend_const() {
  local name="$1"
  local value
  value="$(grep -E "pub const ${name}:" "$BACKEND_FILE" | sed -E 's/.*= ([0-9]+).*/\1/' | head -n1)"
  if [[ -z "${value:-}" ]]; then
    echo "Failed to parse backend constant: $name" >&2
    exit 1
  fi
  echo "$value"
}

extract_frontend_const() {
  local name="$1"
  local value
  value="$(grep -E "export const ${name} =" "$FRONTEND_FILE" | sed -E 's/.*= ([0-9]+).*/\1/' | head -n1)"
  if [[ -z "${value:-}" ]]; then
    echo "Failed to parse frontend constant: $name" >&2
    exit 1
  fi
  echo "$value"
}

backend_sku="$(extract_backend_const "MAX_SKU_SLUG_LEN")"
backend_qty="$(extract_backend_const "MAX_QUANTITY_PER_ITEM")"
backend_addr="$(extract_backend_const "MAX_ADDRESS_LINE_LEN")"
backend_meta_title="$(extract_backend_const "MAX_META_TITLE_LEN")"
backend_meta_desc="$(extract_backend_const "MAX_META_DESCRIPTION_LEN")"

frontend_sku="$(extract_frontend_const "BACKEND_MAX_SKU_SLUG_LEN")"
frontend_qty="$(extract_frontend_const "BACKEND_MAX_QUANTITY_PER_ITEM")"
frontend_addr="$(extract_frontend_const "BACKEND_MAX_ADDRESS_LINE_LEN")"
frontend_postal="$(extract_frontend_const "BACKEND_POSTAL_CODE_LEN")"
frontend_meta_title="$(extract_frontend_const "BACKEND_MAX_META_TITLE_LEN")"
frontend_meta_desc="$(extract_frontend_const "BACKEND_MAX_META_DESCRIPTION_LEN")"

status=0

if [[ "$backend_sku" != "$frontend_sku" ]]; then
  echo "Mismatch: SKU/slug length backend=$backend_sku frontend=$frontend_sku" >&2
  status=1
fi

if [[ "$backend_qty" != "$frontend_qty" ]]; then
  echo "Mismatch: quantity limit backend=$backend_qty frontend=$frontend_qty" >&2
  status=1
fi

if [[ "$backend_addr" != "$frontend_addr" ]]; then
  echo "Mismatch: address line length backend=$backend_addr frontend=$frontend_addr" >&2
  status=1
fi

if [[ "$frontend_postal" != "6" ]]; then
  echo "Mismatch: postal code length frontend=$frontend_postal expected=6" >&2
  status=1
fi

if [[ "$backend_meta_title" != "$frontend_meta_title" ]]; then
  echo "Mismatch: meta title length backend=$backend_meta_title frontend=$frontend_meta_title" >&2
  status=1
fi

if [[ "$backend_meta_desc" != "$frontend_meta_desc" ]]; then
  echo "Mismatch: meta description length backend=$backend_meta_desc frontend=$frontend_meta_desc" >&2
  status=1
fi

if [[ "$status" -ne 0 ]]; then
  echo "FAIL validation parity check" >&2
  exit "$status"
fi

echo "OK validation parity: frontend and backend limits are aligned."
