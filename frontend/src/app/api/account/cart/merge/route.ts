import {
  apiError,
  callGraphqlAsCustomer,
  callGraphqlAsInternalService,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CartItem = {
  cartId: string;
  userId: string;
  variantId: string;
  quantity: string;
};

const GET_CART_ITEMS = `query GetCartItems($userId: String, $sessionId: String) {
  getCartItems(userId: $userId, sessionId: $sessionId) {
    cartId
    userId
    variantId
    quantity
  }
}`;

const ADD_CART_ITEM = `mutation AddCartItem($input: NewCart!) {
  addCartItem(cartItem: $input) {
    cartId
    userId
    variantId
    quantity
  }
}`;

const UPDATE_CART_ITEM = `mutation UpdateCartItem($input: CartMutation!) {
  updateCartItem(cartItem: $input) {
    cartId
    userId
    variantId
    quantity
  }
}`;

const DELETE_CART_ITEM = `mutation DeleteCartItem($input: DeleteCartItem!) {
  deleteCartItem(delete: $input) {
    cartId
  }
}`;

function parseQty(raw: string): number {
  const n = Number.parseInt(raw, 10);
  if (!Number.isFinite(n) || n < 1) return 1;
  return n;
}

function capQuantity(n: number): number {
  return Math.min(Math.max(1, n), 999);
}

export async function POST(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as { guestSessionId?: unknown; idempotencyKey?: unknown };
  const guestSessionId = String(body.guestSessionId ?? "").trim();
  if (!guestSessionId) {
    return apiError("guestSessionId is required", 400, "VALIDATION_ERROR");
  }

  const mergeKey =
    String(body.idempotencyKey ?? "").trim() ||
    request.headers.get("idempotency-key")?.trim() ||
    `cart-merge-${customerUserId}-${guestSessionId}`;

  const [guestRes, userRes] = await Promise.all([
    callGraphqlAsInternalService<{ getCartItems?: CartItem[] }>(GET_CART_ITEMS, {
      userId: null,
      sessionId: guestSessionId,
    }),
    callGraphqlAsCustomer<{ getCartItems?: CartItem[] }>(customerUserId, GET_CART_ITEMS, {
      userId: customerUserId,
      sessionId: null,
    }),
  ]);

  if (guestRes.errors?.length) {
    return apiError(guestRes.errors[0]?.message ?? "Failed to load guest cart", 400, "GRAPHQL_ERROR");
  }
  if (userRes.errors?.length) {
    return apiError(userRes.errors[0]?.message ?? "Failed to load customer cart", 400, "GRAPHQL_ERROR");
  }

  const guestItems = guestRes.data?.getCartItems ?? [];
  if (guestItems.length === 0) {
    return Response.json({
      ok: true,
      data: { merged: 0, deletedGuestItems: 0 },
      errorCode: null,
      message: null,
      fieldErrors: null,
      retryable: false,
    });
  }

  const userByVariant = new Map<string, CartItem>();
  for (const item of userRes.data?.getCartItems ?? []) {
    if (!userByVariant.has(item.variantId)) userByVariant.set(item.variantId, item);
  }

  let merged = 0;
  let deletedGuestItems = 0;

  for (const guest of guestItems) {
    const existing = userByVariant.get(guest.variantId);
    const guestQty = parseQty(guest.quantity);

    if (existing) {
      const mergedQty = capQuantity(parseQty(existing.quantity) + guestQty);
      const upRes = await callGraphqlAsCustomer<{ updateCartItem?: CartItem[] }>(
        customerUserId,
        UPDATE_CART_ITEM,
        {
          input: {
            cartId: existing.cartId,
            userId: customerUserId,
            variantId: existing.variantId,
            quantity: String(mergedQty),
            sessionId: null,
          },
        },
        { "Idempotency-Key": `${mergeKey}:up:${guest.cartId}` }
      );
      if (upRes.errors?.length) {
        return apiError(upRes.errors[0]?.message ?? "Failed to merge cart item", 400, "GRAPHQL_ERROR");
      }
    } else {
      const addRes = await callGraphqlAsCustomer<{ addCartItem?: CartItem[] }>(
        customerUserId,
        ADD_CART_ITEM,
        {
          input: {
            userId: customerUserId,
            variantId: guest.variantId,
            quantity: String(capQuantity(guestQty)),
            sessionId: null,
          },
        },
        { "Idempotency-Key": `${mergeKey}:add:${guest.cartId}` }
      );
      if (addRes.errors?.length) {
        return apiError(addRes.errors[0]?.message ?? "Failed to merge cart item", 400, "GRAPHQL_ERROR");
      }
      const created = addRes.data?.addCartItem?.[0];
      if (created) userByVariant.set(created.variantId, created);
    }

    const delRes = await callGraphqlAsInternalService<{ deleteCartItem?: Array<{ cartId: string }> }>(
      DELETE_CART_ITEM,
      {
        input: {
          userId: "0",
          cartId: guest.cartId,
          sessionId: guestSessionId,
        },
      },
      { "Idempotency-Key": `${mergeKey}:del:${guest.cartId}` }
    );
    if (delRes.errors?.length) {
      return apiError(delRes.errors[0]?.message ?? "Failed to finalize merged guest cart", 400, "GRAPHQL_ERROR");
    }

    merged += 1;
    deletedGuestItems += 1;
  }

  return Response.json({
    ok: true,
    data: { merged, deletedGuestItems },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}