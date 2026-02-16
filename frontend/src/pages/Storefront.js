import React, { useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import {
  Search,
  ShoppingBag,
  Heart,
  Truck,
  ShieldCheck,
  RotateCcw,
  Star,
  SlidersHorizontal,
  Home,
  Grid3x3,
  X,
} from "lucide-react";

// Note: Tailwind is available in canvas preview.

const INR = new Intl.NumberFormat("en-IN", {
  style: "currency",
  currency: "INR",
  maximumFractionDigits: 0,
});

function classNames(...xs) {
  return xs.filter(Boolean).join(" ");
}

const collections = [
  { key: "Ajrakh", blurb: "Hand-block prints with bold geometry." },
  { key: "Chanderi", blurb: "Featherlight sheen for all-day wear." },
  { key: "Tussar", blurb: "Raw silk texture with earthy elegance." },
  { key: "Banarasi", blurb: "Classic zari woven for celebrations." },
];

const productsSeed = [
  {
    id: "p1",
    name: "Ajrakh on Chanderi — Ember",
    collection: "Ajrakh",
    price: 5499,
    tag: "Best Seller",
    rating: 4.7,
    reviews: 312,
    colorHints: ["Maroon", "Indigo", "Gold"],
    fabric: "Chanderi",
    occasion: "Festive",
    description:
      "Signature Ajrakh motifs on airy Chanderi. Clean borders, subtle sheen, effortless drape.",
  },
  {
    id: "p2",
    name: "Chanderi Tissue — Pearl",
    collection: "Chanderi",
    price: 6299,
    tag: "New",
    rating: 4.6,
    reviews: 184,
    colorHints: ["Ivory", "Silver"],
    fabric: "Chanderi",
    occasion: "Cocktail",
    description:
      "Tissue weave with a minimal pallu. Designed for modern silhouettes and statement blouses.",
  },
  {
    id: "p3",
    name: "Tussar Silk — Sandstone",
    collection: "Tussar",
    price: 7499,
    rating: 4.5,
    reviews: 221,
    colorHints: ["Beige", "Rust", "Olive"],
    fabric: "Tussar",
    occasion: "Office",
    description:
      "Matte Tussar body with contrast pallu. Understated luxury for workdays and dinners.",
  },
  {
    id: "p4",
    name: "Banarasi Zari — Noor",
    collection: "Banarasi",
    price: 12999,
    tag: "Limited",
    rating: 4.8,
    reviews: 97,
    colorHints: ["Emerald", "Gold"],
    fabric: "Banarasi Silk",
    occasion: "Wedding",
    description:
      "Dense zari motifs with a refined border. Made to photograph beautifully under warm lights.",
  },
  {
    id: "p5",
    name: "Ajrakh Modal — Midnight",
    collection: "Ajrakh",
    price: 3899,
    rating: 4.4,
    reviews: 404,
    colorHints: ["Indigo", "Black"],
    fabric: "Modal",
    occasion: "Everyday",
    description:
      "Soft modal drape with deep indigo prints. Lightweight, breathable, travel-friendly.",
  },
  {
    id: "p6",
    name: "Chanderi Floral — Rosé",
    collection: "Chanderi",
    price: 5799,
    rating: 4.6,
    reviews: 156,
    colorHints: ["Blush", "Champagne"],
    fabric: "Chanderi",
    occasion: "Festive",
    description:
      "Delicate florals, soft highlight border. A modern take on a classic Chanderi mood.",
  },
  {
    id: "p7",
    name: "Tussar Brushstroke — Sage",
    collection: "Tussar",
    price: 8199,
    tag: "New",
    rating: 4.5,
    reviews: 88,
    colorHints: ["Sage", "Cream"],
    fabric: "Tussar",
    occasion: "Festive",
    description:
      "Art-inspired pallu with a muted body. Built for those who want subtle but not boring.",
  },
  {
    id: "p8",
    name: "Banarasi Minimal — Aari",
    collection: "Banarasi",
    price: 9999,
    rating: 4.3,
    reviews: 62,
    colorHints: ["Wine", "Antique Gold"],
    fabric: "Banarasi Silk",
    occasion: "Wedding",
    description:
      "Clean motifs, sharp fall, timeless palette. For buyers who hate loud but love luxury.",
  },
];

const testimonials = [
  {
    quote:
      "The drape was effortless. Looked premium without screaming for attention — exactly what I wanted.",
    name: "Ananya",
    meta: "Kolkata",
    rating: 5,
  },
  {
    quote:
      "Fast shipping and the fabric quality is legit. The blouse suggestions on the site were useful.",
    name: "Riya",
    meta: "Bengaluru",
    rating: 5,
  },
  {
    quote:
      "Color matched the photos closely. Zero drama. I'll be back for a Banarasi next.",
    name: "Meera",
    meta: "Mumbai",
    rating: 4,
  },
];

// Motion system (fast, modern, consistent) + reduced-motion support.
const motionEase = [0.22, 1, 0.36, 1];

const fadeUp = {
  hidden: { opacity: 0, y: 14 },
  show: { opacity: 1, y: 0, transition: { duration: 0.55, ease: motionEase } },
};

const fadeIn = {
  hidden: { opacity: 0 },
  show: { opacity: 1, transition: { duration: 0.45, ease: motionEase } },
};

const stagger = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06 } },
};

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
      { root: null, threshold: [0.2, 0.35, 0.5, 0.65], rootMargin: "-20% 0px -55% 0px" }
    );

    targets.forEach((t) => obs.observe(t));
    return () => obs.disconnect();
  }, []);

  return active;
}

function Badge({ text }) {
  return (
    <span className="inline-flex items-center rounded-full border border-white/20 bg-white/10 px-2.5 py-1 text-xs font-medium text-white backdrop-blur">
      {text}
    </span>
  );
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
                filled || isHalf ? "text-[#2E7D32]" : "text-zinc-300"
              )}
              fill={filled || isHalf ? "currentColor" : "none"}
              style={{
                clipPath: isHalf ? "polygon(0 0, 50% 0, 50% 100%, 0 100%)" : undefined,
              }}
            />
          </span>
        );
      })}
      <span className="ml-1 text-xs text-zinc-600">{value.toFixed(1)}</span>
    </div>
  );
}

function LogoMark() {
  return (
    <div className="flex items-center gap-3">
      <div className="text-xl font-bold text-brand-green">SUDATTA'S</div>
    </div>
  );
}

function Pill({ children }) {
  return (
    <span className="inline-flex items-center rounded-full border border-[#e5e0d8] bg-white px-3 py-1 text-xs text-zinc-700 shadow-sm">
      {children}
    </span>
  );
}

function PrimaryButton({ children, onClick }) {
  return (
    <button
      onClick={onClick}
      className="inline-flex items-center justify-center rounded-2xl bg-[#7B3F00] px-5 py-3 text-sm font-medium text-white shadow-sm transition hover:bg-[#5c2f00] focus:outline-none focus:ring-2 focus:ring-zinc-900/20"
    >
      {children}
    </button>
  );
}

function SecondaryButton({ children, onClick }) {
  return (
    <button
      onClick={onClick}
      className="inline-flex items-center justify-center rounded-2xl border border-[#e5e0d8] bg-white px-5 py-3 text-sm font-medium text-[#2E7D32] shadow-sm transition hover:bg-zinc-50 focus:outline-none focus:ring-2 focus:ring-zinc-900/10"
    >
      {children}
    </button>
  );
}

function IconChip({ icon, title, subtitle }) {
  return (
    <div className="flex items-start gap-3 rounded-3xl border border-[#e5e0d8] bg-white p-4 shadow-sm">
      <div className="grid h-10 w-10 place-items-center rounded-2xl bg-[#7B3F00] text-white">
        {icon}
      </div>
      <div>
        <div className="text-sm font-semibold text-[#2E7D32]">{title}</div>
        <div className="mt-0.5 text-xs leading-relaxed text-zinc-600">{subtitle}</div>
      </div>
    </div>
  );
}

// Component continues in next part due to length...
export default function SareeStore() {
  // Implementation in next file
  return <div>Saree Store Component - Split into parts for readability</div>;
}
