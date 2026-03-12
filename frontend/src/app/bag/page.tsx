"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useStorefront } from "@/context/storefront-context";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { INR } from "@/lib/constants";

export default function BagPage() {
  const {
    cartLines,
    cartSubtotal,
    decCart,
    incCart,
    cartCount,
  } = useStorefront();
  const { paymentLoading, paymentMessage, runTest } = useRazorpayTest();

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <div className="mx-auto max-w-2xl px-4 py-8">
        <Button
          variant="ghost"
          size="sm"
          className="mb-6 -ml-2 text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          asChild
        >
          <Link href="/" className="flex items-center gap-2">
            <ArrowLeft className="h-4 w-4" />
            Back to shop
          </Link>
        </Button>

        <h1 className="text-xl font-display font-semibold tracking-tight text-[var(--color-ink)]">
          Bag {cartCount > 0 ? `(${cartCount})` : ""}
        </h1>

        {cartLines.length === 0 ? (
          <div className="mt-8 rounded-2xl border border-[var(--color-line)] bg-white p-8 text-center">
            <p className="text-sm text-[var(--color-muted)]">Your bag is empty.</p>
            <Button
              variant="outline"
              className="mt-4 w-full rounded-full border-[var(--color-line)]"
              asChild
            >
              <Link href="/">Continue shopping</Link>
            </Button>
          </div>
        ) : (
          <div className="mt-6 space-y-6">
            {cartLines.map(({ id, product, qty, sizeName }) => (
              <div
                key={id}
                className="border-b border-[var(--color-line)] pb-6"
              >
                <div className="text-[11px] tracking-[0.18em] text-[var(--color-muted)]">
                  {product.collection.toUpperCase()}
                </div>
                <div className="mt-1 text-sm font-semibold text-[var(--color-ink)]">
                  {product.name}
                </div>
                <div className="mt-1 text-xs text-[var(--color-muted)]">
                  {product.fabric} • {product.occasion}
                  {sizeName ? ` • Size ${sizeName}` : null}
                </div>
                <div className="mt-4 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => decCart(id)}
                      aria-label="Decrease"
                      className="border-[var(--color-line)] bg-white hover:bg-white/80"
                    >
                      −
                    </Button>
                    <div className="min-w-10 text-center font-sans text-sm font-semibold">
                      {qty}
                    </div>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => incCart(id)}
                      aria-label="Increase"
                      className="border-[var(--color-line)] bg-white hover:bg-white/80"
                    >
                      +
                    </Button>
                  </div>
                  <div className="font-sans text-sm font-semibold">
                    {INR.format(qty * product.price)}
                  </div>
                </div>
              </div>
            ))}

            <div className="rounded-2xl border border-[var(--color-line)] bg-white p-6">
              <div className="flex items-center justify-between text-sm">
                <span className="text-[var(--color-muted)]">Subtotal</span>
                <span className="font-sans font-semibold">
                  {INR.format(cartSubtotal)}
                </span>
              </div>
              <div className="mt-2 text-xs text-[var(--color-muted)]">
                Shipping and taxes calculated at checkout.
              </div>
              <Button
                className="mt-4 w-full rounded-full bg-[var(--color-accent-gold)] hover:bg-[var(--color-accent-gold)]/90 text-[var(--color-ink)]"
                onClick={() => alert("Checkout flow not wired yet")}
              >
                Checkout
              </Button>
              <Button
                variant="outline"
                className="mt-3 w-full rounded-full border-[var(--color-line)] text-[var(--color-accent-brown)]"
                onClick={runTest}
                disabled={paymentLoading}
              >
                {paymentLoading ? "Opening Razorpay…" : "Test Razorpay (₹100)"}
              </Button>
              {paymentMessage && (
                <p className="mt-3 text-xs text-[var(--color-muted)]">
                  {paymentMessage}
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
