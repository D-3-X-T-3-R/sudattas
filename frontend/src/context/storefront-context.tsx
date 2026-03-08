"use client";

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { Product, CartLine } from "@/lib/schemas";

type CartState = Record<string, { product: Product; qty: number }>;

type StorefrontContextValue = {
  wishlist: Record<string, boolean>;
  toggleWish: (p: Product) => void;
  cart: CartState;
  addToCart: (p: Product, qty?: number) => void;
  decCart: (productId: string) => void;
  incCart: (productId: string) => void;
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

  const toggleWish = useCallback((p: Product) => {
    setWishlist((prev) => ({ ...prev, [p.id]: !prev[p.id] }));
  }, []);

  const addToCart = useCallback((p: Product, qty = 1) => {
    setCart((prev) => {
      const existing = prev[p.id];
      const nextQty = existing ? existing.qty + qty : qty;
      return { ...prev, [p.id]: { product: p, qty: nextQty } };
    });
    setCartOpen(true);
  }, []);

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

  const cartLines = useMemo(() => Object.values(cart), [cart]);
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
