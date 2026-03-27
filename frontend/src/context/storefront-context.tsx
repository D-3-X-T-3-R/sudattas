"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { Product, CartLine } from "@/lib/schemas";
import { useToast } from "@/components/ui/toast";
import { getGuestSessionId } from "@/lib/session";
import {
  fetchCartLines,
  addCartItem,
  updateCartItem,
  deleteCartItem,
  type CartLineMapped,
} from "@/lib/cart-api";

type CartState = Record<
  string,
  { id: string; product: Product; qty: number; sizeName?: string | null }
>;

const CART_CACHE_KEY = "sudattas_cart_cache_v1";

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

/** Resolve variant_id for addToCart from product + size. */
function getVariantId(p: Product, sizeName?: string | null): string | null {
  const stock = p.variantStock ?? [];
  if (stock.length === 0) return null;
  if (sizeName && sizeName !== "Free Size") {
    const row = stock.find((v) => v.sizeName === sizeName);
    return row?.variantId ?? null;
  }
  return stock[0]?.variantId ?? null;
}

/** True if line id is a backend cart_id (numeric string). */
function isBackendCartId(id: string): boolean {
  return /^\d+$/.test(id);
}

type StorefrontContextValue = {
  wishlist: Record<string, boolean>;
  toggleWish: (p: Product) => void;
  cart: CartState;
  addToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
  decCart: (lineId: string) => void;
  incCart: (lineId: string) => void;
  removeCart: (lineId: string) => void;
  cartOpen: boolean;
  setCartOpen: (open: boolean) => void;
  cartLines: CartLine[];
  cartCount: number;
  cartSubtotal: number;
  wishCount: number;
  cartLoading: boolean;
};

const StorefrontContext = createContext<StorefrontContextValue | null>(null);

export function StorefrontProvider({ children }: { children: ReactNode }) {
  const [wishlist, setWishlist] = useState<Record<string, boolean>>({});
  const [cart, setCart] = useState<CartState>({});
  const [cartOpen, setCartOpen] = useState(false);
  const [cartLoading, setCartLoading] = useState(true);
  const { showToast } = useToast();
  const lastToastRef = useRef<{ key: string; at: number } | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      const raw = window.localStorage.getItem(CART_CACHE_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as CartState;
      if (parsed && typeof parsed === "object") {
        setCart(parsed);
      }
    } catch {
      // ignore malformed cache
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    setCartLoading(true);
    fetchCartLines()
      .then((lines) => {
        if (cancelled || lines == null) return;
        setCart(cartLinesToState(lines));
      })
      .finally(() => {
        if (!cancelled) setCartLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(CART_CACHE_KEY, JSON.stringify(cart));
    } catch {
      // ignore storage quota errors
    }
  }, [cart]);

  const toggleWish = useCallback(
    (p: Product) => {
      let next = false;
      setWishlist((prev) => {
        next = !prev[p.id];
        return { ...prev, [p.id]: next };
      });
      // Defer toast — must not call showToast inside setWishlist updater (updates another provider during render).
      queueMicrotask(() => {
        const now = Date.now();
        const toastKey = `wish-${p.id}-${next ? "on" : "off"}`;
        const last = lastToastRef.current;
        if (!last || last.key !== toastKey || now - last.at > 200) {
          showToast({
            group: "wishlist",
            title: "Wishlist",
            description: next ? "Added to wishlist." : "Removed from wishlist.",
          });
          lastToastRef.current = { key: toastKey, at: now };
        }
      });
    },
    [showToast]
  );

  const addToCart = useCallback(
    async (p: Product, qty = 1, sizeName?: string | null) => {
      const variantId = getVariantId(p, sizeName);
      const sessionId = getGuestSessionId();

      if (variantId && sessionId) {
        const lines = await addCartItem(variantId, qty, sessionId);
        if (lines) {
          setCart(cartLinesToState(lines));
          const now = Date.now();
          const toastKey = `cart-add-${p.id}`;
          const last = lastToastRef.current;
          if (!last || last.key !== toastKey || now - last.at > 200) {
            showToast({
              group: "cart",
              title: "Bag",
              description: "Added to bag.",
            });
            lastToastRef.current = { key: toastKey, at: now };
          }
          return;
        }
      }

      // Fallback: local-only when no variantId or API failed
      const key = `${p.id}__${sizeName ?? "nosize"}`;
      setCart((prev) => {
        const existing = prev[key];
        const nextQty = existing ? existing.qty + qty : qty;
        return {
          ...prev,
          [key]: { id: key, product: p, qty: nextQty, sizeName: sizeName ?? null },
        };
      });
      const now = Date.now();
      const toastKey = `cart-${key}`;
      const last = lastToastRef.current;
      if (!last || last.key !== toastKey || now - last.at > 200) {
        showToast({
          group: "cart",
          title: "Bag",
          description: "Added to bag.",
        });
        lastToastRef.current = { key: toastKey, at: now };
      }
    },
    [showToast]
  );

  const decCart = useCallback(async (id: string) => {
    const line = cart[id];
    if (!line) return;
    const sessionId = getGuestSessionId();

    if (isBackendCartId(id) && sessionId) {
      const newQty = line.qty - 1;
      if (newQty < 1) {
        const lines = await deleteCartItem(id, sessionId);
        if (lines) setCart(cartLinesToState(lines));
        else
          setCart((prev) => {
            const { [id]: _, ...rest } = prev;
            return rest;
          });
      } else {
        const variantId = getVariantId(line.product, line.sizeName);
        if (variantId) {
          const lines = await updateCartItem(id, variantId, newQty, sessionId);
          if (lines) setCart(cartLinesToState(lines));
          else
            setCart((prev) => ({
              ...prev,
              [id]: { ...line, qty: newQty },
            }));
        } else {
          setCart((prev) => {
            const { [id]: _, ...rest } = prev;
            return rest;
          });
        }
      }
      return;
    }

    setCart((prev) => {
      const current = prev[id];
      if (!current) return prev;
      if (current.qty <= 1) {
        const { [id]: _, ...rest } = prev;
        return rest;
      }
      return { ...prev, [id]: { ...current, qty: current.qty - 1 } };
    });
  }, [cart]);

  const incCart = useCallback(async (id: string) => {
    const line = cart[id];
    if (!line) return;
    const sessionId = getGuestSessionId();

    if (isBackendCartId(id) && sessionId) {
      const variantId = getVariantId(line.product, line.sizeName);
      if (variantId) {
        const newQty = line.qty + 1;
        const lines = await updateCartItem(id, variantId, newQty, sessionId);
        if (lines) setCart(cartLinesToState(lines));
        else
          setCart((prev) => ({ ...prev, [id]: { ...line, qty: newQty } }));
      }
      return;
    }

    setCart((prev) => {
      const current = prev[id];
      if (!current) return prev;
      return { ...prev, [id]: { ...current, qty: current.qty + 1 } };
    });
  }, [cart]);

  const removeCart = useCallback(async (id: string) => {
    const sessionId = getGuestSessionId();
    if (isBackendCartId(id) && sessionId) {
      const lines = await deleteCartItem(id, sessionId);
      if (lines) setCart(cartLinesToState(lines));
      else setCart((prev) => { const { [id]: _, ...rest } = prev; return rest; });
      return;
    }
    setCart((prev) => { const { [id]: _, ...rest } = prev; return rest; });
  }, []);

  const cartLines = useMemo<CartLine[]>(() => Object.values(cart), [cart]);
  const cartCount = useMemo(
    () => cartLines.reduce((s, l) => s + l.qty, 0),
    [cartLines]
  );
  const cartSubtotal = useMemo(
    () => cartLines.reduce((s, l) => s + l.qty * l.product.price, 0),
    [cartLines]
  );
  const wishCount = useMemo(
    () => Object.values(wishlist).filter(Boolean).length,
    [wishlist]
  );

  const value = useMemo(
    () => ({
      wishlist,
      toggleWish,
      cart,
      addToCart,
      decCart,
      incCart,
      cartOpen,
      setCartOpen,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
      cartLoading,
      removeCart,
    }),
    [
      wishlist,
      toggleWish,
      cart,
      addToCart,
      decCart,
      incCart,
      removeCart,
      cartOpen,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
      cartLoading,
    ]
  );

  return (
    <StorefrontContext.Provider value={value}>
      {children}
    </StorefrontContext.Provider>
  );
}

export function useStorefront(): StorefrontContextValue {
  const ctx = useContext(StorefrontContext);
  if (!ctx)
    throw new Error("useStorefront must be used within StorefrontProvider");
  return ctx;
}
