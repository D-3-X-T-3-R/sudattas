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

function resolveAdminRoute(query: string): string {
  if (
    containsAny(query, [
      "getShipment",
      "createShipment",
      "updateShipment",
      "adminMarkOrderShipped",
      "adminMarkOrderDelivered",
      "syncOrderShipmentsFromShiprocket",
    ])
  ) {
    return "/api/admin/shipments";
  }

  if (
    containsAny(query, [
      "searchReview",
      "updateReview",
      "deleteReview",
      "adminUpdateReviewStatus",
    ])
  ) {
    return "/api/admin/reviews";
  }

  if (containsAny(query, ["searchUser", "updateUser", "createUser", "deleteUser"])) {
    return "/api/admin/customers";
  }

  if (
    containsAny(query, [
      "searchCouponAdmin",
      "createCouponAdmin",
      "updateCouponAdmin",
      "deleteCouponAdmin",
    ])
  ) {
    return "/api/admin/coupons";
  }

  if (
    containsAny(query, [
      "createTransaction",
      "searchTransaction",
      "updateTransaction",
      "deleteTransaction",
    ])
  ) {
    return "/api/admin/transactions";
  }

  if (
    containsAny(query, [
      "createUserRole",
      "searchUserRole",
      "updateUserRole",
      "deleteUserRole",
    ])
  ) {
    return "/api/admin/roles";
  }

  if (
    containsAny(query, [
      "searchUserActivity",
      "searchEventLog",
      "createUserActivity",
      "createEventLog",
      "deleteUserActivity",
      "deleteEventLog",
    ])
  ) {
    return "/api/admin/activity-log";
  }

  if (
    containsAny(query, [
      "searchNewsletterSubscriber",
      "createNewsletterSubscriber",
      "updateNewsletterSubscriber",
      "deleteNewsletterSubscriber",
      "sendNewsletterCampaign",
      "searchNewsletterCampaign",
    ])
  ) {
    return "/api/admin/newsletter";
  }

  if (
    containsAny(query, [
      "searchShippingMethod",
      "createShippingMethod",
      "updateShippingMethod",
      "deleteShippingMethod",
    ])
  ) {
    return "/api/admin/shipping-methods";
  }

  if (
    containsAny(query, [
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
    ])
  ) {
    return "/api/admin/orders";
  }

  if (
    containsAny(query, [
      "searchCategory",
      "createCategory",
      "updateCategory",
      "deleteCategory",
      "searchProduct",
      "createProduct",
      "updateProduct",
      "deleteProduct",
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
    ])
  ) {
    return "/api/admin/products";
  }

  return "/api/admin/graphql";
}
