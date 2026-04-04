import { useCallback, useEffect, useRef, useState } from "react";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import type { Product } from "@/lib/schemas";

type ToastArgs = { group?: string; title: string; description: string };

type UseStorefrontWishlistProps = {
  status: "authenticated" | "loading" | "unauthenticated";
  showToast: (args: ToastArgs) => void;
  announce: (message: string, politeness?: "polite" | "assertive") => void;
};

export function useStorefrontWishlist({
  status,
  showToast,
  announce,
}: UseStorefrontWishlistProps) {
  const [wishlist, setWishlist] = useState<Record<string, boolean>>({});
  const lastToastRef = useRef<{ key: string; at: number } | null>(null);

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
    (product: Product) => {
      if (status === "authenticated") {
        const currentlyWished = !!wishlist[product.id];
        const nextWished = !currentlyWished;
        setWishlist((prev) => ({ ...prev, [product.id]: nextWished }));
        void (async () => {
          try {
            await fetchApiEnvelope<boolean>("/api/account/wishlist", {
              method: nextWished ? "POST" : "DELETE",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ productId: product.id }),
            });
            showToast({
              group: "wishlist",
              title: "Wishlist",
              description: nextWished ? "Added to wishlist." : "Removed from wishlist.",
            });
            announce(nextWished ? "Added to wishlist." : "Removed from wishlist.");
          } catch {
            setWishlist((prev) => ({ ...prev, [product.id]: currentlyWished }));
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

      let nextWished = false;
      setWishlist((prev) => {
        nextWished = !prev[product.id];
        return { ...prev, [product.id]: nextWished };
      });

      queueMicrotask(() => {
        const now = Date.now();
        const toastKey = `wish-${product.id}-${nextWished ? "on" : "off"}`;
        const last = lastToastRef.current;
        if (!last || last.key !== toastKey || now - last.at > 200) {
          showToast({
            group: "wishlist",
            title: "Wishlist",
            description: nextWished ? "Added to wishlist." : "Removed from wishlist.",
          });
          announce(nextWished ? "Added to wishlist." : "Removed from wishlist.");
          lastToastRef.current = { key: toastKey, at: now };
        }
      });
    },
    [announce, showToast, status, wishlist]
  );

  return { wishlist, toggleWish };
}
