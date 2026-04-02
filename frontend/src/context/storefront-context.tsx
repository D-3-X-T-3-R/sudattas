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
import { useSession } from "next-auth/react";
import type { Product, CartLine } from "@/lib/schemas";
import { useToast } from "@/components/ui/toast";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { getGuestSessionId, ensureGuestSession } from "@/lib/session";
import {
  fetchCartLines,
  addCartItem,
  updateCartItem,
  deleteCartItem,
  type CartLineMapped,
} from "@/lib/cart-api";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { paiseToRupeesNumber } from "@/lib/money";

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
  const { status } = useSession();
  const [wishlist, setWishlist] = useState<Record<string, boolean>>({});
  const [cart, setCart] = useState<CartState>({});
  const [cartOpen, setCartOpen] = useState(false);
  const [cartLoading, setCartLoading] = useState(true);
  const { showToast } = useToast();
  const { announce } = useLiveAnnouncer();
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

  useEffect(() => {
    if (typeof window === "undefined") return;
    const onAuthChanged = () => {
      void reloadCartFromBackend();
    };
    window.addEventListener("sudattas-auth-changed", onAuthChanged);
    return () => window.removeEventListener("sudattas-auth-changed", onAuthChanged);
  }, [reloadCartFromBackend]);

  useEffect(() => {
    if (status !== "authenticated") return;
    let cancelled = false;
    void (async () => {
      try {
        const productIds = await fetchApiEnvelope<string[]>("/api/account/wishlist", {
          cache: "no-store",
        });
        if (cancelled) return;
        const next: Record<string, boolean> = {};
        for (const id of productIds ?? []) next[id] = true;
        setWishlist(next);
      } catch {
        if (!cancelled) {
          showToast({
            group: "wishlist",
            title: "Wishlist",
            description: "Could not sync wishlist right now.",
          });
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [status, showToast]);

  const toggleWish = useCallback(
    (p: Product) => {
      const authenticated = status === "authenticated";
      if (authenticated) {
        const currentlyWished = !!wishlist[p.id];
        const nextWished = !currentlyWished;
        setWishlist((prev) => ({ ...prev, [p.id]: nextWished }));
        void (async () => {
          try {
            await fetchApiEnvelope<boolean>("/api/account/wishlist", {
              method: nextWished ? "POST" : "DELETE",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ productId: p.id }),
            });
            showToast({
              group: "wishlist",
              title: "Wishlist",
              description: nextWished ? "Added to wishlist." : "Removed from wishlist.",
            });
            announce(nextWished ? "Added to wishlist." : "Removed from wishlist.");
          } catch {
            setWishlist((prev) => ({ ...prev, [p.id]: currentlyWished }));
            showToast({
              group: "wishlist",
              title: "Wishlist",
              description: "Could not update wishlist. Please retry.",
            });
            announce("Could not update wishlist. Please retry.", "assertive");
          }
        })();
        return;
      }

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
          announce(next ? "Added to wishlist." : "Removed from wishlist.");
          lastToastRef.current = { key: toastKey, at: now };
        }
      });
    },
    [showToast, status, wishlist]
  );

  const addToCart = useCallback(
    async (p: Product, qty = 1, sizeName?: string | null) => {
      const variantId = getVariantId(p, sizeName);
      const sessionId = (await ensureGuestSession()) ?? getGuestSessionId();

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
            announce(`Added ${p.name} to bag.`);
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

  const decCart = useCallback(async (id: string) => {
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
        if (variantId) {
          const lines = await updateCartItem(id, variantId, newQty, sessionId);
          if (lines) setCart(cartLinesToState(lines));
          else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        } else {
          showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
        }
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
  }, [cart, showToast]);

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
        else showToast({ group: "cart", title: "Bag", description: "Could not update bag right now. Please retry." });
      }
      return;
    }

    setCart((prev) => {
      const current = prev[id];
      if (!current) return prev;
      return { ...prev, [id]: { ...current, qty: current.qty + 1 } };
    });
  }, [cart, showToast]);

  const removeCart = useCallback(async (id: string) => {
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
  }, [showToast]);

  const cartLines = useMemo<CartLine[]>(() => Object.values(cart), [cart]);
  const cartCount = useMemo(
    () => cartLines.reduce((s, l) => s + l.qty, 0),
    [cartLines]
  );
  const cartSubtotal = useMemo(
    () =>
      paiseToRupeesNumber(
        cartLines.reduce(
          (s, l) => s + l.qty * (l.product.pricePaise ?? Math.round(l.product.price * 100)),
          0
        )
      ),
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
