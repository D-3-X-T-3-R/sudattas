# Admin Flow Coverage Matrix

This document maps each current admin screen to the backend GraphQL operations it uses and calls out gaps.

## Coverage sources reviewed

1. Frontend admin routes under `frontend/src/app/api/admin/*`
2. Backend GraphQL query/mutation roots under `backend/graphql/src/query_handler/*`
3. Backend integration backlog file `backend/R2_Integration_Hardening.todo`

Note: `backend/graphql/TODO.md` is referenced in `TODO_CHECKLIST.md` but does not exist in this repository.

## Screen coverage

### `/imtheboss` (dashboard)

- List/read:
`searchOrder`, `searchOrderStatus`, `searchProduct`, `searchUser`
- Create/update/delete:
None from dashboard itself (read-only aggregates)
- Status change:
None from dashboard itself
- Error state:
Query failures surface through shared admin query error handling

### `/imtheboss/products`

- List/read:
`searchProduct`, `searchCategory`, `searchProductImage`, `searchSize`, `searchColor`, `searchFabric`, `searchWeave`, `searchOccasion`, `searchProductMood`, `searchProductMoodMapping`, `searchInventoryItem`
- Create:
`createProduct`, `createCategory`, `createProductVariant`, `createInventoryItem`, `createProductMood`, `createProductMoodMapping`, `getPresignedUploadUrl`, `confirmImageUpload`
- Update:
`updateProduct`, `updateProductVariant`, `updateInventoryItem`, `syncProductImages`
- Delete:
`deleteProduct`, `deleteProductVariant`, `deleteProductImage`, `deleteProductMoodMapping`
- Status change:
Product status is updated via `updateProduct(productStatusId)`
- Error state:
Handled in mutation/query error branches and toast UI

### `/imtheboss/orders`

- List/read:
`searchOrder`, `searchOrderStatus`
- Create/update/delete:
No direct create/update/delete actions exposed in this screen today
- Status change:
Not yet exposed in UI (backend supports admin shipment/status mutations via admin API routes)
- Error state:
Handled in query error branch with retry button

### `/imtheboss/customers`

- List/read:
`searchUser`, plus `searchOrder` for per-customer aggregates
- Create/update/delete:
No customer mutation actions exposed in this screen today
- Status change:
Not applicable
- Error state:
Handled in query error branch with retry

### `/imtheboss/settings`

- Current state:
Informational placeholder only; no backend write actions are exposed.
- UI safety:
No action buttons are presented, so no unsupported mutation path can be invoked.

## Buttons/actions without backend paths

No live admin action buttons were found that call missing backend operations.
Where backend support exists but the UI does not expose it yet (for example order shipment/status transitions), the action is absent rather than broken.
