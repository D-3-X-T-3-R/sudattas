import React, { useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import {
  Search,
  ShoppingBag,
  Heart,
  ChevronRight,
  ChevronLeft,
  Menu,
  X,
  Star,
} from "lucide-react";
import { gql } from "./api";

/**
 * LV-style design cues:
 * - Editorial, full-bleed hero
 * - Minimal header, centered wordmark
 * - High whitespace / low cognitive load
 * - Subtle micro-interactions (hover, scroll reveal, drawer)
 *
 * Brand palette (design.txt):
 * - Accent Green: #8BC34A
 * - Accent Brown: #6B3F1A
 * - Ink: #111111
 * - Ivory: #F7F5F0
 */

const THEME = {
  ivory: "#F7F5F0",
  ink: "#111111",
  muted: "#6B7280",
  line: "#E7E1D6",
  accentGreen: "#8BC34A",
  accentBrown: "#6B3F1A",
};

const INR = new Intl.NumberFormat("en-IN", {
  style: "currency",
  currency: "INR",
  maximumFractionDigits: 0,
});

function classNames(...xs) {
  return xs.filter(Boolean).join(" ");
}

const heroSlides = [
  {
    eyebrow: "NEW DROP",
    title: "Sudatta's Signature Sarees",
    cta: "Discover the collection",
    image:
      "https://images.unsplash.com/photo-1520975869011-7c1b3b0a4c7b?auto=format&fit=crop&w=2400&q=80",
    imageAlt: "Editorial fashion hero image",
    tone: "dark",
  },
  {
    eyebrow: "CRAFT • MODERN",
    title: "Borders that feel like jewellery",
    cta: "Shop best sellers",
    image:
      "https://images.unsplash.com/photo-1520975682038-6c155b610bdb?auto=format&fit=crop&w=2400&q=80",
    imageAlt: "Luxury editorial fabric detail",
    tone: "dark",
  },
];

const productsSeed = [
  {
    id: "p1",
    name: "Ivory Silk with Folk Border",
    collection: "Signature",
    price: 6499,
    rating: 4.7,
    reviews: 312,
    fabric: "Silk Blend",
    occasion: "Festive",
    description:
      "Clean ivory body with a statement border. Designed to look expensive without being loud.",
    image:
      "https://images.unsplash.com/photo-1520975682038-6c155b610bdb?auto=format&fit=crop&w=1200&q=80",
    hoverImage:
      "https://images.unsplash.com/photo-1520975681764-3c4a77d2d3e0?auto=format&fit=crop&w=1200&q=80",
    imageAlt: "Saree product image",
  },
  {
    id: "p2",
    name: "Violet Handpaint Floral",
    collection: "Studio",
    price: 5899,
    rating: 4.6,
    reviews: 184,
    fabric: "Silk",
    occasion: "Cocktail",
    description:
      "A modern floral story—editorial colors, fluid drape, and crisp finishing.",
    image:
      "https://images.unsplash.com/photo-1520975761338-fc4c7a8d3c3b?auto=format&fit=crop&w=1200&q=80",
    hoverImage:
      "https://images.unsplash.com/photo-1520975755572-4a71d1d0b2b7?auto=format&fit=crop&w=1200&q=80",
    imageAlt: "Saree product image",
  },
  {
    id: "p3",
    name: "Navy Minimal with Green Edge",
    collection: "Everyday Luxury",
    price: 4999,
    rating: 4.5,
    reviews: 221,
    fabric: "Cotton Silk",
    occasion: "Office",
    description:
      "Minimal motifs with a sharp edge. Built for repeat-wear and clean styling.",
    image:
      "https://images.unsplash.com/photo-1520975678339-fd3db6db45c1?auto=format&fit=crop&w=1200&q=80",
    hoverImage:
      "https://images.unsplash.com/photo-1520975672097-f72e8c8c0b69?auto=format&fit=crop&w=1200&q=80",
    imageAlt: "Saree product image",
  },
  {
    id: "p4",
    name: "Wedding Silk — Zari Whisper",
    collection: "Occasion",
    price: 12999,
    rating: 4.8,
    reviews: 97,
    fabric: "Banarasi Silk",
    occasion: "Wedding",
    description:
      "Dense craft, refined finish. Premium without the chaos.",
    image:
      "https://images.unsplash.com/photo-1520975745262-3c41b4fe2b47?auto=format&fit=crop&w=1200&q=80",
    hoverImage:
      "https://images.unsplash.com/photo-1520975748873-12a74f25b742?auto=format&fit=crop&w=1200&q=80",
    imageAlt: "Saree product image",
  },
];

const collections = [
  { key: "Signature", blurb: "Iconic borders. Clean bodies." },
  { key: "Studio", blurb: "Modern art on fabric." },
  { key: "Everyday Luxury", blurb: "Light, sharp, repeatable." },
  { key: "Occasion", blurb: "Wedding and festive statements." },
];

const motionEase = [0.22, 1, 0.36, 1];

function useActiveSection(ids) {
  const [active, setActive] = useState(ids[0] ?? "");
  const idsRef = useRef(ids);
  idsRef.current = ids;

  useEffect(() => {
    const targets = idsRef.current
      .map((id) => document.getElementById(id))
      .filter(Boolean);
    if (targets.length === 0) return;

    const obs = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => (b.intersectionRatio ?? 0) - (a.intersectionRatio ?? 0));
        const top = visible[0];
        if (top?.target?.id) setActive(top.target.id);
      },
      { threshold: [0.2, 0.35, 0.5], rootMargin: "-20% 0px -60% 0px" }
    );

    targets.forEach((t) => obs.observe(t));
    return () => obs.disconnect();
  }, []);

  return active;
}

function Rating({ value }) {
  const full = Math.floor(value);
  const half = value - full >= 0.5;
  return (
    <div className="flex items-center gap-1">
      {Array.from({ length: 5 }).map((_, i) => {
        const filled = i < full;
        const isHalf = i === full && half;
        return (
          <span key={i} aria-hidden className="inline-flex">
            <Star
              className={classNames(
                "h-4 w-4",
                filled || isHalf ? "text-[#111]" : "text-[#CFC7B8]"
              )}
              fill={filled || isHalf ? "currentColor" : "none"}
              style={{
                clipPath: isHalf
                  ? "polygon(0 0, 50% 0, 50% 100%, 0 100%)"
                  : undefined,
              }}
            />
          </span>
        );
      })}
      <span className="ml-1 text-xs text-[#6B7280]">{value.toFixed(1)}</span>
    </div>
  );
}

function Drawer({ open, title, children, onClose, side = "left" }) {
  const fromX = side === "left" ? -420 : 420;
  return (
    <AnimatePresence>
      {open ? (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 z-40 bg-black/40"
          />
          <motion.div
            initial={{ x: fromX }}
            animate={{ x: 0 }}
            exit={{ x: fromX }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            className={classNames(
              "fixed top-0 z-50 h-full w-full max-w-md bg-[var(--ivory)] shadow-2xl",
              side === "left" ? "left-0 border-r" : "right-0 border-l",
              "border-[var(--line)]"
            )}
            style={{ "--ivory": THEME.ivory, "--line": THEME.line }}
          >
            <div className="flex items-center justify-between border-b border-[var(--line)] p-4">
              <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">
                {title}
              </div>
              <button
                onClick={onClose}
                className="grid h-10 w-10 place-items-center rounded-full border border-[var(--line)] bg-[var(--ivory)] text-[#111] hover:bg-white"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <div className="h-[calc(100%-64px)] overflow-auto p-5">{children}</div>
          </motion.div>
        </>
      ) : null}
    </AnimatePresence>
  );
}

function ProductCard({
  product,
  wished,
  onToggleWish,
  onAddToCart,
  onQuickView,
  reduceMotion,
}) {
  const [hover, setHover] = useState(false);
  const img = hover && product.hoverImage ? product.hoverImage : product.image;

  return (
    <motion.div
      whileHover={reduceMotion ? undefined : { y: -3 }}
      transition={{ duration: 0.25, ease: motionEase }}
      className="group"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
    >
      <div className="relative overflow-hidden bg-white">
        <div className="aspect-[3/4] w-full">
          <img
            src={img}
            alt={product.imageAlt || product.name}
            className="h-full w-full object-cover transition duration-500 group-hover:scale-[1.02]"
            loading="lazy"
          />
        </div>
        <button
          onClick={() => onToggleWish(product)}
          className="absolute right-3 top-3 grid h-10 w-10 place-items-center rounded-full bg-white/80 backdrop-blur border border-[var(--line)] text-[#111] hover:bg-white"
          style={{ "--line": THEME.line }}
          aria-label={wished ? "Remove from wishlist" : "Add to wishlist"}
        >
          <Heart className={classNames("h-5 w-5", wished && "fill-[#111]")} />
        </button>

        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-black/30 to-transparent opacity-0 transition group-hover:opacity-100" />
        <div className="absolute inset-x-3 bottom-3 flex gap-2 opacity-0 transition group-hover:opacity-100">
          <button
            onClick={() => onQuickView(product)}
            className="pointer-events-auto flex-1 rounded-full bg-white/90 px-4 py-2 text-xs font-semibold text-[#111] backdrop-blur hover:bg-white"
          >
            Quick view
          </button>
          <button
            onClick={() => onAddToCart(product)}
            className="pointer-events-auto rounded-full px-4 py-2 text-xs font-semibold text-white"
            style={{ background: THEME.ink }}
          >
            Add
          </button>
        </div>
      </div>

      <div className="mt-4">
        <div className="text-[11px] tracking-[0.18em] text-[#6B7280]">
          {product.collection.toUpperCase()}
        </div>
        <div className="mt-1 text-sm font-semibold text-[#111] line-clamp-2">
          {product.name}
        </div>
        <div className="mt-2 flex items-center justify-between">
          <div className="text-sm font-semibold text-[#111]">{INR.format(product.price)}</div>
          <Rating value={product.rating} />
        </div>
      </div>
    </motion.div>
  );
}

function Modal({ open, title, children, onClose }) {
  return (
    <AnimatePresence>
      {open ? (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 z-40 bg-black/40"
          />
          <motion.div
            initial={{ opacity: 0, y: 18, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 18, scale: 0.98 }}
            transition={{ type: "spring", stiffness: 320, damping: 26 }}
            className="fixed left-1/2 top-1/2 z-50 w-[92vw] max-w-4xl -translate-x-1/2 -translate-y-1/2 overflow-hidden bg-[var(--ivory)] shadow-2xl"
            style={{ "--ivory": THEME.ivory }}
          >
            <div className="flex items-center justify-between border-b border-[var(--line)] p-4" style={{ "--line": THEME.line }}>
              <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">{title}</div>
              <button
                onClick={onClose}
                className="grid h-10 w-10 place-items-center rounded-full border border-[var(--line)] bg-[var(--ivory)] text-[#111] hover:bg-white"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <div className="p-5">{children}</div>
          </motion.div>
        </>
      ) : null}
    </AnimatePresence>
  );
}

function useLockBodyScroll(lock) {
  useEffect(() => {
    if (!lock) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, [lock]);
}

function goTo(id, reduceMotion) {
  if (id === "top") {
    window.scrollTo({ top: 0, behavior: reduceMotion ? "auto" : "smooth" });
    return;
  }
  const el = document.getElementById(id);
  el?.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth" });
}

const COLLECTION_IMAGES = [
  "https://images.unsplash.com/photo-1520975869011-7c1b3b0a4c7b?auto=format&fit=crop&w=1600&q=80",
  "https://images.unsplash.com/photo-1520975682038-6c155b610bdb?auto=format&fit=crop&w=1600&q=80",
  "https://images.unsplash.com/photo-1520975761338-fc4c7a8d3c3b?auto=format&fit=crop&w=1600&q=80",
  "https://images.unsplash.com/photo-1520975745262-3c41b4fe2b47?auto=format&fit=crop&w=1600&q=80",
];

export default function App() {
  const reduceMotion = useReducedMotion();

  const [menuOpen, setMenuOpen] = useState(false);
  const [cartOpen, setCartOpen] = useState(false);
  const [wishOpen, setWishOpen] = useState(false);
  const [quickView, setQuickView] = useState(null);

  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("All");
  const [occasion, setOccasion] = useState("All");
  const [sort, setSort] = useState("Featured");

  const [wishlist, setWishlist] = useState({});
  const [cart, setCart] = useState({});
  const [paymentMessage, setPaymentMessage] = useState(null);
  const [paymentLoading, setPaymentLoading] = useState(false);

  const [slide, setSlide] = useState(0);
  const activeSection = useActiveSection(["top", "collections", "shop", "story"]);

  useLockBodyScroll(menuOpen || cartOpen || wishOpen || !!quickView);

  const occasions = useMemo(() => {
    const set = new Set(productsSeed.map((p) => p.occasion));
    return ["All", ...Array.from(set)];
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    let xs = productsSeed.filter((p) => {
      const matchesQuery =
        !q ||
        [p.name, p.collection, p.fabric, p.occasion]
          .join(" ")
          .toLowerCase()
          .includes(q);
      const matchesCollection = collection === "All" || p.collection === collection;
      const matchesOccasion = occasion === "All" || p.occasion === occasion;
      return matchesQuery && matchesCollection && matchesOccasion;
    });

    if (sort === "Price: Low") xs = xs.slice().sort((a, b) => a.price - b.price);
    if (sort === "Price: High") xs = xs.slice().sort((a, b) => b.price - a.price);
    if (sort === "Rating") xs = xs.slice().sort((a, b) => b.rating - a.rating);

    return xs;
  }, [query, collection, occasion, sort]);

  const cartLines = useMemo(() => Object.values(cart), [cart]);
  const cartCount = useMemo(() => cartLines.reduce((s, l) => s + l.qty, 0), [cartLines]);
  const cartSubtotal = useMemo(
    () => cartLines.reduce((s, l) => s + l.qty * l.product.price, 0),
    [cartLines]
  );
  const wishCount = useMemo(() => Object.values(wishlist).filter(Boolean).length, [wishlist]);

  const wishedProducts = useMemo(
    () => productsSeed.filter((p) => wishlist[p.id]),
    [wishlist]
  );

  function toggleWish(p) {
    setWishlist((prev) => ({ ...prev, [p.id]: !prev[p.id] }));
  }

  function addToCart(p) {
    setCart((prev) => {
      const existing = prev[p.id];
      const nextQty = existing ? existing.qty + 1 : 1;
      return { ...prev, [p.id]: { product: p, qty: nextQty } };
    });
    setCartOpen(true);
  }

  function decCart(id) {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      if (line.qty <= 1) {
        const { [id]: _, ...rest } = prev;
        return rest;
      }
      return { ...prev, [id]: { ...line, qty: line.qty - 1 } };
    });
  }

  function incCart(id) {
    setCart((prev) => {
      const line = prev[id];
      if (!line) return prev;
      return { ...prev, [id]: { ...line, qty: line.qty + 1 } };
    });
  }

  function loadRazorpayScript() {
    if (window.Razorpay) return Promise.resolve();
    return new Promise((resolve, reject) => {
      const s = document.createElement("script");
      s.src = "https://checkout.razorpay.com/v1/checkout.js";
      s.onload = () => resolve();
      s.onerror = () => reject(new Error("Failed to load Razorpay"));
      document.body.appendChild(s);
    });
  }

  async function handleTestRazorpay() {
    setPaymentMessage(null);
    setPaymentLoading(true);
    try {
      const data = await gql(
        `mutation CreatePaymentIntent($input: NewPaymentIntent!) {
          createPaymentIntent(input: $input) {
            intentId
            razorpayOrderId
            razorpayKeyId
            orderId
            amountPaise
            currency
          }
        }`,
        {
          input: {
            orderId: "1",
            userId: "1",
            amountPaise: "10000",
            currency: "INR",
          },
        }
      );
      const intent = data?.createPaymentIntent?.[0];
      if (!intent?.razorpayKeyId || !intent?.razorpayOrderId) {
        setPaymentMessage("No Razorpay key/order returned. Check backend RAZORPAY_KEY_ID and order 1 exists.");
        return;
      }
      await loadRazorpayScript();
      const orderId = intent.orderId || "1";
      const options = {
        key: intent.razorpayKeyId,
        amount: intent.amountPaise,
        currency: intent.currency || "INR",
        order_id: intent.razorpayOrderId,
        name: "Sudatta's",
        description: "Test payment (₹100)",
        handler: async function (response) {
          try {
            await gql(
              `mutation VerifyRazorpay($input: VerifyRazorpayPaymentInput!) {
                verifyRazorpayPayment(input: $input) { verified paymentIntent { status } }
              }`,
              {
                input: {
                  orderId,
                  razorpayPaymentId: response.razorpay_payment_id,
                  razorpayOrderId: response.razorpay_order_id,
                  razorpaySignature: response.razorpay_signature,
                },
              }
            );
            setPaymentMessage("Payment verified successfully.");
          } catch (e) {
            setPaymentMessage("Verify failed: " + (e.message || String(e)));
          }
        },
      };
      const rzp = new window.Razorpay(options);
      rzp.on("payment.failed", () => {
        setPaymentMessage("Payment failed or was closed.");
      });
      rzp.open();
    } catch (e) {
      setPaymentMessage("Error: " + (e.message || String(e)));
    } finally {
      setPaymentLoading(false);
    }
  }

  function nextSlide() {
    setSlide((s) => (s + 1) % heroSlides.length);
  }
  function prevSlide() {
    setSlide((s) => (s - 1 + heroSlides.length) % heroSlides.length);
  }

  useEffect(() => {
    const t = window.setInterval(() => {
      if (reduceMotion) return;
      setSlide((s) => (s + 1) % heroSlides.length);
    }, 7000);
    return () => window.clearInterval(t);
  }, [reduceMotion]);

  return (
    <div
      id="top"
      className="min-h-screen"
      style={{ background: THEME.ivory, color: THEME.ink }}
    >
      {/* Announcement bar */}
      <div
        className="border-b text-[11px]"
        style={{ borderColor: THEME.line, color: "#374151" }}
      >
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-2">
          <span>Complimentary shipping above {INR.format(4999)} • 7-day returns</span>
          <span className="hidden sm:block">Support: WhatsApp-style chat (replace)</span>
        </div>
      </div>

      {/* Header */}
      <header
        className="sticky top-0 z-30 backdrop-blur"
        style={{ background: "rgba(247,245,240,0.8)", borderBottom: `1px solid ${THEME.line}` }}
      >
        <div className="mx-auto grid max-w-7xl grid-cols-3 items-center px-4 py-3">
          <div className="flex items-center gap-2">
            <button
              onClick={() => setMenuOpen(true)}
              className="grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
              style={{ "--ivory": THEME.ivory, borderColor: THEME.line }}
              aria-label="Open menu"
            >
              <Menu className="h-5 w-5" />
            </button>
            <button
              onClick={() => goTo("shop", !!reduceMotion)}
              className="hidden md:inline-flex items-center gap-2 text-xs font-semibold tracking-[0.18em]"
            >
              SHOP
              <ChevronRight className="h-4 w-4" />
            </button>
          </div>

          <button
            onClick={() => goTo("top", !!reduceMotion)}
            className="mx-auto flex items-center justify-center"
            aria-label="Go to top"
          >
            <div className="flex flex-col items-center">
              <div className="text-sm font-semibold tracking-[0.35em]">SUDATTA'S</div>
              <div className="text-[10px] tracking-[0.22em] text-[#6B7280]">DESIGNER BOUTIQUE</div>
            </div>
          </button>

          <div className="ml-auto flex items-center justify-end gap-2">
            <div className="hidden md:flex items-center">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#6B7280]" />
                <input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Search sarees, fabric, occasion"
                  className="w-[320px] rounded-full border bg-white/60 py-2.5 pl-10 pr-4 text-sm outline-none focus:bg-white"
                  style={{ borderColor: THEME.line }}
                />
              </div>
            </div>

            <button
              onClick={() => setWishOpen(true)}
              className="relative grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
              style={{ "--ivory": THEME.ivory, borderColor: THEME.line }}
              aria-label="Wishlist"
            >
              <Heart className="h-5 w-5" />
              {wishCount > 0 ? (
                <span
                  className="absolute -right-1 -top-1 grid h-6 w-6 place-items-center rounded-full text-xs font-semibold text-white"
                  style={{ background: THEME.ink }}
                >
                  {wishCount}
                </span>
              ) : null}
            </button>

            <button
              onClick={() => setCartOpen(true)}
              className="relative grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
              style={{ "--ivory": THEME.ivory, borderColor: THEME.line }}
              aria-label="Bag"
            >
              <ShoppingBag className="h-5 w-5" />
              {cartCount > 0 ? (
                <span
                  className="absolute -right-1 -top-1 grid h-6 w-6 place-items-center rounded-full text-xs font-semibold text-white"
                  style={{ background: THEME.ink }}
                >
                  {cartCount}
                </span>
              ) : null}
            </button>
          </div>
        </div>

        <div className="md:hidden border-t" style={{ borderColor: THEME.line }}>
          <div className="mx-auto max-w-7xl px-4 py-3">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#6B7280]" />
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search sarees, fabric, occasion"
                className="w-full rounded-full border bg-white/60 py-3 pl-10 pr-4 text-sm outline-none focus:bg-white"
                style={{ borderColor: THEME.line }}
              />
            </div>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className="relative">
        <div className="relative h-[72vh] min-h-[560px] w-full overflow-hidden">
          <AnimatePresence mode="wait">
            <motion.div
              key={slide}
              initial={{ opacity: 0, scale: 1.02 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 1.02 }}
              transition={{ duration: reduceMotion ? 0 : 0.7, ease: motionEase }}
              className="absolute inset-0"
            >
              <img
                src={heroSlides[slide].image}
                alt={heroSlides[slide].imageAlt}
                className="h-full w-full object-cover"
              />
              <div className="absolute inset-0 bg-black/35" />

              <div className="absolute inset-x-0 bottom-0">
                <div className="mx-auto max-w-7xl px-4 pb-12">
                  <div className="max-w-2xl">
                    <div className="text-[11px] font-semibold tracking-[0.24em] text-white/80">
                      {heroSlides[slide].eyebrow}
                    </div>
                    <h1 className="mt-3 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
                      {heroSlides[slide].title}
                    </h1>
                    <div className="mt-6 flex flex-col gap-3 sm:flex-row">
                      <button
                        onClick={() => goTo("shop", !!reduceMotion)}
                        className="rounded-full bg-white px-6 py-3 text-sm font-semibold text-[#111] hover:bg-white/90"
                      >
                        {heroSlides[slide].cta}
                      </button>
                      <button
                        onClick={() => goTo("collections", !!reduceMotion)}
                        className="rounded-full border border-white/40 bg-white/0 px-6 py-3 text-sm font-semibold text-white hover:bg-white/10"
                      >
                        Explore collections
                      </button>
                    </div>

                    <div className="mt-8 flex items-center gap-3 text-xs text-white/75">
                      <span className="inline-flex items-center gap-2">
                        <span className="h-1.5 w-1.5 rounded-full" style={{ background: THEME.accentGreen }} />
                        Premium finish, clean drape
                      </span>
                      <span className="hidden sm:inline">•</span>
                      <span className="hidden sm:inline">Ships in 24–48 hours</span>
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          </AnimatePresence>

          <button
            onClick={prevSlide}
            className="absolute left-4 top-1/2 -translate-y-1/2 grid h-11 w-11 place-items-center rounded-full bg-white/70 backdrop-blur hover:bg-white"
            aria-label="Previous"
          >
            <ChevronLeft className="h-5 w-5 text-[#111]" />
          </button>
          <button
            onClick={nextSlide}
            className="absolute right-4 top-1/2 -translate-y-1/2 grid h-11 w-11 place-items-center rounded-full bg-white/70 backdrop-blur hover:bg-white"
            aria-label="Next"
          >
            <ChevronRight className="h-5 w-5 text-[#111]" />
          </button>

          <div className="absolute bottom-5 left-1/2 -translate-x-1/2 flex items-center gap-2">
            {heroSlides.map((_, i) => (
              <button
                key={i}
                onClick={() => setSlide(i)}
                className="h-2.5 w-2.5 rounded-full border border-white/60"
                style={{ background: i === slide ? "white" : "transparent" }}
                aria-label={`Go to slide ${i + 1}`}
              />
            ))}
          </div>
        </div>
      </section>

      {/* Collections */}
      <section id="collections" className="mx-auto max-w-7xl px-4 py-14">
        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">COLLECTIONS</div>
            <h2 className="mt-2 text-2xl font-semibold tracking-tight">Shop by mood</h2>
          </div>
          <button
            onClick={() => goTo("shop", !!reduceMotion)}
            className="hidden sm:inline-flex items-center gap-2 text-xs font-semibold tracking-[0.18em]"
          >
            VIEW ALL
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>

        <div className="mt-8 grid gap-4 md:grid-cols-2">
          {collections.map((c, idx) => (
            <button
              key={c.key}
              onClick={() => {
                setCollection(c.key);
                goTo("shop", !!reduceMotion);
              }}
              className="group relative overflow-hidden bg-white"
            >
              <div className="aspect-[16/10] w-full">
                <img
                  src={COLLECTION_IMAGES[idx % COLLECTION_IMAGES.length]}
                  alt={c.key}
                  className="h-full w-full object-cover transition duration-500 group-hover:scale-[1.02]"
                  loading="lazy"
                />
              </div>
              <div className="absolute inset-0 bg-black/25" />
              <div className="absolute inset-x-0 bottom-0 p-6 text-left">
                <div className="text-[11px] font-semibold tracking-[0.24em] text-white/80">{c.key.toUpperCase()}</div>
                <div className="mt-2 text-lg font-semibold text-white">{c.blurb}</div>
                <div className="mt-4 inline-flex items-center gap-2 rounded-full bg-white px-4 py-2 text-xs font-semibold text-[#111]">
                  Explore
                  <ChevronRight className="h-4 w-4" />
                </div>
              </div>
            </button>
          ))}
        </div>
      </section>

      {/* Shop */}
      <section id="shop" className="mx-auto max-w-7xl px-4 pb-16">
        <div className="flex flex-col gap-4 border-y py-6" style={{ borderColor: THEME.line }}>
          <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
            <div>
              <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">SHOP</div>
              <h3 className="mt-2 text-2xl font-semibold tracking-tight">New arrivals</h3>
              <p className="mt-1 text-sm text-[#6B7280]">
                {filtered.length} item{filtered.length === 1 ? "" : "s"} • Collection: {collection} • Occasion: {occasion}
              </p>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[#6B7280]">COLLECTION</span>
                <select
                  value={collection}
                  onChange={(e) => setCollection(e.target.value)}
                  className="rounded-full border bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                  style={{ borderColor: THEME.line }}
                >
                  <option value="All">All</option>
                  {collections.map((c) => (
                    <option key={c.key} value={c.key}>{c.key}</option>
                  ))}
                </select>
              </div>

              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[#6B7280]">OCCASION</span>
                <select
                  value={occasion}
                  onChange={(e) => setOccasion(e.target.value)}
                  className="rounded-full border bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                  style={{ borderColor: THEME.line }}
                >
                  {occasions.map((o) => (
                    <option key={o} value={o}>{o}</option>
                  ))}
                </select>
              </div>

              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[#6B7280]">SORT</span>
                <select
                  value={sort}
                  onChange={(e) => setSort(e.target.value)}
                  className="rounded-full border bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                  style={{ borderColor: THEME.line }}
                >
                  <option>Featured</option>
                  <option>Price: Low</option>
                  <option>Price: High</option>
                  <option>Rating</option>
                </select>
              </div>
            </div>
          </div>

          {filtered.length === 0 ? (
            <div className="rounded-2xl bg-white p-6 text-sm text-[#374151]">
              No products match your filters.
            </div>
          ) : null}
        </div>

        <div className="mt-8 grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
          {filtered.map((p) => (
            <ProductCard
              key={p.id}
              product={p}
              wished={!!wishlist[p.id]}
              reduceMotion={!!reduceMotion}
              onToggleWish={toggleWish}
              onAddToCart={addToCart}
              onQuickView={(x) => setQuickView(x)}
            />
          ))}
        </div>
      </section>

      {/* Story */}
      <section id="story" className="mx-auto max-w-7xl px-4 pb-16">
        <div className="grid gap-10 md:grid-cols-2">
          <div className="border-t pt-10" style={{ borderColor: THEME.line }}>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">OUR POINT OF VIEW</div>
            <h3 className="mt-3 text-2xl font-semibold tracking-tight">Premium is calm.</h3>
            <p className="mt-4 text-sm leading-relaxed text-[#4B5563]">
              If your website feels busy, the product feels cheaper. We keep the layout clean, the typography deliberate,
              and the interactions subtle—so your sarees feel expensive before a user even scrolls.
            </p>

            <div className="mt-8 grid gap-3">
              {["Clarity over clutter", "Editorial imagery", "Details that feel intentional"].map((x) => (
                <div key={x} className="flex items-center gap-3">
                  <span className="h-2 w-2 rounded-full" style={{ background: THEME.accentBrown }} />
                  <span className="text-sm text-[#111]">{x}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="border-t pt-10" style={{ borderColor: THEME.line }}>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">STAY IN THE LOOP</div>
            <h3 className="mt-3 text-2xl font-semibold tracking-tight">Get the next drop first.</h3>
            <p className="mt-4 text-sm leading-relaxed text-[#4B5563]">Weekly releases. No spam. Unsubscribe anytime.</p>
            <div className="mt-6 flex flex-col gap-3 sm:flex-row">
              <input
                placeholder="you@example.com"
                className="flex-1 rounded-full border bg-white/70 px-5 py-3 text-sm outline-none focus:bg-white"
                style={{ borderColor: THEME.line }}
              />
              <button
                className="rounded-full px-6 py-3 text-sm font-semibold text-white"
                style={{ background: THEME.ink }}
                onClick={() => alert("Mock: wire to backend later")}
              >
                Notify me
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t" style={{ borderColor: THEME.line }}>
        <div className="mx-auto grid max-w-7xl gap-10 px-4 py-12 md:grid-cols-4">
          <div>
            <div className="text-sm font-semibold tracking-[0.35em]">SUDATTA'S</div>
            <div className="mt-1 text-[10px] tracking-[0.22em] text-[#6B7280]">DESIGNER BOUTIQUE</div>
            <p className="mt-4 text-sm leading-relaxed text-[#4B5563]">
              Replace the placeholder images with your saree photos and host them (S3/Cloudflare R2/etc.) for production.
            </p>
          </div>
          <div>
            <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">SHOP</div>
            <ul className="mt-4 space-y-2 text-sm text-[#4B5563]">
              <li>
                <button onClick={() => goTo("collections", !!reduceMotion)} className="hover:text-[#111]">
                  Collections
                </button>
              </li>
              <li>
                <button onClick={() => goTo("shop", !!reduceMotion)} className="hover:text-[#111]">
                  New arrivals
                </button>
              </li>
              <li>Gift cards</li>
            </ul>
          </div>
          <div>
            <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">SERVICES</div>
            <ul className="mt-4 space-y-2 text-sm text-[#4B5563]">
              <li>Shipping & delivery</li>
              <li>Returns & exchanges</li>
              <li>Care guide</li>
            </ul>
          </div>
          <div>
            <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">CONTACT</div>
            <ul className="mt-4 space-y-2 text-sm text-[#4B5563]">
              <li>support@sudattas.com</li>
              <li>+91 90000 00000</li>
              <li>Instagram: @sudattas</li>
            </ul>
            <div className="mt-6 text-xs text-[#6B7280]">© {new Date().getFullYear()} Sudatta's.</div>
          </div>
        </div>
      </footer>

      {/* Menu drawer */}
      <Drawer open={menuOpen} title="MENU" onClose={() => setMenuOpen(false)} side="left">
        <div className="space-y-8">
          <div className="space-y-3">
            {["New arrivals", "Collections", "Occasion", "Best sellers"].map((x) => (
              <button
                key={x}
                onClick={() => {
                  if (x === "Collections") goTo("collections", !!reduceMotion);
                  else goTo("shop", !!reduceMotion);
                  setMenuOpen(false);
                }}
                className="flex w-full items-center justify-between border-b pb-3 text-left"
                style={{ borderColor: THEME.line }}
              >
                <span className="text-sm font-semibold text-[#111]">{x}</span>
                <ChevronRight className="h-4 w-4 text-[#6B7280]" />
              </button>
            ))}
          </div>

          <div>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">COLLECTIONS</div>
            <div className="mt-4 grid grid-cols-2 gap-2">
              {collections.map((c) => (
                <button
                  key={c.key}
                  onClick={() => {
                    setCollection(c.key);
                    goTo("shop", !!reduceMotion);
                    setMenuOpen(false);
                  }}
                  className="rounded-full border bg-white px-4 py-2 text-sm hover:bg-white/80"
                  style={{ borderColor: THEME.line }}
                >
                  {c.key}
                </button>
              ))}
              <button
                onClick={() => {
                  setCollection("All");
                  goTo("shop", !!reduceMotion);
                  setMenuOpen(false);
                }}
                className="rounded-full border bg-white px-4 py-2 text-sm hover:bg-white/80"
                style={{ borderColor: THEME.line }}
              >
                All
              </button>
            </div>
          </div>

          <div className="rounded-2xl border bg-white p-4" style={{ borderColor: THEME.line }}>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">NOTE</div>
            <div className="mt-2 text-sm text-[#4B5563]">
              This is a visual mock. For production, replace placeholders with real product media and connect to your
              Rust + MySQL backend.
            </div>
          </div>
        </div>
      </Drawer>

      {/* Wishlist drawer */}
      <Drawer open={wishOpen} title={`WISHLIST (${wishCount})`} onClose={() => setWishOpen(false)} side="right">
        {wishedProducts.length === 0 ? (
          <div className="rounded-2xl bg-white p-6 text-sm text-[#374151]">No items yet.</div>
        ) : (
          <div className="space-y-4">
            {wishedProducts.map((p) => (
              <div key={p.id} className="border-b pb-4" style={{ borderColor: THEME.line }}>
                <div className="text-[11px] tracking-[0.18em] text-[#6B7280]">{p.collection.toUpperCase()}</div>
                <div className="mt-1 text-sm font-semibold text-[#111]">{p.name}</div>
                <div className="mt-2 flex items-center justify-between">
                  <div className="text-sm font-semibold">{INR.format(p.price)}</div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => setQuickView(p)}
                      className="rounded-full border bg-white px-4 py-2 text-xs font-semibold hover:bg-white/80"
                      style={{ borderColor: THEME.line }}
                    >
                      Quick view
                    </button>
                    <button
                      onClick={() => addToCart(p)}
                      className="rounded-full px-4 py-2 text-xs font-semibold text-white"
                      style={{ background: THEME.ink }}
                    >
                      Add
                    </button>
                  </div>
                </div>
                <button
                  onClick={() => toggleWish(p)}
                  className="mt-3 text-xs font-semibold tracking-[0.18em] text-[#6B7280] hover:text-[#111]"
                >
                  REMOVE
                </button>
              </div>
            ))}
          </div>
        )}
      </Drawer>

      {/* Cart drawer */}
      <Drawer open={cartOpen} title={`BAG (${cartCount})`} onClose={() => setCartOpen(false)} side="right">
        {cartLines.length === 0 ? (
          <div className="rounded-2xl bg-white p-6 text-sm text-[#374151]">Your bag is empty.</div>
        ) : (
          <div className="space-y-5">
            {cartLines.map(({ product, qty }) => (
              <div key={product.id} className="border-b pb-5" style={{ borderColor: THEME.line }}>
                <div className="text-[11px] tracking-[0.18em] text-[#6B7280]">{product.collection.toUpperCase()}</div>
                <div className="mt-1 text-sm font-semibold text-[#111]">{product.name}</div>
                <div className="mt-1 text-xs text-[#6B7280]">{product.fabric} • {product.occasion}</div>
                <div className="mt-4 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => decCart(product.id)}
                      className="grid h-10 w-10 place-items-center rounded-full border bg-white hover:bg-white/80"
                      style={{ borderColor: THEME.line }}
                      aria-label="Decrease"
                    >
                      −
                    </button>
                    <div className="min-w-10 text-center text-sm font-semibold">{qty}</div>
                    <button
                      onClick={() => incCart(product.id)}
                      className="grid h-10 w-10 place-items-center rounded-full border bg-white hover:bg-white/80"
                      style={{ borderColor: THEME.line }}
                      aria-label="Increase"
                    >
                      +
                    </button>
                  </div>
                  <div className="text-sm font-semibold">{INR.format(qty * product.price)}</div>
                </div>
              </div>
            ))}

            <div className="rounded-2xl bg-white p-5">
              <div className="flex items-center justify-between text-sm">
                <span className="text-[#6B7280]">Subtotal</span>
                <span className="font-semibold">{INR.format(cartSubtotal)}</span>
              </div>
              <div className="mt-2 text-xs text-[#6B7280]">Shipping and taxes calculated at checkout.</div>
              <button
                onClick={() => alert("Mock: connect to checkout later")}
                className="mt-4 w-full rounded-full px-5 py-3 text-sm font-semibold text-white"
                style={{ background: THEME.ink }}
              >
                Checkout
              </button>
              <button
                type="button"
                onClick={handleTestRazorpay}
                disabled={paymentLoading}
                className="mt-3 w-full rounded-full border px-5 py-3 text-sm font-semibold disabled:opacity-50"
                style={{ borderColor: THEME.line, color: THEME.accentBrown }}
              >
                {paymentLoading ? "Opening Razorpay…" : "Test Razorpay (₹100)"}
              </button>
              {paymentMessage && (
                <p className="mt-3 text-xs" style={{ color: THEME.muted }}>
                  {paymentMessage}
                </p>
              )}
            </div>
          </div>
        )}
      </Drawer>

      {/* Quick view modal */}
      <Modal open={!!quickView} title={quickView ? quickView.name.toUpperCase() : ""} onClose={() => setQuickView(null)}>
        {quickView ? (
          <div className="grid gap-8 md:grid-cols-2">
            <div className="bg-white">
              <div className="aspect-[3/4] w-full">
                <img
                  src={quickView.image}
                  alt={quickView.imageAlt || quickView.name}
                  className="h-full w-full object-cover"
                />
              </div>
            </div>
            <div>
              <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">
                {quickView.collection.toUpperCase()}
              </div>
              <div className="mt-2 text-2xl font-semibold tracking-tight">{quickView.name}</div>
              <div className="mt-4 flex items-center justify-between">
                <div className="text-lg font-semibold">{INR.format(quickView.price)}</div>
                <Rating value={quickView.rating} />
              </div>

              <p className="mt-5 text-sm leading-relaxed text-[#4B5563]">{quickView.description}</p>

              <div className="mt-6 space-y-2 text-sm">
                <div>
                  <span className="font-semibold">Fabric:</span> {quickView.fabric}
                </div>
                <div>
                  <span className="font-semibold">Occasion:</span> {quickView.occasion}
                </div>
              </div>

              <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                <button
                  onClick={() => toggleWish(quickView)}
                  className="rounded-full border bg-white px-6 py-3 text-sm font-semibold hover:bg-white/80"
                  style={{ borderColor: THEME.line }}
                >
                  {wishlist[quickView.id] ? "Wishlisted" : "Add to wishlist"}
                </button>
                <button
                  onClick={() => addToCart(quickView)}
                  className="rounded-full px-6 py-3 text-sm font-semibold text-white"
                  style={{ background: THEME.ink }}
                >
                  Add to bag
                </button>
              </div>

              <div className="mt-8 rounded-2xl border bg-white p-4" style={{ borderColor: THEME.line }}>
                <div className="text-[11px] font-semibold tracking-[0.24em] text-[#6B7280]">CARE</div>
                <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-[#4B5563]">
                  <li>Dry clean recommended for first wash.</li>
                  <li>Store folded with muslin.</li>
                  <li>Avoid direct perfume spray on zari.</li>
                </ul>
              </div>
            </div>
          </div>
        ) : null}
      </Modal>

      {/* Mobile bottom bar */}
      <div className="md:hidden fixed bottom-0 left-0 right-0 z-20 border-t bg-[rgba(247,245,240,0.88)] backdrop-blur" style={{ borderColor: THEME.line }}>
        <div className="mx-auto max-w-7xl px-4 py-2">
          <div className="flex items-center justify-between">
            <button
              onClick={() => goTo(activeSection === "top" ? "shop" : "top", !!reduceMotion)}
              className="text-xs font-semibold tracking-[0.18em]"
            >
              {activeSection === "top" ? "SHOP" : "TOP"}
            </button>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setWishOpen(true)}
                className="relative grid h-10 w-10 place-items-center rounded-full border bg-white"
                style={{ borderColor: THEME.line }}
                aria-label="Wishlist"
              >
                <Heart className="h-5 w-5" />
                {wishCount > 0 ? (
                  <span className="absolute -right-1 -top-1 grid h-5 w-5 place-items-center rounded-full text-[10px] font-semibold text-white" style={{ background: THEME.ink }}>
                    {wishCount}
                  </span>
                ) : null}
              </button>
              <button
                onClick={() => setCartOpen(true)}
                className="relative grid h-10 w-10 place-items-center rounded-full border bg-white"
                style={{ borderColor: THEME.line }}
                aria-label="Bag"
              >
                <ShoppingBag className="h-5 w-5" />
                {cartCount > 0 ? (
                  <span className="absolute -right-1 -top-1 grid h-5 w-5 place-items-center rounded-full text-[10px] font-semibold text-white" style={{ background: THEME.ink }}>
                    {cartCount}
                  </span>
                ) : null}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="h-16 md:hidden" />
    </div>
  );
}
