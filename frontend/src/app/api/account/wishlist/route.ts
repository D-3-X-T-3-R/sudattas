import {
  apiError,
  callGraphql,
  decodeJwtSub,
  requireSessionToken,
} from "@/lib/server-session-auth";

type WishlistRow = {
  wishlistId: string;
  userId: string;
  productId: string;
  dateAdded: string;
};

const LIST_QUERY = `query AccountWishlist($search: SearchWishlistItem!) {
  searchWishlistItem(search: $search) {
    wishlistId
    userId
    productId
    dateAdded
  }
}`;

const ADD_MUTATION = `mutation AddWishlistItem($wishlist: NewWishlistItem!) {
  addWishlistItem(wishlist: $wishlist) {
    wishlistId
    userId
    productId
    dateAdded
  }
}`;

const DELETE_MUTATION = `mutation DeleteWishlistItem($delete: DeleteWishlistItem!) {
  deleteWishlistItem(delete: $delete) {
    wishlistId
  }
}`;

function normalizeProductId(raw: unknown): string {
  return String(raw ?? "").trim();
}

export async function GET() {
  const token = await requireSessionToken();
  if (!token) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const userId = decodeJwtSub(token);
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const result = await callGraphql<{ searchWishlistItem?: WishlistRow[] }>(
    token,
    LIST_QUERY,
    { search: { userId, productId: null, wishlistId: null } }
  );
  if (result.errors?.length) {
    return apiError(
      result.errors[0]?.message ?? "Failed to load wishlist",
      400,
      "GRAPHQL_ERROR"
    );
  }
  const ids = (result.data?.searchWishlistItem ?? []).map((w) => w.productId);
  return Response.json({
    ok: true,
    data: ids,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function POST(request: Request) {
  const token = await requireSessionToken();
  if (!token) return apiError("Unauthorized", 401, "UNAUTHORIZED");
  const userId = decodeJwtSub(token);
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const body = (await request.json().catch(() => ({}))) as { productId?: unknown };
  const productId = normalizeProductId(body.productId);
  if (!productId) {
    return apiError("productId is required", 400, "VALIDATION_ERROR");
  }

  const add = await callGraphql<{ addWishlistItem?: WishlistRow[] }>(
    token,
    ADD_MUTATION,
    { wishlist: { userId, productId } }
  );
  if (add.errors?.length) {
    return apiError(
      add.errors[0]?.message ?? "Failed to add wishlist item",
      400,
      "GRAPHQL_ERROR"
    );
  }
  return Response.json({
    ok: true,
    data: true,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function DELETE(request: Request) {
  const token = await requireSessionToken();
  if (!token) return apiError("Unauthorized", 401, "UNAUTHORIZED");
  const userId = decodeJwtSub(token);
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const body = (await request.json().catch(() => ({}))) as { productId?: unknown };
  const productId = normalizeProductId(body.productId);
  if (!productId) {
    return apiError("productId is required", 400, "VALIDATION_ERROR");
  }

  const list = await callGraphql<{ searchWishlistItem?: WishlistRow[] }>(
    token,
    LIST_QUERY,
    { search: { userId, productId, wishlistId: null } }
  );
  if (list.errors?.length) {
    return apiError(
      list.errors[0]?.message ?? "Failed to resolve wishlist item",
      400,
      "GRAPHQL_ERROR"
    );
  }
  const hit = list.data?.searchWishlistItem?.[0];
  if (!hit?.wishlistId) {
    return Response.json({
      ok: true,
      data: true,
      errorCode: null,
      message: null,
      fieldErrors: null,
      retryable: false,
    });
  }

  const del = await callGraphql<{ deleteWishlistItem?: Array<{ wishlistId: string }> }>(
    token,
    DELETE_MUTATION,
    { delete: { userId, wishlistId: hit.wishlistId } }
  );
  if (del.errors?.length) {
    return apiError(
      del.errors[0]?.message ?? "Failed to remove wishlist item",
      400,
      "GRAPHQL_ERROR"
    );
  }

  return Response.json({
    ok: true,
    data: true,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
