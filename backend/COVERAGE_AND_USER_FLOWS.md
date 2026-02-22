# Test coverage and user interaction coverage

## How coverage is measured

- **Tool:** [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) (line coverage).
- **CI:** `.github/workflows/backend-ci.yml` runs `cargo tarpaulin --all-features --workspace --timeout 300 --out Xml` and uploads `cobertura.xml` to **Codecov**.
- **Target:** >80% line coverage on `core_operations` and `graphql` (see `TESTING.md`).
- **Local:** Install tarpaulin then run:
  ```bash
  cd backend
  cargo tarpaulin --all-features --workspace --out Xml
  ```
  Open or upload `cobertura.xml` (e.g. Codecov or a viewer) to see the actual percentage. The repo does not store a current number; it lives on Codecov after each CI run.

---

## Are we testing all user interactions?

**No.** Summary:

| Layer | User-facing surface | Tested? | Notes |
|-------|---------------------|--------|--------|
| **HTTP** | GET / (health), POST /v2 (GraphQL), POST /webhook/:provider, CORS | Partially | E2E: health, POST /v2 (apiVersion), 404, invalid JSON, syntax error, optional 401. No dedicated webhook E2E. |
| **GraphQL – Queries** | 20+ fields | **All hit by E2E** | **E2E (ignored):** `graphql/tests/e2e_all_graphql_operations.rs` has one test per query (apiVersion, authInfo, getCartItems, searchProduct, searchCategory, searchOrder, searchWishlistItem, searchCountry, searchState, getPaymentIntent, getShipment, validateCoupon, searchReview, searchInventoryItem, searchProductImage, getPresignedUploadUrl, getOrderEvents, searchDiscount, searchShippingMethod, searchShippingZone, getShippingAddresses). Run with `cargo test -p graphql --test e2e_all_graphql_operations -- --ignored` (server must be up). **Unit:** apiVersion, authInfo still covered in `e2e_tests.rs`. |
| **GraphQL – Mutations** | 40+ operations | **All hit by E2E** | **E2E (ignored):** Same file has one test per mutation (cart add/update/delete, create/update/delete product/category, placeOrder, createOrderDetails, updateOrderDetail, delete/update order, wishlist add/delete, country/state create/delete, createPaymentIntent, capturePayment, product image delete/update, create/update shipment, applyCoupon, create/update/delete review, inventory CRUD, discount CRUD, shipping method/zone/address CRUD, createOrderEvent, confirmImageUpload). Each sends minimal payload and asserts 200 + valid GraphQL response (data or errors). |
| **gRPC handlers** | Many RPCs | **Extended** | **Tested (unit, MockDB):** city (create, search, delete), cart (create, get, update, delete), users (create_user), products (search_product), **categories (create, search), country (create, search), state (create, search), wishlist (add, search)** — see `core_operations/tests/handler_user_flows.rs`. **Not yet tested (unit):** order, order_details, payment_intent, shipment, coupon, review, inventory, discount, shipping_*, product_images, order_events, webhooks. |
| **Integration (real DB)** | Critical paths | **4 flows** | create_user, search_product, cart_by_session, place_order. No integration tests for other domains (reviews, payments, shipments, addresses, etc.). |

So:

- **Coverage (line %):** Determined by tarpaulin; see Codecov (or run tarpaulin locally). Many GraphQL resolvers and gRPC handlers are never executed in tests, so line coverage for those areas is low.
- **User interactions:** We now test:
  - **GraphQL:** Every query and mutation is hit by E2E tests in `graphql/tests/e2e_all_graphql_operations.rs` (run with `--ignored` when server + gRPC are up). Each test POSTs a minimal request and asserts 200 and a valid response (data or errors).
  - gRPC handlers: city, cart, user create, product search; **plus categories (create, search), country (create, search), state (create, search), wishlist (add, search)** in `core_operations/tests/handler_user_flows.rs`.
  - A few integration flows (user, product search, cart, place_order).
  - E2E for health, apiVersion, authInfo, and error cases in `e2e_tests.rs`.

To expand further:

1. **gRPC:** Add handler tests (MockDB) for orders, order_details, payment_intent, shipment, coupon, review, inventory, discount, shipping_*, product_images, order_events.
2. **E2E:** Optional E2E for full flows (browse → cart → place order) and webhook endpoint.
3. **Integration:** Optional integration tests for more domains (reviews, payments, addresses).

Running `cargo tarpaulin` (or checking Codecov) gives the exact coverage number; this doc answers “what is tested” and “are we testing all user interaction” (we are not).
