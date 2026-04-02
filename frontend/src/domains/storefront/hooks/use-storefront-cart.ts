import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Product } from "@/lib/schemas";
import { getGuestSessionId, ensureGuestSession } from "@/lib/session";
import {
  fetchCartLines,
  addCartItem,
  updateCartItem,
  deleteCartItem,
  type CartLineMapped,
} from "@/lib/cart-api";

type ToastArgs = { group?: string; title: string; description: string };

type CartState = Record<
  string,
  { id: string; product: Product; qty: number; sizeName?: string | null }
>;

function cartLinesToState(lines: CartLineMapped[]): CartState {
  const state: CartState = {};
  for (const line of lines) {
    state[line.id] = {
      id: line.id,
      product: line.product,
      qty: line.qty,
      sizeName: line.sizeName ?? null,
    };
  }
  return state;
}

function getVariantId(product: Product, sizeName?: string | null): string | null {
  const stock = product.variantStock ?? [];
  if (stock.length === 0) return null;
  if (sizeName && sizeName !== "Free Size") {
    const row = stock.find((variant) => variant.sizeName === sizeName);
    return row?.variantId ?? null;
  }
  return stock[0]?.variantId ?? null;
}

function isBackendCartId(id: string): boolean {
  return /^\d+$/.test(id);
}

type UseStorefrontCartProps = {
  showToast: (args: ToastArgs) => void;
  announce: (message: string, politeness?: "polite" | "assertive") => void;
};

export function useStorefrontCart({ showToast, announce }: UseStorefrontCartProps) {
  const [cart, setCart] = useState<CartState>({});
  const [cartLoading, setCartLoading] = useState(true);
  const lastToastRef = useRef<{ key: string; at: number } | null>(null);

  const reloadCartFromBackend = useCallback(async () => {
    setCartLoading(true);
    try {
      await ensureGuestSession();
      const lines = await fetchCartLines();
      setCart(lines ? cartLinesToState(lines) : {});
    } finally {
      setCartLoading(false);
    }
  }, []);

  useEffect(() => {
    void reloadCartFromBackend();
  }, [reloadCartFromBackend]);

  const addToCart = useCallback(
    async (product: Product, qty = 1, sizeName?: string | null) => {
      const variantId = getVariantId(product, sizeName);
      const sessionId = (await ensureGuestSession()) ?? getGuestSessionId();

      if (variantId && sessionId) {
        const lines = await addCartItem(variantId, qty, sessionId);
        if (lines) {
          setCart(cartLinesToState(lines));
          const now = Date.now();
          const toastKey = `cart-add-${product.id}`;
          const last = lastToastRef.current;
          if (!last || last.key !== toastKey || now - last.at > 200) {
            showToast({ group: "cart", title: "Bag", description: "Added to bag." });
            announce(`Added ${product.name} to bag.`);
            lastToastRef.current = { key: toastKey, at: now };
          }
          return;
        }
      }

      showToast({
        group: "cart",
        title: "Bag",
        description: "Could not update bag right now. Please retry.",
      });
      announce("Could not update bag right now. Please retry.", "assertive");
    },
    [announce, showToast]
  );

  const decCart = useCallback(
    async (id: string) => {
      const line = cart[id];
      if (!line) return;
      const sessionId = getGuestSessionId();

      if (isBackendCartId(id) && sessionId) {
        const newQty = line.qty - 1;
        if (newQty < 1) {
          const lines = await deleteCartItem(id, sessionId);
          if (lines) setCart(cartLinesToState(lines));
          else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        } else {
          const variantId = getVariantId(line.product, line.sizeName);
          if (!variantId) {
            showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
            return;
          }
          const lines = await updateCartItem(id, variantId, newQty, sessionId);
          if (lines) setCart(cartLinesToState(lines));
          else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        }
        return;
      }

      setCart((prev) => {
        const current = prev[id];
        if (!current) return prev;
        if (current.qty <= 1) {
          const rest = { ...prev };
          delete rest[id];
          return rest;
        }
        return { ...prev, [id]: { ...current, qty: current.qty - 1 } };
      });
    },
    [cart, showToast]
  );

  const incCart = useCallback(
    async (id: string) => {
      const line = cart[id];
      if (!line) return;
      const sessionId = getGuestSessionId();

      if (isBackendCartId(id) && sessionId) {
        const variantId = getVariantId(line.product, line.sizeName);
        if (!variantId) return;
        const lines = await updateCartItem(id, variantId, line.qty + 1, sessionId);
        if (lines) setCart(cartLinesToState(lines));
        else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        return;
      }

      setCart((prev) => {
        const current = prev[id];
        if (!current) return prev;
        return { ...prev, [id]: { ...current, qty: current.qty + 1 } };
      });
    },
    [cart, showToast]
  );

  const removeCart = useCallback(
    async (id: string) => {
      const sessionId = getGuestSessionId();
      if (isBackendCartId(id) && sessionId) {
        const lines = await deleteCartItem(id, sessionId);
        if (lines) setCart(cartLinesToState(lines));
        else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        return;
      }

      setCart((prev) => {
        const rest = { ...prev };
        delete rest[id];
        return rest;
      });
    },
    [showToast]
  );

  const cartLines = useMemo(() => Object.values(cart), [cart]);

  return {
    cart,
    cartLoading,
    cartLines,
    addToCart,
    decCart,
    incCart,
    removeCart,
    reloadCartFromBackend,
  };
}
