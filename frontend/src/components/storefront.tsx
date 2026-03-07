"use client";

import { useEffect, useMemo, useState } from "react";
import { useReducedMotion } from "framer-motion";
import { ensureGuestSession } from "@/lib/session";
import { PRODUCTS_SEED } from "@/lib/seed-data";
import type { Product, CartLine } from "@/lib/schemas";
import { useActiveSection } from "@/hooks/use-active-section";
import { useLockBodyScroll } from "@/hooks/use-lock-body-scroll";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { goTo } from "@/hooks/use-scroll-to";
import { AnnouncementBar } from "@/components/announcement-bar";
import { Header } from "@/components/header";
import { HeroSection } from "@/components/hero-section";
import { EditorialStrip } from "@/components/editorial-strip";
import { CollectionsSection } from "@/components/collections-section";
import { EditorialBlock } from "@/components/editorial-block";
import { SectionNav } from "@/components/section-nav";
import { ShopSection } from "@/components/shop-section";
import { StorySection } from "@/components/story-section";
import { Footer } from "@/components/footer";
import { MenuDrawer } from "@/components/menu-drawer";
import { CartDrawer } from "@/components/cart-drawer";
import { WishlistDrawer } from "@/components/wishlist-drawer";
import { QuickViewModal } from "@/components/quick-view-modal";
import { MobileBottomBar } from "@/components/mobile-bottom-bar";

export function Storefront() {
  const reduceMotion = useReducedMotion();
  const [menuOpen, setMenuOpen] = useState(false);
  const [cartOpen, setCartOpen] = useState(false);
  const [wishOpen, setWishOpen] = useState(false);
  const [quickView, setQuickView] = useState<Product | null>(null);
  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("All");
  const [occasion, setOccasion] = useState("All");
  const [sort, setSort] = useState("Featured");
  const [wishlist, setWishlist] = useState<Record<string, boolean>>({});
  const [cart, setCart] = useState<Record<string, { product: Product; qty: number }>>({});

  const { paymentMessage, paymentLoading, runTest } = useRazorpayTest();
  const activeSection = useActiveSection(["top", "collections", "shop", "story"]);
  useLockBodyScroll(menuOpen || cartOpen || wishOpen || !!quickView);

  useEffect(() => {
    ensureGuestSession();
  }, []);

  const occasions = useMemo(() => {
    const set = new Set(PRODUCTS_SEED.map((p) => p.occasion));
    return ["All", ...Array.from(set)];
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    let xs = PRODUCTS_SEED.filter((p) => {
      const matchesQuery =
        !q ||
        [p.name, p.collection, p.fabric, p.occasion]
          .join(" ")
          .toLowerCase()
          .includes(q);
      const matchesCollection =
        collection === "All" || p.collection === collection;
      const matchesOccasion = occasion === "All" || p.occasion === occasion;
      return matchesQuery && matchesCollection && matchesOccasion;
    });
    if (sort === "Price: Low") xs = [...xs].sort((a, b) => a.price - b.price);
    if (sort === "Price: High") xs = [...xs].sort((a, b) => b.price - a.price);
    if (sort === "Rating") xs = [...xs].sort((a, b) => b.rating - a.rating);
    return xs;
  }, [query, collection, occasion, sort]);

  const cartLines: CartLine[] = useMemo(() => Object.values(cart), [cart]);
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
  const wishedProducts = useMemo(
    () => PRODUCTS_SEED.filter((p) => wishlist[p.id]),
    [wishlist]
  );

  const toggleWish = (p: Product) => {
    setWishlist((prev) => ({ ...prev, [p.id]: !prev[p.id] }));
  };
  const addToCart = (p: Product) => {
    setCart((prev) => {
      const existing = prev[p.id];
      const nextQty = existing ? existing.qty + 1 : 1;
      return { ...prev, [p.id]: { product: p, qty: nextQty } };
    });
    setCartOpen(true);
  };
  const decCart = (id: string) => {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      if (line.qty <= 1) {
        const { [id]: _, ...rest } = prev;
        return rest;
      }
      return { ...prev, [id]: { ...line, qty: line.qty - 1 } };
    });
  };
  const incCart = (id: string) => {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      return { ...prev, [id]: { ...line, qty: line.qty + 1 } };
    });
  };

  const goToWithMotion = (id: string, instant?: boolean) =>
    goTo(id, instant ?? !!reduceMotion);

  return (
    <div
      id="top"
      className="min-h-screen bg-[var(--color-ivory)] text-[var(--color-ink)]"
    >
      <AnnouncementBar />
      <Header
        query={query}
        setQuery={setQuery}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={setMenuOpen}
        setCartOpen={setCartOpen}
        setWishOpen={setWishOpen}
        goTo={goToWithMotion}
      />

      <HeroSection />

      <EditorialStrip />

      <CollectionsSection
        setCollection={setCollection}
        reduceMotion={!!reduceMotion}
      />

      <ShopSection
        filtered={filtered}
        collection={collection}
        occasion={occasion}
        sort={sort}
        setCollection={setCollection}
        setOccasion={setOccasion}
        setSort={setSort}
        occasions={occasions}
        wishlist={wishlist}
        onToggleWish={toggleWish}
        onAddToCart={addToCart}
        onQuickView={setQuickView}
      />

      <EditorialBlock />

      <StorySection />

      <Footer goTo={goToWithMotion} />

      <MenuDrawer
        open={menuOpen}
        onClose={() => setMenuOpen(false)}
        setCollection={setCollection}
        reduceMotion={!!reduceMotion}
      />

      <WishlistDrawer
        open={wishOpen}
        onClose={() => setWishOpen(false)}
        wishCount={wishCount}
        wishedProducts={wishedProducts}
        onQuickView={setQuickView}
        onAddToCart={addToCart}
        onToggleWish={toggleWish}
      />

      <CartDrawer
        open={cartOpen}
        onClose={() => setCartOpen(false)}
        cartLines={cartLines}
        cartSubtotal={cartSubtotal}
        onDecCart={decCart}
        onIncCart={incCart}
        paymentLoading={paymentLoading}
        paymentMessage={paymentMessage}
        onTestRazorpay={runTest}
        onCheckout={() => alert("Checkout flow not wired yet")}
      />

      <QuickViewModal
        product={quickView}
        open={!!quickView}
        onClose={() => setQuickView(null)}
        wished={!!(quickView && wishlist[quickView.id])}
        onToggleWish={toggleWish}
        onAddToCart={addToCart}
      />

      <MobileBottomBar
        activeSection={activeSection}
        wishCount={wishCount}
        cartCount={cartCount}
        onWishOpen={() => setWishOpen(true)}
        onCartOpen={() => setCartOpen(true)}
        reduceMotion={!!reduceMotion}
      />

      <SectionNav />

      <div className="h-16 md:hidden" />
    </div>
  );
}
