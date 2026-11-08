/**
 * Browser admin client. All privileged operations go through Next.js server route:
 * POST /api/admin/graphql
 */
import { fetchApiEnvelope } from "@/lib/api-envelope";

export async function gqlAdmin<T = unknown>(
  query: string,
  variables: Record<string, unknown> = {}
): Promise<T> {
  return fetchApiEnvelope<T>(resolveAdminRoute(query), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query, variables }),
  });
}

function containsAny(query: string, roots: string[]): boolean {
  const normalized = query.replace(/\s+/g, " ");
  return roots.some((root) => new RegExp(`\\b${root}\\b`).test(normalized));
}

const ADMIN_ROUTES = [
  {
    route: "/api/admin/shipments",
    roots: [
      "getShipment",
      "createShipment",
      "updateShipment",
      "adminMarkOrderShipped",
      "adminMarkOrderDelivered",
      "syncOrderShipmentsFromShiprocket",
    ],
  },
  {
    route: "/api/admin/reviews",
    roots: [
      "searchReview",
      "updateReview",
      "deleteReview",
      "adminUpdateReviewStatus",
    ],
  },
  {
    route: "/api/admin/customers",
    roots: [
      "searchUser",
      "updateUser",
      "createUser",
      "deleteUser",
      "setUserStatus",
      "adminExportUserPii",
    ],
  },
  {
    route: "/api/admin/coupons",
    roots: [
      "searchCouponAdmin",
      "createCouponAdmin",
      "updateCouponAdmin",
      "deleteCouponAdmin",
    ],
  },
  {
    route: "/api/admin/transactions",
    roots: [
      "createTransaction",
      "searchTransaction",
      "updateTransaction",
      "deleteTransaction",
    ],
  },
  {
    route: "/api/admin/roles",
    roots: [
      "createUserRole",
      "searchUserRole",
      "updateUserRole",
      "deleteUserRole",
    ],
  },
  {
    route: "/api/admin/activity-log",
    roots: [
      "searchUserActivity",
      "searchEventLog",
      "createUserActivity",
      "createEventLog",
      "deleteUserActivity",
      "deleteEventLog",
    ],
  },
  {
    route: "/api/admin/newsletter",
    roots: [
      "searchNewsletterSubscriber",
      "createNewsletterSubscriber",
      "updateNewsletterSubscriber",
      "deleteNewsletterSubscriber",
      "sendNewsletterCampaign",
      "searchNewsletterCampaign",
    ],
  },
  {
    route: "/api/admin/shipping-methods",
    roots: [
      "searchShippingMethod",
      "createShippingMethod",
      "updateShippingMethod",
      "deleteShippingMethod",
    ],
  },
  {
    route: "/api/admin/app-settings",
    roots: [
      "abandonedCartSettings",
      "updateAbandonedCartSettings",
    ],
  },
  {
    route: "/api/admin/returns",
    roots: [
      "searchReturnRequests",
      "adminMarkReturnReceived",
      "adminUpdateReturnStatus",
    ],
  },
  {
    route: "/api/admin/exchanges",
    roots: [
      "searchExchangeRequests",
      "adminMarkExchangeReceived",
      "adminUpdateExchangeStatus",
      "scheduleExchangePickup",
      "syncExchangePickup",
    ],
  },
  {
    route: "/api/admin/refunds",
    roots: [
      "getRefunds",
      "createRefund",
      "searchRefundAttempts",
      "resolveRefundAttemptNeedsReview",
    ],
  },
  {
    route: "/api/admin/payment-intents",
    roots: [
      "searchPaymentIntent",
      "getPaymentIntent",
    ],
  },
  {
    route: "/api/admin/orders",
    roots: [
      "searchOrder",
      "searchOrderStatus",
      "updateOrder",
      "deleteOrder",
      "createOrder",
      "createOrderAdmin",
      "createOrderDetails",
      "placeOrderAdmin",
      "resolveNeedsReview",
      "orderStats",
      "updatePickupTarget",
      "getOrderEvents",
      "createOrderEvent",
      "getShippingAddresses",
      "createShippingAddress",
    ],
  },
  {
    route: "/api/admin/products",
    roots: [
      "searchCategory",
      "createCategory",
      "updateCategory",
      "deleteCategory",
      "searchProduct",
      "createProduct",
      "updateProduct",
      "deleteProduct",
      "permanentlyDeleteProduct",
      "searchProductImage",
      "updateProductImage",
      "deleteProductImage",
      "searchSize",
      "createSize",
      "updateSize",
      "deleteSize",
      "searchColor",
      "createColor",
      "updateColor",
      "deleteColor",
      "searchFabric",
      "createFabric",
      "updateFabric",
      "deleteFabric",
      "searchWeave",
      "createWeave",
      "updateWeave",
      "deleteWeave",
      "searchOccasion",
      "createOccasion",
      "updateOccasion",
      "deleteOccasion",
      "searchProductMood",
      "createProductMood",
      "updateProductMood",
      "deleteProductMood",
      "searchProductMoodMapping",
      "createProductMoodMapping",
      "deleteProductMoodMapping",
      "createProductVariant",
      "updateProductVariant",
      "deleteProductVariant",
      "createInventoryItem",
      "searchInventoryItem",
      "updateInventoryItem",
      "deleteInventoryItem",
      "createInventoryLog",
      "searchInventoryLog",
      "updateInventoryLog",
      "deleteInventoryLog",
      "getPresignedUploadUrl",
      "confirmImageUpload",
      "syncProductImages",
      "shopHighlightMoods",
    ],
  },
];

function resolveAdminRoute(query: string): string {
  return ADMIN_ROUTES.find(({ roots }) => containsAny(query, roots))?.route ?? "/api/admin/graphql";
}
