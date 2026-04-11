"use client";

import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  type ReactNode,
} from "react";
import { useSession } from "next-auth/react";
import type { CartLine, Product } from "@/lib/schemas";
import { useToast } from "@/components/ui/toast";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { paiseToRupeesNumber } from "@/lib/money";
import { useStorefrontWishlist } from "@/domains/storefront/hooks/use-storefront-wishlist";
import { useStorefrontCart } from "@/domains/storefront/hooks/use-storefront-cart";

type StorefrontContextValue = {
  wishlist: Record<string, boolean>;
  toggleWish: (p: Product) => void;
  addToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
  decCart: (lineId: string) => void;
  incCart: (lineId: string) => void;
  removeCart: (lineId: string) => void;
  cartLines: CartLine[];
  cartCount: number;
  cartSubtotal: number;
  wishCount: number;
  cartLoading: boolean;
};

const StorefrontContext = createContext<StorefrontContextValue | null>(null);

export function StorefrontProvider({ children }: { children: ReactNode }) {
  const { status } = useSession();
  const { showToast } = useToast();
  const { announce } = useLiveAnnouncer();

  const { wishlist, toggleWish } = useStorefrontWishlist({ status, showToast, announce });
  const {
    cartLoading,
    cartLines,
    addToCart,
    decCart,
    incCart,
    removeCart,
    reloadCartFromBackend,
  } = useStorefrontCart({ showToast, announce });

  useEffect(() => {
    if (typeof window === "undefined") return;
    const onAuthChanged = () => {
      void reloadCartFromBackend();
    };
    window.addEventListener("sudattas-auth-changed", onAuthChanged);
    return () => window.removeEventListener("sudattas-auth-changed", onAuthChanged);
  }, [reloadCartFromBackend]);

  const cartCount = useMemo(
    () => cartLines.reduce((sum, line) => sum + line.qty, 0),
    [cartLines]
  );
  const cartSubtotal = useMemo(
    () =>
      paiseToRupeesNumber(
        cartLines.reduce(
          (sum, line) => sum + line.qty * (line.product.pricePaise ?? Math.round(line.product.price * 100)),
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
      addToCart,
      decCart,
      incCart,
      removeCart,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
      cartLoading,
    }),
    [
      wishlist,
      toggleWish,
      addToCart,
      decCart,
      incCart,
      removeCart,
      cartLines,
      cartCount,
      cartSubtotal,
      wishCount,
      cartLoading,
    ]
  );

  return <StorefrontContext.Provider value={value}>{children}</StorefrontContext.Provider>;
}

export function useStorefront(): StorefrontContextValue {
  const ctx = useContext(StorefrontContext);
  if (!ctx) throw new Error("useStorefront must be used within StorefrontProvider");
  return ctx;
}
