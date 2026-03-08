"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/header";
import { CartDrawer } from "@/components/cart-drawer";
import { WishlistDrawer } from "@/components/wishlist-drawer";
import { ProductDetailView } from "@/components/product-detail-view";
import { useStorefront } from "@/context/storefront-context";
import type { Product } from "@/lib/schemas";
import { getGuestSessionId } from "@/lib/session";

export default function ProductPage() {
  const params = useParams();
  const router = useRouter();
  const id = typeof params.id === "string" ? params.id : "";
  const {
    wishlist,
    toggleWish,
    addToCart,
    decCart,
    incCart,
    cartOpen,
    setCartOpen,
    wishOpen,
    setWishOpen,
    cartLines,
    cartSubtotal,
    cartCount,
    wishCount,
  } = useStorefront();

  const [product, setProduct] = useState<Product | null>(null);
  const [sizes, setSizes] = useState<{ sizeId: string; sizeName: string }[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) {
      setLoading(false);
      setError("Invalid product");
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    const sessionId = getGuestSessionId();
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    fetch(`/api/products/${id}`, { headers })
      .then((res) => res.json())
      .then((data: { product: Product | null; sizes?: { sizeId: string; sizeName: string }[]; error: string | null }) => {
        if (cancelled) return;
        if (data.error) setError(data.error);
        else {
          setProduct(data.product ?? null);
          setSizes(data.sizes ?? []);
        }
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : "Failed to load");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  const goToHome = () => router.push("/");
  const wishedProducts: Product[] = product && wishlist[product.id] ? [product] : [];

  return (
    <div className="min-h-screen bg-[var(--color-ivory)] text-[var(--color-ink)]">
      <Header
        query=""
        setQuery={() => {}}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={() => {}}
        setCartOpen={setCartOpen}
        setWishOpen={setWishOpen}
        goTo={goToHome}
      />

      <div className="mx-auto min-w-0 max-w-7xl px-4 py-4">
        <Button
          variant="ghost"
          size="sm"
          className="mb-4 -ml-2 text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          asChild
        >
          <Link href="/" className="flex items-center gap-2">
            <ArrowLeft className="h-4 w-4" />
            Back to shop
          </Link>
        </Button>

        {loading && (
          <div className="flex min-h-[50vh] items-center justify-center">
            <p className="text-sm text-[var(--color-muted)]">Loading…</p>
          </div>
        )}
        {error && !loading && (
          <div className="flex min-h-[50vh] flex-col items-center justify-center gap-4">
            <p className="text-sm text-[var(--color-muted)]">{error}</p>
            <Button variant="outline" onClick={goToHome}>
              Back to shop
            </Button>
          </div>
        )}
        {product && !loading && (
          <ProductDetailView
            product={product}
            sizes={sizes}
            wished={!!wishlist[product.id]}
            onToggleWish={toggleWish}
            onAddToCart={addToCart}
          />
        )}
      </div>

      <CartDrawer
        open={cartOpen}
        onClose={() => setCartOpen(false)}
        cartLines={cartLines}
        cartSubtotal={cartSubtotal}
        onDecCart={decCart}
        onIncCart={incCart}
        paymentLoading={false}
        paymentMessage={null}
        onTestRazorpay={() => {}}
        onCheckout={() => alert("Checkout flow not wired yet")}
      />
      <WishlistDrawer
        open={wishOpen}
        onClose={() => setWishOpen(false)}
        wishCount={wishCount}
        wishedProducts={wishedProducts}
        onQuickView={(p) => router.push(`/product/${p.id}`)}
        onAddToCart={addToCart}
        onToggleWish={toggleWish}
      />
    </div>
  );
}
