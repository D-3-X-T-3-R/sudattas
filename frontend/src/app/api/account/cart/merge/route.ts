import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CartItem = {
  cartId: string;
  userId: string;
  variantId: string;
  quantity: string;
};

const MERGE_CART = `mutation MergeCart($sessionId: String!) {
  mergeCart(sessionId: $sessionId) {
    cartId
    userId
    variantId
    quantity
  }
}`;

export async function POST(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    guestSessionId?: unknown;
    idempotencyKey?: unknown;
  };
  const guestSessionIdFromBody = String(body.guestSessionId ?? "").trim();
  const guestSessionIdFromHeader = request.headers.get("x-guest-session-id")?.trim() ?? "";
  if (!guestSessionIdFromHeader || !guestSessionIdFromBody) {
    return apiError("guestSessionId is required", 400, "VALIDATION_ERROR");
  }
  if (guestSessionIdFromHeader !== guestSessionIdFromBody) {
    return apiError("guestSessionId does not match the active guest session", 403, "FORBIDDEN");
  }
  const guestSessionId = guestSessionIdFromHeader;

  const idempotencyKey =
    String(body.idempotencyKey ?? "").trim() ||
    request.headers.get("idempotency-key")?.trim() ||
    `cart-merge-${customerUserId}-${guestSessionId}`;

  // The backend mergeCart mutation does the whole guest-cart-into-user-cart
  // merge atomically in one DB transaction (sum quantities on overlapping
  // variants, reassign new ones) — this replaces the previous hand-rolled
  // per-item delete/update/add sequence, which made multiple separate GraphQL
  // calls with no shared transaction and could leave a partial merge on failure.
  const res = await callGraphqlAsCustomer<{ mergeCart?: CartItem[] }>(
    customerUserId,
    MERGE_CART,
    { sessionId: guestSessionId },
    { "Idempotency-Key": idempotencyKey }
  );

  if (res.errors?.length) {
    return apiError(res.errors[0]?.message ?? "Failed to merge cart", 400, "GRAPHQL_ERROR");
  }

  const items = res.data?.mergeCart ?? [];

  return Response.json({
    ok: true,
    data: { items },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
