import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  ensureGuestSession: vi.fn(async () => "guest-1"),
  getGuestSessionId: vi.fn(() => "guest-1"),
  fetchCartLines: vi.fn(async () => []),
  addCartItem: vi.fn(),
  updateCartItem: vi.fn(),
  deleteCartItem: vi.fn(),
}));

vi.mock("@/lib/session", () => ({
  ensureGuestSession: mocks.ensureGuestSession,
  getGuestSessionId: mocks.getGuestSessionId,
}));

vi.mock("@/lib/cart-api", () => ({
  fetchCartLines: mocks.fetchCartLines,
  addCartItem: mocks.addCartItem,
  updateCartItem: mocks.updateCartItem,
  deleteCartItem: mocks.deleteCartItem,
}));

import { useStorefrontCart } from "@/domains/storefront/hooks/use-storefront-cart";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

describe("useStorefrontCart in-flight dedupe", () => {
  it("prevents duplicate addToCart calls on rapid double click", async () => {
    const showToast = vi.fn();
    const announce = vi.fn();
    const addDef = deferred<never[] | null>();
    mocks.addCartItem.mockReturnValue(addDef.promise);

    const { result } = renderHook(() => useStorefrontCart({ showToast, announce }));

    const product = {
      id: "p1",
      name: "Saree",
      price: 100,
      pricePaise: 10000,
      rating: 4,
      reviews: 1,
      fabric: "Cotton",
      occasion: "Festive",
      description: "d",
      image: "x",
      imageAlt: "x",
      collection: "C",
      variantStock: [{ variantId: "v1", sizeId: "s", sizeName: "S", quantity: 2 }],
    };

    await act(async () => {
      void result.current.addToCart(product as never, 1, "S");
      void result.current.addToCart(product as never, 1, "S");
    });

    expect(mocks.addCartItem).toHaveBeenCalledTimes(1);

    await act(async () => {
      addDef.resolve([]);
      await Promise.resolve();
    });
  });
});
