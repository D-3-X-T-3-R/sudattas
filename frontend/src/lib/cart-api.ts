/**
 * Cart API: persist bag to backend via GraphQL.
 * Uses guest session (X-Session-Id) from graphqlClient.
 */

import { gql } from "@/lib/graphqlClient";
import { getGuestSessionId } from "@/lib/session";
import { getAccessToken } from "@/lib/authStore";
import type { Product } from "@/lib/schemas";
import { parsePaise, paiseToRupeesNumber } from "@/lib/money";

/** Cart line as returned by getCartItems (GraphQL). */
export interface CartItemGql {
  cartId: string;
  userId: string;
  variantId: string;
  quantity: string;
  productDetails: CartProductDetail[];
}

export interface CartProductDetail {
  productId: string;
  name: string;
  description?: string | null;
  amountPaise: string;
  formatted: string;
  categoryId?: string | null;
  fabric?: string | null;
  occasion?: string | null;
  images?: { url?: string | null; thumbnailUrl?: string | null }[] | null;
  variantStock?: { variantId?: string; sizeId: string; sizeName: string; quantity: number }[] | null;
}

/** Map GraphQL cart product to storefront Product shape (minimal for cart line). */
function mapCartProductToProduct(d: CartProductDetail, categoryName?: string): Product {
  const pricePaise = parsePaise(d.amountPaise);
  const priceRupees = paiseToRupeesNumber(pricePaise);
  const imageList = d.images?.filter((i) => i?.url || i?.thumbnailUrl) ?? [];
  const allUrls = imageList.map((i) => i?.url || i?.thumbnailUrl || "").filter(Boolean);
  return {
    id: d.productId,
    name: d.name,
    collection: categoryName ?? "Collection",
    price: priceRupees,
    pricePaise,
    priceFormatted: d.formatted?.trim() || undefined,
    rating: 4.5,
    reviews: 0,
    fabric: d.fabric ?? "",
    occasion: d.occasion ?? "",
    description: d.description ?? "",
    image: allUrls[0] ?? "",
    hoverImage: allUrls[1],
    images: allUrls.length > 0 ? allUrls : undefined,
    imageAlt: d.name,
    variantStock: d.variantStock?.map((v) => ({
      variantId: v.variantId,
      sizeId: v.sizeId,
      sizeName: v.sizeName,
      quantity: v.quantity,
    })),
  };
}

/** Get size name for a cart line from product's variantStock by variantId. */
function sizeNameForVariant(
  variantId: string,
  variantStock?: { variantId?: string; sizeName: string }[] | null
): string | null {
  if (!variantStock?.length) return "Free Size";
  const row = variantStock.find((v) => v.variantId === variantId);
  return row?.sizeName ?? null;
}

const GET_CART_ITEMS = `query GetCartItems($userId: String, $sessionId: String) {
  getCartItems(userId: $userId, sessionId: $sessionId) {
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
  }
}`;

const ADD_CART_ITEM = `mutation AddCartItem($input: NewCart!) {
  addCartItem(cartItem: $input) {
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
  }
}`;

const UPDATE_CART_ITEM = `mutation UpdateCartItem($input: CartMutation!) {
  updateCartItem(cartItem: $input) {
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
  }
}`;

const DELETE_CART_ITEM = `mutation DeleteCartItem($input: DeleteCartItem!) {
  deleteCartItem(delete: $input) {
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
  }
}`;

export type CartLineMapped = {
  id: string;
  product: Product;
  qty: number;
  sizeName: string | null;
};

function decodeJwtSubAsNumericUserId(token: string | null): string | null {
  if (!token) return null;
  try {
    const raw = token.startsWith("Bearer ") ? token.slice(7) : token;
    const parts = raw.split(".");
    if (parts.length < 2) return null;
    const payload = JSON.parse(atob(parts[1])) as { sub?: string };
    const sub = String(payload.sub ?? "").trim();
    return /^\d+$/.test(sub) ? sub : null;
  } catch {
    return null;
  }
}

function cartScope(preferredSessionId?: string): {
  userId: string;
  sessionId?: string;
} {
  const userId = decodeJwtSubAsNumericUserId(getAccessToken());
  if (userId) {
    return { userId, sessionId: undefined };
  }
  const sid = preferredSessionId || getGuestSessionId() || undefined;
  return { userId: "", sessionId: sid };
}

/**
 * Fetch cart from backend and return lines in storefront shape.
 * Returns null on API/session failure so caller can keep existing local state.
 */
export async function fetchCartLines(): Promise<CartLineMapped[] | null> {
  if (typeof window === "undefined") return null;
  const { ensureGuestSession } = await import("@/lib/session");
  await ensureGuestSession();
  const scope = cartScope();
  if (!scope.userId && !scope.sessionId) return null;

  try {
    const data = await gql<{ getCartItems?: CartItemGql[] }>(GET_CART_ITEMS, {
      userId: scope.userId || null,
      sessionId: scope.sessionId || null,
    });
    const items = data?.getCartItems ?? [];
    return items.map((item) => {
      const productDetail = item.productDetails?.[0];
      if (!productDetail) {
        return null;
      }
      const product = mapCartProductToProduct(productDetail);
      const sizeName = sizeNameForVariant(item.variantId, productDetail.variantStock);
      return {
        id: item.cartId,
        product,
        qty: parseInt(item.quantity, 10) || 1,
        sizeName,
      };
    }).filter((line): line is CartLineMapped => line != null);
  } catch {
    return null;
  }
}

/**
 * Add item to cart on backend. Returns updated cart lines or null on error.
 */
export async function addCartItem(
  variantId: string,
  quantity: number,
  sessionId?: string
): Promise<CartLineMapped[] | null> {
  try {
    const scope = cartScope(sessionId);
    const data = await gql<{ addCartItem?: CartItemGql[] }>(ADD_CART_ITEM, {
      input: {
        userId: scope.userId,
        variantId,
        quantity: String(quantity),
        sessionId: scope.sessionId || null,
      },
    });
    const items = data?.addCartItem ?? [];
    return items.map((item) => {
      const productDetail = item.productDetails?.[0];
      if (!productDetail) return null;
      const product = mapCartProductToProduct(productDetail);
      const sizeName = sizeNameForVariant(item.variantId, productDetail.variantStock);
      return {
        id: item.cartId,
        product,
        qty: parseInt(item.quantity, 10) || 1,
        sizeName,
      };
    }).filter((line): line is CartLineMapped => line != null);
  } catch {
    return null;
  }
}

/**
 * Update cart line quantity. Returns updated cart lines or null on error.
 */
export async function updateCartItem(
  cartId: string,
  variantId: string,
  quantity: number,
  sessionId?: string
): Promise<CartLineMapped[] | null> {
  try {
    const scope = cartScope(sessionId);
    const data = await gql<{ updateCartItem?: CartItemGql[] }>(UPDATE_CART_ITEM, {
      input: {
        cartId,
        userId: scope.userId,
        variantId,
        quantity: String(quantity),
        sessionId: scope.sessionId || null,
      },
    });
    const items = data?.updateCartItem ?? [];
    return items.map((item) => {
      const productDetail = item.productDetails?.[0];
      if (!productDetail) return null;
      const product = mapCartProductToProduct(productDetail);
      const sizeName = sizeNameForVariant(item.variantId, productDetail.variantStock);
      return {
        id: item.cartId,
        product,
        qty: parseInt(item.quantity, 10) || 1,
        sizeName,
      };
    }).filter((line): line is CartLineMapped => line != null);
  } catch {
    return null;
  }
}

/**
 * Delete cart line. Returns updated cart lines or null on error.
 */
export async function deleteCartItem(
  cartId: string,
  sessionId?: string
): Promise<CartLineMapped[] | null> {
  try {
    const scope = cartScope(sessionId);
    const data = await gql<{ deleteCartItem?: CartItemGql[] }>(DELETE_CART_ITEM, {
      input: {
        userId: scope.userId,
        cartId,
        sessionId: scope.sessionId || null,
      },
    });
    const items = data?.deleteCartItem ?? [];
    return items.map((item) => {
      const productDetail = item.productDetails?.[0];
      if (!productDetail) return null;
      const product = mapCartProductToProduct(productDetail);
      const sizeName = sizeNameForVariant(item.variantId, productDetail.variantStock);
      return {
        id: item.cartId,
        product,
        qty: parseInt(item.quantity, 10) || 1,
        sizeName,
      };
    }).filter((line): line is CartLineMapped => line != null);
  } catch {
    return null;
  }
}
