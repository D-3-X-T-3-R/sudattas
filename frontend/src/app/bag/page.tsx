"use client";

import Link from "next/link";
import { ChevronDown, ShoppingBag } from "lucide-react";
import { useStorefront } from "@/context/storefront-context";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { INR } from "@/lib/constants";
import { SiteHeader } from "@/components/site-header";
import { getGuestSessionId } from "@/lib/session";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useState, useEffect, useRef } from "react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { signIn, useSession } from "next-auth/react";

type CatalogSize = { sizeId: string; sizeName: string };

function FingerprintIcon(props: React.SVGProps<SVGSVGElement>) {
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
      <path d="M12 3a7 7 0 0 0-7 7v1" />
      <path d="M19 11v-1a7 7 0 0 0-14 0v3" />
      <path d="M8 14v-4a4 4 0 1 1 8 0v1" />
      <path d="M12 17v-3" />
      <path d="M17 14c0 4-2 7-5 7s-5-3-5-7" />
      <path d="M21 14c0 5.5-3.6 10-9 10" />
    </svg>
  );
}

function GoogleIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <path
        fill="#EA4335"
        d="M12 10.2v3.9h5.4c-.2 1.3-1.7 3.9-5.4 3.9-3.2 0-5.9-2.7-5.9-6s2.7-6 5.9-6c1.8 0 3 .8 3.7 1.4l2.5-2.4C16.7 3.6 14.6 2.7 12 2.7 6.9 2.7 2.8 6.8 2.8 12S6.9 21.3 12 21.3c6.9 0 9.1-4.8 9.1-7.3 0-.5 0-.9-.1-1.3H12Z"
      />
      <path
        fill="#34A853"
        d="M2.8 12c0 1.9.7 3.7 1.9 5.1l3.1-2.4c-.4-.8-.7-1.7-.7-2.7s.2-1.8.7-2.7L4.7 6.9A9.2 9.2 0 0 0 2.8 12Z"
      />
      <path
        fill="#FBBC05"
        d="M12 21.3c2.5 0 4.6-.8 6.2-2.3l-3-2.4c-.8.5-1.8.8-3.1.8-2.4 0-4.4-1.6-5.1-3.8l-3.2 2.4c1.6 3.1 4.7 5.3 8.2 5.3Z"
      />
      <path
        fill="#4285F4"
        d="M18.2 19c1.8-1.7 2.9-4.1 2.9-7 0-.5 0-.9-.1-1.3H12v3.9h5.4c-.2 1.1-.8 2.1-1.7 2.8l2.5 1.6Z"
      />
    </svg>
  );
}

function getBackendBaseUrl(): string {
  const url = process.env.NEXT_PUBLIC_GRAPHQL_URL || "http://localhost:8080/v2";
  return url.replace(/\/v2\/?$/, "");
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
        className="relative inline-flex min-h-[46px] w-fit max-w-full items-center gap-2 rounded-full border border-[var(--color-line)] bg-[#F9F5F0] py-2.5 pl-4 pr-10 text-left transition-shadow hover:shadow-sm"
      >
        <span className="shrink-0 text-xs font-medium uppercase leading-none tracking-[0.14em] text-[var(--color-muted)]">
          Size:
        </span>
        <span className="shrink-0 text-lg font-bold leading-none tracking-tight text-[var(--color-ink)]">
          {display}
        </span>
        <ChevronDown
          className={cn(
            "pointer-events-none absolute right-2.5 top-1/2 h-[18px] w-[18px] -translate-y-1/2 text-[var(--color-muted)] transition-transform duration-200",
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
  paymentLoading,
  paymentMessage,
  runTest,
  onCheckout,
}: {
  cartLines: ReturnType<typeof useStorefront>["cartLines"];
  cartSubtotal: number;
  cartCount: number;
  paymentLoading: boolean;
  paymentMessage: string | null;
  runTest: () => void;
  onCheckout: () => void;
}) {
  return (
    <div className="rounded-xl border border-[var(--color-line)] bg-white p-8">
      <h2 className="font-display text-lg font-medium uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
        Price Details
      </h2>
      <p className="mt-1 text-sm text-[var(--color-muted)]">({cartCount} {cartCount === 1 ? "item" : "items"})</p>

      <ul className="mt-6 space-y-3.5 border-b border-[var(--color-line)] pb-6">
        {cartLines.map(({ id, product, qty, sizeName }) => (
          <li key={id} className="flex items-start justify-between gap-3 text-base">
            <span className="text-[var(--color-muted)]">
              {product.name}
              {sizeName && sizeName.toLowerCase() !== "free size" && (
                <span className="ml-1 text-xs opacity-60">/ {sizeName}</span>
              )}
              <span className="ml-1 opacity-70">× {qty}</span>
            </span>
            <span className="shrink-0 font-sans font-semibold text-[var(--color-ink)]">
              {INR.format(qty * product.price)}
            </span>
          </li>
        ))}
      </ul>

      <div className="mt-5 flex items-center justify-between border-b border-[var(--color-line)] pb-5">
        <span className="text-base font-medium text-[var(--color-ink)]">Total Amount</span>
        <span className="font-sans text-xl font-bold text-[var(--color-ink)]">
          {INR.format(cartSubtotal)}
        </span>
      </div>

      <p className="mt-4 text-sm text-[var(--color-muted)]">
        Shipping and taxes calculated at checkout.
      </p>

      <button
        onClick={onCheckout}
        className="mt-6 w-full rounded-full bg-[var(--color-accent-gold)] py-4 text-sm font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90 active:scale-[0.98]"
      >
        Checkout
      </button>
      <button
        onClick={runTest}
        disabled={paymentLoading}
        className="mt-3 w-full rounded-full border border-[var(--color-line)] py-4 text-sm font-semibold uppercase tracking-[0.2em] text-[var(--color-accent-gold)] transition-colors hover:border-[var(--color-accent-gold)] disabled:opacity-50"
      >
        {paymentLoading ? "Opening Razorpay…" : "Test Razorpay (₹100)"}
      </button>
      {paymentMessage && (
        <p className="mt-3 text-xs text-[var(--color-muted)]">{paymentMessage}</p>
      )}
    </div>
  );
}

export default function BagPage() {
  const { status } = useSession();
  const {
    cartLines,
    cartSubtotal,
    decCart,
    incCart,
    cartCount,
    removeCart,
    toggleWish,
    wishlist,
    addToCart,
  } = useStorefront();
  const { paymentLoading, paymentMessage, runTest } = useRazorpayTest();

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [openSizeForId, setOpenSizeForId] = useState<string | null>(null);
  const [loginOpen, setLoginOpen] = useState(false);
  const [identifier, setIdentifier] = useState("");
  const [otp, setOtp] = useState("");
  const [otpSent, setOtpSent] = useState(false);
  const [otpBusy, setOtpBusy] = useState(false);
  const [loginNote, setLoginNote] = useState<string | null>(null);
  const [passkeySupported, setPasskeySupported] = useState(false);

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
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });

  const selectedLines = cartLines.filter((l) => selected.has(l.id));
  const selectedCount = selectedLines.reduce((s, l) => s + l.qty, 0);
  const selectedSubtotal = selectedLines.reduce((s, l) => s + l.qty * l.product.price, 0);

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
      setLoginOpen(true);
      return;
    }
    alert("Checkout flow not wired yet");
  };

  useEffect(() => {
    if (typeof window === "undefined") return;
    const ok =
      "PublicKeyCredential" in window &&
      typeof window.PublicKeyCredential !== "undefined";
    setPasskeySupported(ok);
  }, []);

  const normalizedDigits = identifier.replace(/\D/g, "");
  const identifierTrimmed = identifier.trim();
  const looksLikeEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(identifierTrimmed);
  const looksLikePhone = normalizedDigits.length === 10;

  const adaptiveActionLabel = looksLikeEmail
    ? "Send Login Link"
    : looksLikePhone
      ? otpSent
        ? "Resend Code on WhatsApp"
        : "Get Code on WhatsApp"
      : "Continue";

  const handleAdaptiveContinue = () => {
    if (looksLikeEmail) {
      setLoginNote("Magic link login will be enabled next. Use Google or WhatsApp OTP for now.");
      return;
    }
    if (!looksLikePhone) {
      setLoginNote("Enter a valid email or 10-digit phone number.");
      return;
    }

    const digits = normalizedDigits;
    if (digits.length !== 10) {
      setLoginNote("Please enter a valid 10-digit phone number.");
      return;
    }
    setOtpBusy(true);
    setLoginNote(null);
    void fetch(`${getBackendBaseUrl()}/auth/phone-otp/request`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ phone: digits, channel: "whatsapp" }),
    })
      .then(async (res) => {
        const json = (await res.json().catch(() => ({}))) as { ok?: boolean; error?: string };
        if (!res.ok || !json.ok) {
          throw new Error(json.error ?? "OTP_SEND_FAILED");
        }
        setOtpSent(true);
        setLoginNote("Code sent on WhatsApp.");
      })
      .catch(() => {
        setLoginNote("Could not send code. Please try again.");
      })
      .finally(() => setOtpBusy(false));
  };

  const handleOtpLogin = () => {
    const digits = normalizedDigits;
    if (digits.length !== 10) {
      setLoginNote("Please enter a valid 10-digit phone number.");
      return;
    }
    if (!/^\d{4,8}$/.test(otp.trim())) {
      setLoginNote("Please enter a valid OTP.");
      return;
    }
    setOtpBusy(true);
    setLoginNote(null);
    void signIn("phone-otp", {
      phone: digits,
      otp: otp.trim(),
      redirect: false,
      callbackUrl: "/bag",
    })
      .then((res) => {
        if (res?.ok) {
          setLoginOpen(false);
          setOtp("");
          setOtpSent(false);
          setLoginNote(null);
          return;
        }
        setLoginNote("Invalid OTP. Please try again.");
      })
      .catch(() => setLoginNote("Login failed. Please try again."))
      .finally(() => setOtpBusy(false));
  };

  return (
    <div className="min-h-screen w-full min-w-0 bg-[var(--background)] text-[var(--foreground)]">
      <SiteHeader />

      <div className="mx-auto w-full max-w-[2000px] px-4 pt-8 pb-32 md:pb-12">

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
                Looks like you haven't added anything yet.
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
          <div className="flex flex-col gap-6 md:flex-row md:items-start">

            {/* ── Left: product list ── */}
            <div className="flex-1 min-w-0">

              {/* Items selected header */}
              <div className="mb-4 flex items-center gap-4 rounded-xl border border-[var(--color-line)] bg-white px-5 py-4 sm:px-6 sm:py-4">
                <input
                  type="checkbox"
                  checked={allSelected}
                  onChange={toggleAll}
                  className="h-5 w-5 cursor-pointer accent-[var(--color-accent-gold)]"
                  aria-label="Select all items"
                />
                <span className="text-sm font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]">
                  {selected.size}/{cartLines.length} Items Selected
                </span>
                {selectedSubtotal > 0 && (
                  <span className="text-sm font-semibold text-[var(--color-accent-gold)]">
                    ({INR.format(selectedSubtotal)})
                  </span>
                )}
              </div>

              {/* Product cards */}
              <div className="rounded-xl border border-[var(--color-line)] bg-white md:max-h-[calc(100vh-200px)] md:overflow-y-auto md:[scrollbar-width:none] md:[-ms-overflow-style:none] md:[&::-webkit-scrollbar]:hidden">
                <AnimatePresence initial={false} mode="popLayout">
                  {cartLines.map(({ id, product, qty, sizeName }, idx) => (
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
                        "relative will-change-transform",
                        openSizeForId === id ? "z-30" : "z-0"
                      )}
                    >
                      {idx > 0 && <div className="mx-4 h-px bg-[var(--color-line)]" />}

                      <div className="flex gap-0 p-5 sm:gap-3 sm:p-6">

                      {/* Checkbox */}
                      <div className="flex items-start pt-1 pr-3">
                        <input
                          type="checkbox"
                          checked={selected.has(id)}
                          onChange={() => toggleOne(id)}
                          className="h-4 w-4 cursor-pointer accent-[var(--color-accent-gold)]"
                          aria-label={`Select ${product.name}`}
                        />
                      </div>

                      {/* Image */}
                      <Link href={`/product/${product.id}`} className="w-[148px] shrink-0 sm:w-[168px]">
                        <div className="aspect-[3/4] overflow-hidden rounded-md bg-[var(--color-line)]">
                          {product.image ? (
                            // eslint-disable-next-line @next/next/no-img-element
                            <img
                              src={product.image}
                              alt={product.name}
                              className="h-full w-full object-cover transition-transform duration-500 hover:scale-105"
                            />
                          ) : (
                            <div className="h-full w-full" />
                          )}
                        </div>
                      </Link>

                      {/* Middle content */}
                      <div className="flex flex-1 flex-col justify-between pl-4 sm:pl-5">
                        <div>
                          {/* Name + price row */}
                          <div className="flex items-start justify-between gap-4">
                            <div className="flex-1">
                              <Link href={`/product/${product.id}`}>
                                <p className="font-display text-lg font-medium leading-snug text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-gold)] sm:text-xl">
                                  {product.name}
                                </p>
                              </Link>
                              <p className="mt-0.5 text-xs uppercase tracking-[0.16em] text-[var(--color-muted)]">
                                {product.collection}
                              </p>
                            </div>
                            {/* Price — top right */}
                            <div className="shrink-0 text-right">
                              <p className="font-sans text-lg font-bold text-[var(--color-ink)] sm:text-xl">
                                {INR.format(qty * product.price)}
                              </p>
                              <p className="mt-1 text-xs leading-snug text-[var(--color-muted)]">
                                MRP incl. of all taxes
                              </p>
                            </div>
                          </div>

                          {/* Size + Qty selectors — matching pill style */}
                          <div className="mt-3 flex flex-wrap items-center gap-2">
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
                            <div className="inline-flex min-h-[46px] items-center gap-1 rounded-full border border-[var(--color-line)] bg-[#F9F5F0] px-1.5 py-1 text-base">
                              <button
                                onClick={() => decCart(id)}
                                aria-label="Decrease"
                                className="flex h-10 w-10 items-center justify-center rounded-full text-lg leading-none text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-gold)]"
                              >−</button>
                              <span className="min-w-[2rem] text-center font-sans text-base font-semibold text-[var(--color-ink)]">
                                {qty}
                              </span>
                              <button
                                onClick={() => incCart(id)}
                                aria-label="Increase"
                                className="flex h-10 w-10 items-center justify-center rounded-full text-lg leading-none text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-gold)]"
                              >+</button>
                            </div>
                          </div>
                        </div>

                        {/* Action row */}
                        <div className="mt-4 flex items-center gap-1 border-t border-[var(--color-line)] pt-3">
                          <button
                            onClick={() => removeCart(id)}
                            className="rounded px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)] transition-colors hover:text-red-400"
                          >
                            Remove
                          </button>
                          <span className="text-[var(--color-line)]">|</span>
                          <button
                            type="button"
                            onClick={async () => {
                              if (!wishlist[product.id]) {
                                toggleWish(product);
                              }
                              await removeCart(id);
                            }}
                            className="rounded px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-muted)] transition-colors hover:text-[var(--color-accent-gold)]"
                          >
                            Move to Wishlist
                          </button>
                        </div>
                      </div>

                    </div>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
            </div>

            {/* ── Right: sticky order summary ── */}
            <div className="hidden w-[340px] shrink-0 md:block lg:w-[400px]">
              <div className="sticky top-6">
                <OrderSummaryPanel
                  cartLines={selectedLines}
                  cartSubtotal={selectedSubtotal}
                  cartCount={selectedCount}
                  paymentLoading={paymentLoading}
                  paymentMessage={paymentMessage}
                  runTest={runTest}
                  onCheckout={handleCheckout}
                />
              </div>
            </div>

          </div>
        )}
      </div>

      <Dialog open={loginOpen} onOpenChange={setLoginOpen}>
        <DialogContent
          className="max-w-xl overflow-hidden rounded-[28px] bg-[#F6F3EA] shadow-[0_20px_70px_rgba(0,0,0,0.22)]"
          contentClassName="space-y-0 p-0"
          showClose={false}
        >
          <div className="p-6 sm:p-8">
            <div className="space-y-4">
              {passkeySupported && (
                <button
                  type="button"
                  onClick={() => setLoginNote("Passkey login is coming next. Use Google or WhatsApp OTP for now.")}
                  className="flex h-14 w-full items-center justify-center gap-3 rounded-full border border-[#0F3D2E]/12 bg-white px-5 text-sm font-medium tracking-[0.12em] text-[#0F3D2E] shadow-[0_8px_24px_rgba(15,61,46,0.06)] transition hover:-translate-y-0.5 hover:border-[#C9A646]/40 hover:shadow-[0_12px_30px_rgba(15,61,46,0.10)]"
                >
                  <FingerprintIcon className="h-4 w-4 text-[#C9A646]" />
                  <span>Continue with Face ID / Fingerprint</span>
                </button>
              )}

              <button
                type="button"
                onClick={() => signIn("google", { callbackUrl: "/bag" })}
                className="flex h-14 w-full items-center justify-center gap-3 rounded-full bg-[#C9A646] px-5 text-sm font-semibold tracking-[0.12em] text-white shadow-[0_14px_30px_rgba(201,166,70,0.28)] transition hover:-translate-y-0.5 hover:bg-[#B89435]"
              >
                <span className="flex h-6 w-6 items-center justify-center rounded-full bg-white">
                  <GoogleIcon className="h-4 w-4" />
                </span>
                <span>Continue with Google</span>
              </button>
            </div>

            <div className="my-7 flex items-center gap-4">
              <div className="h-px flex-1 bg-[#0F3D2E]/10" />
              <span className="text-[11px] font-medium uppercase tracking-[0.28em] text-[#9B927E]">
                Or
              </span>
              <div className="h-px flex-1 bg-[#0F3D2E]/10" />
            </div>

            <div className="rounded-2xl border border-[#0F3D2E]/10 bg-white p-4">
              <label
                htmlFor="adaptive-identifier"
                className="mb-2.5 block text-[11px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]"
              >
                Email or phone number
              </label>
              <Input
                id="adaptive-identifier"
                inputMode="email"
                autoComplete="username"
                placeholder="Enter your email or phone number"
                value={identifier}
                onChange={(e) => {
                  setIdentifier(e.target.value);
                  if (loginNote) setLoginNote(null);
                  if (otpSent) {
                    setOtpSent(false);
                    setOtp("");
                  }
                }}
                className="h-14 w-full rounded-2xl border border-[#0F3D2E]/10 bg-white px-4 text-sm text-[#1B1B1B] outline-none transition placeholder:text-[#B8AE9A] focus:border-[#C9A646] focus:shadow-[0_0_0_4px_rgba(201,166,70,0.14)]"
              />

              <button
                type="button"
                onClick={handleAdaptiveContinue}
                disabled={otpBusy}
                className="mt-4 h-14 w-full rounded-full bg-[#0F3D2E] px-5 text-sm font-semibold tracking-[0.14em] text-[#F6F3EA] shadow-[0_18px_32px_rgba(15,61,46,0.18)] transition hover:-translate-y-0.5 hover:bg-[#0C3126] disabled:opacity-60"
              >
                {otpBusy ? "Processing..." : adaptiveActionLabel}
              </button>

              {looksLikePhone && otpSent && (
                <>
                  <Input
                    inputMode="numeric"
                    autoComplete="one-time-code"
                    placeholder="Enter OTP"
                    value={otp}
                    onChange={(e) => {
                      setOtp(e.target.value.replace(/\D/g, "").slice(0, 8));
                      if (loginNote) setLoginNote(null);
                    }}
                    className="mt-3 h-12 w-full rounded-2xl border border-[#0F3D2E]/10 bg-white px-4 text-sm text-[#1B1B1B] outline-none transition placeholder:text-[#B8AE9A] focus:border-[#C9A646] focus:shadow-[0_0_0_4px_rgba(201,166,70,0.14)]"
                  />
                  <button
                    type="button"
                    onClick={handleOtpLogin}
                    disabled={otpBusy}
                    className="mt-3 h-12 w-full rounded-full bg-[#C9A646] px-5 text-sm font-semibold tracking-[0.12em] text-white shadow-[0_14px_30px_rgba(201,166,70,0.28)] transition hover:-translate-y-0.5 hover:bg-[#B89435] disabled:opacity-60"
                  >
                    {otpBusy ? "Verifying..." : "Verify Code & Sign in"}
                  </button>
                </>
              )}
            </div>

            {loginNote && (
              <p className="mt-4 text-center text-xs leading-6 text-[#7B7568]">
                {loginNote}
              </p>
            )}
          </div>
        </DialogContent>
      </Dialog>

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
