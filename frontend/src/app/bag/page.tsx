"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { ChevronDown, ShoppingBag } from "lucide-react";
import { useStorefront } from "@/context/storefront-context";
import { INR } from "@/lib/constants";
import { formatInrFromPaise } from "@/lib/money";
import { SiteHeader } from "@/components/site-header";
import { getGuestSessionId } from "@/lib/session";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { useState, useEffect, useRef } from "react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useSession } from "next-auth/react";
import { useStorefrontLogin } from "@/context/storefront-login-context";

type CatalogSize = { sizeId: string; sizeName: string };

function CheckIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d="m5 12 5 5L20 7" />
    </svg>
  );
}

function BagIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d="M6 8h12l-1 11H7L6 8Z" />
      <path d="M9 8a3 3 0 0 1 6 0" />
    </svg>
  );
}

function HeartIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d="M12 20s-6.7-4.35-9-8.2C1.3 8.6 3.1 5 7 5c2.2 0 3.6 1.2 5 3 1.4-1.8 2.8-3 5-3 3.9 0 5.7 3.6 4 6.8-2.3 3.85-9 8.2-9 8.2Z" />
    </svg>
  );
}

function TrashIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d="M3 6h18" />
      <path d="M8 6V4h8v2" />
      <path d="m19 6-1 14H6L5 6" />
      <path d="M10 11v6" />
      <path d="M14 11v6" />
    </svg>
  );
}

function ArrowRightIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d="M5 12h14" />
      <path d="m13 5 7 7-7 7" />
    </svg>
  );
}

function GoldDivider() {
  return (
    <div className="flex items-center gap-3">
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
      <div className="h-1.5 w-1.5 rounded-full bg-[#C9A646]" />
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
    </div>
  );
}

/** Only sizes that are in stock for this product. */
function buildSizeOptions(
  product: Product,
  catalog: CatalogSize[]
): { sizeId: string; sizeName: string }[] {
  const stock = product.variantStock ?? [];
  const byId = new Map(stock.map((v) => [v.sizeId, v]));

  if (catalog.length > 0) {
    return catalog
      .filter((s) => s.sizeName.toLowerCase() !== "free size")
      .map((s) => {
        const v = byId.get(s.sizeId);
        if (!v || v.quantity <= 0) return null;
        return { sizeId: s.sizeId, sizeName: s.sizeName };
      })
      .filter((x): x is { sizeId: string; sizeName: string } => x !== null);
  }

  const seen = new Set<string>();
  return stock
    .filter((v) => {
      if (v.sizeName.toLowerCase() === "free size") return false;
      if (v.quantity <= 0) return false;
      if (seen.has(v.sizeId)) return false;
      seen.add(v.sizeId);
      return true;
    })
    .map((v) => ({
      sizeId: v.sizeId,
      sizeName: v.sizeName,
    }));
}

function BagSizeDropdown({
  options,
  sizeName,
  hasCurrent,
  onSelectSize,
  onOpenChange,
}: {
  options: { sizeId: string; sizeName: string }[];
  sizeName: string | null | undefined;
  hasCurrent: boolean;
  onSelectSize: (newSize: string) => void | Promise<void>;
  onOpenChange?: (open: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, []);

  useEffect(() => {
    onOpenChange?.(open);
    return () => onOpenChange?.(false);
  }, [open, onOpenChange]);

  const display = hasCurrent && sizeName ? sizeName : "Choose";

  return (
    <div ref={rootRef} className="relative inline-flex w-fit max-w-full">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="listbox"
        onClick={() => setOpen((v) => !v)}
        className="relative inline-flex h-9 w-fit max-w-full items-center gap-1.5 rounded-full border border-[var(--color-line)] bg-[#F9F5F0] pl-3 pr-8 text-left transition-shadow hover:shadow-sm"
      >
        <span className="shrink-0 text-[10px] font-medium uppercase leading-none tracking-[0.14em] text-[var(--color-muted)]">
          Size:
        </span>
        <span className="shrink-0 text-sm font-bold leading-none tracking-tight text-[var(--color-ink)]">
          {display}
        </span>
        <ChevronDown
          className={cn(
            "pointer-events-none absolute right-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--color-muted)] transition-transform duration-200",
            open && "rotate-180"
          )}
          strokeWidth={2}
          aria-hidden
        />
      </button>

      {open && (
        <ul
          role="listbox"
          className="absolute left-0 top-[calc(100%+6px)] z-50 w-full max-h-52 overflow-y-auto rounded-xl border border-[var(--color-line)] bg-[#F9F5F0] py-1.5 shadow-[0_12px_40px_rgba(26,24,20,0.12)] [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden"
        >
          {!hasCurrent && (
            <li className="px-4 py-2 text-xs uppercase tracking-[0.12em] text-[var(--color-muted)]">
              Choose size
            </li>
          )}
          {options.map((o) => {
            const selected = sizeName === o.sizeName;
            return (
              <li key={o.sizeId} role="option" aria-selected={selected}>
                <button
                  type="button"
                  className={cn(
                    "flex w-full items-center px-4 py-3 text-left text-base font-semibold tracking-wide text-[var(--color-ink)] transition-colors",
                    selected
                      ? "bg-[var(--color-accent-gold)]/15 text-[var(--color-accent-gold)]"
                      : "hover:bg-white/70"
                  )}
                  onClick={() => {
                    setOpen(false);
                    void onSelectSize(o.sizeName);
                  }}
                >
                  {o.sizeName}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

function OrderSummaryPanel({
  cartLines,
  cartSubtotal,
  cartCount,
  onCheckout,
}: {
  cartLines: ReturnType<typeof useStorefront>["cartLines"];
  cartSubtotal: number;
  cartCount: number;
  onCheckout: () => void;
}) {
  return (
    <div className="overflow-hidden rounded-[30px] bg-[radial-gradient(circle_at_top,rgba(201,166,70,0.22),transparent_42%),linear-gradient(165deg,#0E3D2F_0%,#114636_48%,#082E24_100%)] text-[#F6F3EA] shadow-[0_2px_8px_rgba(15,61,46,0.06)]">
      <div className="p-6 sm:p-7">
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-full border border-white/10 bg-white/10 text-[#E7CF82] shadow-[0_12px_24px_rgba(0,0,0,0.18)]">
            <BagIcon className="h-5 w-5" />
          </div>
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[#E7CF82]">
              Price Details
            </p>
            <p className="mt-1 text-sm text-[#F6F3EA]/78">
              {cartCount} {cartCount === 1 ? "item" : "items"} in your bag
            </p>
          </div>
        </div>

        <div className="mt-7 rounded-[24px] border border-white/10 bg-white/5 p-5 backdrop-blur-sm">
          {cartLines.map(({ id, product, qty, sizeName }) => (
            <div key={id} className="mb-3 flex items-center justify-between gap-4">
              <span className="text-sm text-[#F6F3EA]/82">
                {product.name}
                {sizeName && sizeName.toLowerCase() !== "free size" ? ` / ${sizeName}` : ""} × {qty}
              </span>
              <span className="text-sm font-medium text-white">
                {formatInrFromPaise(
                  qty * (product.pricePaise ?? Math.round(product.price * 100))
                )}
              </span>
            </div>
          ))}
          <div className="my-4 border-t border-white/10" />
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-[#F6F3EA]/82">Subtotal</span>
            <span className="text-sm font-medium text-white">{INR.format(cartSubtotal)}</span>
          </div>
          <div className="mt-3 flex items-center justify-between gap-4">
            <span className="text-sm text-[#F6F3EA]/65">Shipping</span>
            <span className="text-sm font-medium text-[#F6F3EA]/70">Calculated at checkout</span>
          </div>
          <div className="my-5 border-t border-white/10" />
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm font-semibold text-[#F6F3EA]">Total Amount</span>
            <span className="text-3xl font-semibold text-white">{INR.format(cartSubtotal)}</span>
          </div>
        </div>

        <div className="mt-7">
          <button
            onClick={onCheckout}
            className="group inline-flex h-14 w-full items-center justify-center gap-2 rounded-full bg-[#C9A646] px-5 text-[11px] font-semibold uppercase tracking-[0.24em] text-white shadow-[0_16px_28px_rgba(201,166,70,0.28)] transition duration-300 hover:-translate-y-1 hover:bg-[#B89435]"
          >
            Checkout
            <ArrowRightIcon className="h-4 w-4 transition duration-300 group-hover:translate-x-1" />
          </button>
        </div>
      </div>
    </div>
  );
}

export default function BagPage() {
  const router = useRouter();
  const { status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const {
    cartLines,
    decCart,
    incCart,
    removeCart,
    toggleWish,
    wishlist,
    addToCart,
  } = useStorefront();

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [openSizeForId, setOpenSizeForId] = useState<string | null>(null);

  // Keep selection in sync when cart lines change (e.g. item removed)
  useEffect(() => {
    const ids = new Set(cartLines.map((l) => l.id));
    setSelected((prev) => new Set([...prev].filter((id) => ids.has(id))));
  }, [cartLines]);

  // Select all by default on first load
  useEffect(() => {
    if (cartLines.length > 0 && selected.size === 0) {
      setSelected(new Set(cartLines.map((l) => l.id)));
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cartLines.length]);

  const allSelected = cartLines.length > 0 && selected.size === cartLines.length;
  const toggleAll = () =>
    setSelected(allSelected ? new Set() : new Set(cartLines.map((l) => l.id)));
  const toggleOne = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const selectedLines = cartLines.filter((l) => selected.has(l.id));
  const selectedCount = selectedLines.reduce((s, l) => s + l.qty, 0);
  const selectedSubtotal = selectedLines.reduce(
    (s, l) =>
      s + l.qty * ((l.product.pricePaise ?? Math.round(l.product.price * 100)) / 100),
    0
  );

  const reduceMotion = useReducedMotion();

  const [catalogSizes, setCatalogSizes] = useState<CatalogSize[]>([]);
  useEffect(() => {
    const sid = getGuestSessionId();
    if (!sid) return;
    void fetch("/api/sizes", { headers: { "x-session-id": sid } })
      .then((r) => r.json())
      .then((d: { sizes?: CatalogSize[] }) => setCatalogSizes(d.sizes ?? []))
      .catch(() => setCatalogSizes([]));
  }, []);

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

        {/* ── Empty state ── */}
        {cartLines.length === 0 ? (
          <div className="mt-24 flex flex-col items-center gap-6 text-center">
            <div className="flex h-24 w-24 items-center justify-center rounded-full border border-[var(--color-line)] bg-white">
              <ShoppingBag size={36} strokeWidth={1.25} className="text-[var(--color-accent-gold)]" />
            </div>
            <div>
              <p className="font-display text-2xl font-medium text-[var(--color-ink)]">
                Your bag is empty
              </p>
              <p className="mt-2 text-sm text-[var(--color-muted)]">
                Looks like you haven&apos;t added anything yet.
              </p>
            </div>
            <Link
              href="/"
              className="mt-2 rounded-full bg-[var(--color-accent-gold)] px-8 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90"
            >
              Continue Shopping
            </Link>
          </div>
        ) : (
          <div className="space-y-6 pb-28 md:pb-6 lg:flex lg:h-full lg:min-h-0 lg:flex-col lg:pb-0">
            <div className="flex flex-row items-stretch justify-between gap-2 border-b border-[#0F3D2E]/8 pb-8 sm:gap-3">
              <div className="flex min-w-0">
                <p className="inline-flex h-full min-h-[44px] items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px] sm:tracking-[0.28em]">
                  Shopping Bag
                </p>
              </div>
              <div className="inline-flex shrink-0 items-center gap-2 rounded-full border border-[#0F3D2E]/10 bg-[#FFFDF8] px-2.5 py-2 sm:gap-2.5 sm:px-3">
                <button
                  type="button"
                  onClick={toggleAll}
                  aria-label="Select all items"
                  className={cn(
                    "inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors",
                    allSelected
                      ? "bg-[#0F3D2E] text-[#F6F3EA] shadow-[0_6px_12px_rgba(15,61,46,0.18)]"
                      : "bg-white text-[var(--color-muted)] border border-[var(--color-line)]"
                  )}
                >
                  <CheckIcon className="h-3 w-3" />
                </button>
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">
                    Selected Items
                  </p>
                  <p className="text-[11px] font-semibold text-[#162019]">
                    {selected.size} / {cartLines.length} selected
                  </p>
                </div>
              </div>
            </div>
            <div className="mt-8 grid min-h-0 flex-1 gap-7 xl:grid-cols-[minmax(0,1fr)_390px]">

            {/* ── Left: product list ── */}
            <div className="min-h-0 h-full flex flex-col">

              {/* Product cards */}
              <div className="min-h-0 flex-1 space-y-6 overflow-y-auto pr-2 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
                <AnimatePresence initial={false} mode="popLayout">
                  {cartLines.map(({ id, product, qty, sizeName }) => (
                    <motion.div
                      key={id}
                      layout
                      initial={reduceMotion ? false : { opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={
                        reduceMotion
                          ? { opacity: 0, transition: { duration: 0.15 } }
                          : {
                              opacity: 0,
                              x: -48,
                              scale: 0.96,
                              filter: "blur(3px)",
                              transition: {
                                duration: 0.4,
                                ease: [0.22, 1, 0.36, 1],
                              },
                            }
                      }
                      transition={{
                        layout: { duration: 0.35, ease: [0.22, 1, 0.36, 1] },
                        opacity: { duration: 0.25 },
                        y: { duration: 0.3 },
                      }}
                      className={cn(
                        "group relative overflow-hidden rounded-[22px] border border-[#0F3D2E]/10 bg-[linear-gradient(180deg,#FFFDF9_0%,#FAF6EF_100%)] shadow-[0_2px_8px_rgba(15,61,46,0.06)] will-change-transform transition duration-500",
                        openSizeForId === id ? "z-30" : "z-0"
                      )}
                    >
                      <div className="grid grid-cols-[minmax(0,100px)_minmax(0,1fr)] gap-0 sm:grid-cols-[minmax(0,118px)_minmax(0,1fr)] lg:grid-cols-[188px_minmax(0,1fr)]">

                      {/* Product image column */}
                      <div className="relative flex flex-col justify-center border-r border-[#0F3D2E]/6 bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.10),transparent_40%),linear-gradient(180deg,#F6F0E7_0%,#EFE6D9_100%)] p-2 sm:p-3 lg:p-4">
                        <Link href={`/product/${product.id}`} className="mx-auto block h-[132px] w-[88px] overflow-hidden rounded-[14px] border border-white/55 bg-white shadow-[0_12px_28px_rgba(15,61,46,0.10)] sm:h-[156px] sm:w-[102px] sm:rounded-[16px] lg:h-[182px] lg:w-[128px] lg:rounded-[18px]">
                          {product.image ? (
                            // eslint-disable-next-line @next/next/no-img-element
                            <img
                              src={product.image}
                              alt={product.name}
                              className="h-full w-full object-cover transition duration-700 group-hover:scale-[1.035]"
                            />
                          ) : (
                            <div className="h-full w-full" />
                          )}
                        </Link>
                      </div>

                      {/* Product details */}
                      <div className="min-w-0 p-3 sm:p-4 lg:p-5">
                        <div>
                          {/* Name + price row */}
                          <div className="flex items-start justify-between gap-3">
                            <div className="flex-1">
                              <Link href={`/product/${product.id}`}>
                                <p className="font-display text-[1.4rem] leading-tight text-[#0F3D2E] sm:text-[1.6rem]">
                                  {product.name}
                                </p>
                              </Link>
                              <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.3em] text-[#8B816D]">
                                {product.collection}
                              </p>
                            </div>
                            {/* Price — top right */}
                            <div className="shrink-0 text-right">
                              <p className="font-display text-[1.4rem] leading-none text-[#0F3D2E] sm:text-[1.6rem]">
                                {formatInrFromPaise(
                                  qty * (product.pricePaise ?? Math.round(product.price * 100))
                                )}
                              </p>
                              <p className="mt-1 text-[10px] tracking-[0.04em] text-[#807769]">
                                MRP incl. of all taxes
                              </p>
                            </div>
                          </div>
                          <div className="my-3">
                            <GoldDivider />
                          </div>
                          <div className="flex flex-wrap items-center gap-2">
                            {(() => {
                              const options = buildSizeOptions(product, catalogSizes);
                              if (options.length === 0) return null;
                              const hasCurrent =
                                !!sizeName && options.some((o) => o.sizeName === sizeName);
                              return (
                                <BagSizeDropdown
                                  options={options}
                                  sizeName={sizeName}
                                  hasCurrent={hasCurrent}
                                  onOpenChange={(isOpen) =>
                                    setOpenSizeForId((prev) =>
                                      isOpen ? id : prev === id ? null : prev
                                    )
                                  }
                                  onSelectSize={async (newSize) => {
                                    if (!newSize || newSize === sizeName) return;
                                    await removeCart(id);
                                    await addToCart(product, qty, newSize);
                                  }}
                                />
                              );
                            })()}
                            <div className="inline-flex h-9 items-center rounded-full border border-[#0F3D2E]/10 bg-[#FFFDF8] px-1.5 shadow-[0_8px_16px_rgba(15,61,46,0.05)]">
                              <button
                                onClick={() => decCart(id)}
                                aria-label="Decrease"
                                className="flex h-7 w-7 items-center justify-center rounded-full text-[#0F3D2E] transition duration-300 hover:scale-105 hover:bg-[#0F3D2E]/6"
                              >−</button>
                              <span className="min-w-[2rem] text-center text-sm font-semibold text-[#162019]">
                                {qty}
                              </span>
                              <button
                                onClick={() => incCart(id)}
                                aria-label="Increase"
                                className="flex h-7 w-7 items-center justify-center rounded-full text-[#0F3D2E] transition duration-300 hover:scale-105 hover:bg-[#0F3D2E]/6"
                              >+</button>
                            </div>
                          </div>
                        </div>

                        {/* Action row */}
                        <div className="mt-4 flex flex-wrap items-center gap-5 border-t border-[#0F3D2E]/8 pt-4">
                          <button
                            onClick={() => removeCart(id)}
                            className="group inline-flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-[#9B6A62] transition duration-300 hover:text-[#7B3F38]"
                          >
                            <TrashIcon className="h-4 w-4 transition duration-300 group-hover:-translate-y-0.5" />
                            Remove
                          </button>
                          <button
                            type="button"
                            onClick={async () => {
                              if (!wishlist[product.id]) {
                                toggleWish(product);
                              }
                              await removeCart(id);
                            }}
                            className="group inline-flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-[#7C7367] transition duration-300 hover:text-[#0F3D2E]"
                          >
                            <HeartIcon className="h-4 w-4 transition duration-300 group-hover:-translate-y-0.5" />
                            Move to Wishlist
                          </button>
                        </div>
                      </div>

                    </div>
                      <button
                        type="button"
                        onClick={() => toggleOne(id)}
                        aria-label={`Select ${product.name}`}
                        className={cn(
                          "absolute bottom-4 right-4 flex h-7 w-7 items-center justify-center rounded-full border-2 shadow-[0_4px_12px_rgba(15,61,46,0.20)] transition-colors duration-200",
                          selected.has(id)
                            ? "border-[#0F3D2E] bg-[#0F3D2E] text-white"
                            : "border-[#0F3D2E]/20 bg-white/60 text-transparent backdrop-blur-sm"
                        )}
                      >
                        <CheckIcon className="h-3.5 w-3.5" />
                      </button>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
            </div>

            {/* ── Right: order summary ── */}
            <div className="xl:min-h-0 xl:overflow-y-auto [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
              <div>
                <OrderSummaryPanel
                  cartLines={selectedLines}
                  cartSubtotal={selectedSubtotal}
                  cartCount={selectedCount}
                  onCheckout={handleCheckout}
                />
              </div>
            </div>

            </div>
          </div>
        )}
      </div>

      {/* ── Mobile sticky footer ── */}
      {cartLines.length > 0 && (
        <div className="fixed bottom-0 left-0 right-0 z-40 border-t border-[var(--color-line)] bg-white/95 px-4 py-3 backdrop-blur-sm md:hidden">
          <div className="flex items-center justify-between gap-4">
            <div>
              <p className="text-[10px] uppercase tracking-[0.18em] text-[var(--color-muted)]">Total</p>
              <p className="font-sans text-base font-bold text-[var(--color-ink)]">
                {INR.format(selectedSubtotal)}
              </p>
            </div>
            <button
              onClick={handleCheckout}
              disabled={selectedCount === 0}
              className="flex-1 rounded-full bg-[var(--color-accent-gold)] py-3 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90 disabled:opacity-40"
            >
              Checkout ({selectedCount})
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
