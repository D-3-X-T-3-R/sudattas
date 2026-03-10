"use client";

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { Product, CartLine } from "@/lib/schemas";
import { useToast } from "@/components/ui/toast";

type CartState = Record<
  string,
  { id: string; product: Product; qty: number; sizeName?: string | null }
>;

type StorefrontContextValue = {
  wishlist: Record<string, boolean>;
  toggleWish: (p: Product) => void;
  cart: CartState;
  addToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
  decCart: (lineId: string) => void;
  incCart: (lineId: string) => void;
  cartOpen: boolean;
  setCartOpen: (open: boolean) => void;
  wishOpen: boolean;
  setWishOpen: (open: boolean) => void;
  cartLines: CartLine[];
  cartCount: number;
  cartSubtotal: number;
  wishCount: number;
};

const StorefrontContext = createContext<StorefrontContextValue | null>(null);

export function StorefrontProvider({ children }: { children: ReactNode }) {
  const [wishlist, setWishlist] = useState<Record<string, boolean>>({});
  const [cart, setCart] = useState<CartState>({});
  const [cartOpen, setCartOpen] = useState(false);
  const [wishOpen, setWishOpen] = useState(false);
  const { showToast } = useToast();
  const lastToastRef = useRef<{ key: string; at: number } | null>(null);

  const toggleWish = useCallback(
    (p: Product) => {
      setWishlist((prev) => {
        const next = !prev[p.id];
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
        return { ...prev, [p.id]: next };
      });
    },
    [showToast]
  );

  const addToCart = useCallback(
    (p: Product, qty = 1, sizeName?: string | null) => {
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

  const decCart = useCallback((id: string) => {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      if (line.qty <= 1) {
        const { [id]: _, ...rest } = prev;
        return rest;
      }
      return { ...prev, [id]: { ...line, qty: line.qty - 1 } };
    });
  }, []);

  const incCart = useCallback((id: string) => {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      return { ...prev, [id]: { ...line, qty: line.qty + 1 } };
    });
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
      wishOpen,
      setWishOpen,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
    }),
    [
      wishlist,
      toggleWish,
      cart,
      addToCart,
      decCart,
      incCart,
      cartOpen,
      wishOpen,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
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
