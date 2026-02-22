# Backend testing

## Overview

- **Unit tests**: gRPC handlers with mocked DB; GraphQL schema execution with mock context. No real DB or network.
- **Integration tests**: Critical paths against a real MySQL database. Require `TEST_DATABASE_URL` and loaded schema.
- **E2E**: Full flow against running GraphQL + gRPC services (optional; for manual or CI with services).

---

## Running tests

### Unit tests (default)

```bash
cd backend
cargo test
```

Runs all unit tests in all crates (core_operations, graphql, etc.), including:

- `core_operations`: handler tests using SeaORM `MockDatabase` (city, cart, orders, users, products).
- `graphql`: GraphQL query execution tests (`api_version`, `auth_info`) with in-memory context.

Exclude integration and E2E tests (which are `#[ignore]`):

```bash
cargo test -- --skip ignored
```

### Integration tests (real database)

Requires MySQL with schema loaded (e.g. from `backend/database/sql_dump/01_schema.sql`).

1. Set the database URL:

   ```bash
   export TEST_DATABASE_URL="mysql://root:password@127.0.0.1:3306/sudattas_test"
   # or on Windows:
   set TEST_DATABASE_URL=mysql://root:password@127.0.0.1:3306/sudattas_test
   ```

2. Load schema (one-time):

   ```bash
   mysql -h 127.0.0.1 -u root -p sudattas_test < backend/database/sql_dump/01_schema.sql
   ```

3. Run only integration tests:

   ```bash
   cd backend
   cargo test --test integration_critical_paths -- --ignored
   ```

   Or run all tests including ignored:

   ```bash
   cargo test -- --include-ignored
   ```

Integration tests cover:

- `integration_create_user` — create user (Argon2id hash)
- `integration_search_product` — product search
- `integration_cart_by_session` — guest cart by `session_id`
- `integration_place_order` — place order (may skip if cart empty / stock missing)

### E2E test (running services)

Requires GraphQL and gRPC servers to be running, and `GRAPHQL_URL` set (e.g. `http://127.0.0.1:8080`).

```bash
export GRAPHQL_URL=http://127.0.0.1:8080
cargo test --test e2e_tests -- --ignored
```

See `tests/e2e_tests.rs` for the single E2E flow (e.g. health → apiVersion).

---

## Environment variables

| Variable             | Used by              | Description |
|----------------------|----------------------|-------------|
| `TEST_DATABASE_URL`  | Integration tests    | MySQL URL for test DB (fallback: `DATABASE_URL`) |
| `DATABASE_URL`       | Integration tests    | Fallback if `TEST_DATABASE_URL` is not set |
| `GRAPHQL_URL`        | E2E test             | Base URL of GraphQL server (e.g. `http://127.0.0.1:8080`) |

Unit tests do not require any env vars (mocks only).

---

## Coverage

Generate coverage with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin):

```bash
cargo install cargo-tarpaulin
cd backend
cargo tarpaulin --all-features --workspace --out Xml
```

Output: `cobertura.xml` in the current directory. Target: aim for >80% line coverage on `core_operations` and `graphql` (see CI).

---

## CI

GitHub Actions (`.github/workflows/backend-ci.yml`) runs:

1. **Lint**: `cargo fmt --check`
2. **Build + unit tests**: `cargo test --lib --bins --all-features` (with MySQL service for dependency)
3. **Integration tests**: `cargo test --test '*' -- --ignored` with `TEST_DATABASE_URL` set
4. **Coverage**: `cargo tarpaulin` → upload to Codecov
5. **Security**: `cargo audit`
6. **Build**: stable and nightly

Schema is loaded from `backend/database/sql_dump/01_schema.sql` before tests.
