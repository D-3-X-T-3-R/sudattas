#!/usr/bin/env sh


set -e
set -x

rm -f *.rs
# Always use latest SeaORM CLI before generation.
cargo install sea-orm-cli --locked --force

ENV_FILE="$(dirname "$0")/../../../.env"
DATABASE_URL="$(grep -E '^\s*DATABASE_URL=' "$ENV_FILE" | head -1 | cut -d= -f2-)"
if [ -z "$DATABASE_URL" ]; then
  echo "ERROR: DATABASE_URL not found in $ENV_FILE." >&2
  exit 1
fi

~/.cargo/bin/sea-orm-cli generate entity -u "$DATABASE_URL" --with-serde=both --date-time-crate chrono --max-connections=1000
