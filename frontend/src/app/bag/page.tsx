"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { useSession } from "next-auth/react";
import { SiteHeader } from "@/components/site-header";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { getGuestSessionId } from "@/lib/session";
import { BagEmptyState } from "@/domains/bag/components/bag-empty-state";
import { BagContent } from "@/domains/bag/components/bag-content";
import { BagMobileCheckoutBar } from "@/domains/bag/components/bag-mobile-checkout-bar";

type CatalogSize = { sizeId: string; sizeName: string };

export default function BagPage() {
  const router = useRouter();
  const { status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const { cartLines, decCart, incCart, removeCart, toggleWish, wishlist, addToCart } = useStorefront();
  const reduceMotion = !!useReducedMotion();

  const [selectedLineIds, setSelectedLineIds] = useState<Set<string>>(new Set());
  const [openSizeForId, setOpenSizeForId] = useState<string | null>(null);
  const [catalogSizes, setCatalogSizes] = useState<CatalogSize[]>([]);

  useEffect(() => {
    const currentIds = new Set(cartLines.map((line) => line.id));
    setSelectedLineIds((prev) => new Set([...prev].filter((id) => currentIds.has(id))));
  }, [cartLines]);

  useEffect(() => {
    if (cartLines.length > 0 && selectedLineIds.size === 0) {
      setSelectedLineIds(new Set(cartLines.map((line) => line.id)));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cartLines.length]);

  useEffect(() => {
    const sessionId = getGuestSessionId();
    if (!sessionId) return;
    void fetch("/api/sizes", { headers: { "x-session-id": sessionId } })
      .then((response) => response.json())
      .then((data: { sizes?: CatalogSize[] }) => setCatalogSizes(data.sizes ?? []))
      .catch(() => setCatalogSizes([]));
  }, []);

  const allSelected = cartLines.length > 0 && selectedLineIds.size === cartLines.length;
  const selectedLines = useMemo(
    () => cartLines.filter((line) => selectedLineIds.has(line.id)),
    [cartLines, selectedLineIds]
  );
  const selectedCount = useMemo(
    () => selectedLines.reduce((sum, line) => sum + line.qty, 0),
    [selectedLines]
  );
  const selectedSubtotal = useMemo(
    () =>
      selectedLines.reduce(
        (sum, line) => sum + line.qty * ((line.product.pricePaise ?? Math.round(line.product.price * 100)) / 100),
        0
      ),
    [selectedLines]
  );

  const handleCheckout = () => {
    if (status !== "authenticated") {
      openLogin("/checkout/address");
      return;
    }
    router.push("/checkout/address");
  };

  return (
    <div className="min-h-screen w-full min-w-0 bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] text-[var(--foreground)]">
      <SiteHeader />
      <div className="mx-auto w-full max-w-7xl rounded-[36px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.82),rgba(255,255,255,0.72))] p-4 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-6 lg:h-[calc(100vh-100px)] lg:overflow-hidden lg:p-8">
        {cartLines.length === 0 ? (
          <BagEmptyState />
        ) : (
          <BagContent
            cartLines={cartLines}
            selectedLineIds={selectedLineIds}
            selectedLines={selectedLines}
            selectedSubtotal={selectedSubtotal}
            selectedCount={selectedCount}
            allSelected={allSelected}
            catalogSizes={catalogSizes}
            openSizeForId={openSizeForId}
            reduceMotion={reduceMotion}
            wishlist={wishlist}
            onToggleAll={() =>
              setSelectedLineIds(allSelected ? new Set() : new Set(cartLines.map((line) => line.id)))
            }
            onToggleOne={(id) =>
              setSelectedLineIds((prev) => {
                const next = new Set(prev);
                if (next.has(id)) next.delete(id);
                else next.add(id);
                return next;
              })
            }
            onSetOpenSizeForId={setOpenSizeForId}
            onDecCart={decCart}
            onIncCart={incCart}
            onRemoveCart={removeCart}
            onToggleWish={toggleWish}
            onAddToCart={addToCart}
            onCheckout={handleCheckout}
          />
        )}
      </div>

      {cartLines.length > 0 && (
        <BagMobileCheckoutBar
          selectedSubtotal={selectedSubtotal}
          selectedCount={selectedCount}
          onCheckout={handleCheckout}
        />
      )}
    </div>
  );
}
