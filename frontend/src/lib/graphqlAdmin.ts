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
      "searchShipment",
      "createShipment",
      "updateShipment",
      "markOrderShipped",
      "markOrderDelivered",
      "updateOrderStatus",
    ])
  ) {
    return "/api/admin/shipments";
  }

  if (
    containsAny(query, [
      "searchReview",
      "searchProductReview",
      "updateReview",
      "deleteReview",
      "approveReview",
      "rejectReview",
    ])
  ) {
    return "/api/admin/reviews";
  }

  if (containsAny(query, ["searchUser", "updateUser", "createUser", "deleteUser"])) {
    return "/api/admin/customers";
  }

  if (containsAny(query, ["searchOrder", "searchOrderStatus", "updateOrder", "createOrder"])) {
    return "/api/admin/orders";
  }

  if (
    containsAny(query, [
      "searchCategory",
      "createCategory",
      "searchProduct",
      "createProduct",
      "updateProduct",
      "deleteProduct",
      "searchProductImage",
      "deleteProductImage",
      "searchSize",
      "searchColor",
      "searchFabric",
      "searchWeave",
      "searchOccasion",
      "searchProductMood",
      "createProductMood",
      "searchProductMoodMapping",
      "createProductMoodMapping",
      "deleteProductMoodMapping",
      "createProductVariant",
      "updateProductVariant",
      "deleteProductVariant",
      "createInventoryItem",
      "searchInventoryItem",
      "updateInventoryItem",
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
