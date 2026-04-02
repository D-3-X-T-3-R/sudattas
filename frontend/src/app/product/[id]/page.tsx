"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SiteHeader } from "@/components/site-header";
import { ProductDetailView } from "@/components/product-detail-view";
import { useStorefront } from "@/context/storefront-context";
import type { Product } from "@/lib/schemas";
import { ensureGuestSession, getGuestSessionId, setGuestSessionId } from "@/lib/session";
import { toRouteFailureUi } from "@/lib/route-state";

export default function ProductPage() {
  const params = useParams();
  const router = useRouter();
  const id = typeof params.id === "string" ? params.id : "";
  const { wishlist, toggleWish, addToCart } = useStorefront();

  const [product, setProduct] = useState<Product | null>(null);
  const [sizes, setSizes] = useState<{ sizeId: string; sizeName: string }[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) {
      setLoading(false);
      setError("This product could not be found.");
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    void (async () => {
      try {
        await ensureGuestSession();
        if (cancelled) return;
        const sessionId = getGuestSessionId();
        const headers: Record<string, string> = {};
        if (sessionId) headers["X-Session-Id"] = sessionId;
        const res = await fetch(`/api/products/${id}`, { headers });
        const newSid = res.headers.get("X-Set-Guest-Session")?.trim();
        if (newSid) setGuestSessionId(newSid);
        const data = (await res.json()) as {
          product: Product | null;
          sizes?: { sizeId: string; sizeName: string }[];
          error: string | null;
        };
        if (cancelled) return;
        if (data.error) {
          setError(toRouteFailureUi("public", new Error(data.error)).message);
        }
        else {
          setProduct(data.product ?? null);
          setSizes(data.sizes ?? []);
        }
      } catch (e) {
        if (!cancelled) {
          setError(toRouteFailureUi("public", e).message);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id]);

  const goToHome = () => router.push("/");

  return (
    <div className="min-h-screen bg-[var(--color-ivory)] text-[var(--color-ink)]">
      <SiteHeader />

      <div className="mx-auto min-w-0 max-w-[2000px] px-4 py-4">
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
    </div>
  );
}
