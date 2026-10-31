import {
  apiError,
  callGraphqlAsCustomer,
  graphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CartItem = {
  cartId: string;
  userId: string;
  variantId: string;
  quantity: string;
  productDetails?: unknown;
};

const CART_FIELDS = `
  cartId
  userId
  variantId
  quantity
  productDetails {
    productId
    name
    description
    amountPaise
    formatted
    categoryId
    fabric
    occasion
    images { url thumbnailUrl }
    variantStock { variantId sizeId sizeName quantity }
  }
`;

const GET_CART_ITEMS = `query GetCartItems($userId: String, $sessionId: String) {
  getCartItems(userId: $userId, sessionId: $sessionId) {
    ${CART_FIELDS}
  }
}`;

const ADD_CART_ITEM = `mutation AddCartItem($input: NewCart!) {
  addCartItem(cartItem: $input) {
    ${CART_FIELDS}
  }
}`;

const UPDATE_CART_ITEM = `mutation UpdateCartItem($input: CartMutation!) {
  updateCartItem(cartItem: $input) {
    ${CART_FIELDS}
  }
}`;

const DELETE_CART_ITEM = `mutation DeleteCartItem($input: DeleteCartItem!) {
  deleteCartItem(delete: $input) {
    ${CART_FIELDS}
  }
}`;

function toPositiveInt(value: unknown): number | null {
  const num = Number.parseInt(String(value ?? ""), 10);
  return Number.isFinite(num) && num > 0 ? num : null;
}

export async function GET() {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const result = await callGraphqlAsCustomer<{ getCartItems?: CartItem[] }>(
    customerUserId,
    GET_CART_ITEMS,
    { userId: customerUserId, sessionId: null }
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to load cart");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.getCartItems ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function POST(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as { variantId?: unknown; quantity?: unknown };
  const variantId = String(body.variantId ?? "").trim();
  const quantity = toPositiveInt(body.quantity);
  if (!variantId || !quantity) {
    return apiError("variantId and quantity are required", 400, "VALIDATION_ERROR");
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();
  const result = await callGraphqlAsCustomer<{ addCartItem?: CartItem[] }>(
    customerUserId,
    ADD_CART_ITEM,
    {
      input: {
        userId: customerUserId,
        variantId,
        quantity: String(quantity),
        sessionId: null,
      },
    },
    idempotencyKey ? { "Idempotency-Key": idempotencyKey } : {}
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to add cart item");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.addCartItem ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function PATCH(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    cartId?: unknown;
    variantId?: unknown;
    quantity?: unknown;
  };
  const cartId = String(body.cartId ?? "").trim();
  const variantId = String(body.variantId ?? "").trim();
  const quantity = toPositiveInt(body.quantity);
  if (!cartId || !variantId || !quantity) {
    return apiError("cartId, variantId, and quantity are required", 400, "VALIDATION_ERROR");
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();
  const result = await callGraphqlAsCustomer<{ updateCartItem?: CartItem[] }>(
    customerUserId,
    UPDATE_CART_ITEM,
    {
      input: {
        cartId,
        userId: customerUserId,
        variantId,
        quantity: String(quantity),
        sessionId: null,
      },
    },
    idempotencyKey ? { "Idempotency-Key": idempotencyKey } : {}
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to update cart item");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.updateCartItem ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function DELETE(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as { cartId?: unknown };
  const cartId = String(body.cartId ?? "").trim();
  if (!cartId) {
    return apiError("cartId is required", 400, "VALIDATION_ERROR");
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();
  const result = await callGraphqlAsCustomer<{ deleteCartItem?: CartItem[] }>(
    customerUserId,
    DELETE_CART_ITEM,
    {
      input: {
        userId: customerUserId,
        cartId,
        sessionId: null,
      },
    },
    idempotencyKey ? { "Idempotency-Key": idempotencyKey } : {}
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to delete cart item");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.deleteCartItem ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}