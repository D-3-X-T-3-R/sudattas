"use client";

import { Sheet } from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { CartLine } from "@/lib/schemas";

export interface CartDrawerProps {
  open: boolean;
  onClose: () => void;
  cartLines: CartLine[];
  cartSubtotal: number;
  onDecCart: (productId: string) => void;
  onIncCart: (productId: string) => void;
  paymentLoading: boolean;
  paymentMessage: string | null;
  onTestRazorpay: () => void;
  onCheckout: () => void;
  authEnabled?: boolean;
  authCheckoutButton?: React.ReactNode;
}

export function CartDrawer({
  open,
  onClose,
  cartLines,
  cartSubtotal,
  onDecCart,
  onIncCart,
  paymentLoading,
  paymentMessage,
  onTestRazorpay,
  onCheckout,
  authEnabled,
  authCheckoutButton,
}: CartDrawerProps) {
  const count = cartLines.reduce((s, l) => s + l.qty, 0);

  return (
    <Sheet
      open={open}
      onClose={onClose}
      title={`BAG (${count})`}
      side="right"
    >
      {cartLines.length === 0 ? (
        <div className="rounded-2xl bg-white p-6 text-sm text-[var(--color-muted)]">
          Your bag is empty.
        </div>
      ) : (
        <div className="space-y-5">
          {cartLines.map(({ product, qty }) => (
            <div
              key={product.id}
              className="border-b border-[var(--color-line)] pb-5"
            >
              <div className="text-[11px] tracking-[0.18em] text-[var(--color-muted)]">
                {product.collection.toUpperCase()}
              </div>
              <div className="mt-1 text-sm font-semibold text-[var(--color-ink)]">
                {product.name}
              </div>
              <div className="mt-1 text-xs text-[var(--color-muted)]">
                {product.fabric} • {product.occasion}
              </div>
              <div className="mt-4 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => onDecCart(product.id)}
                    aria-label="Decrease"
                    className="border-[var(--color-line)] bg-white hover:bg-white/80"
                  >
                    −
                  </Button>
                  <div className="min-w-10 text-center text-sm font-semibold">
                    {qty}
                  </div>
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={() => onIncCart(product.id)}
                    aria-label="Increase"
                    className="border-[var(--color-line)] bg-white hover:bg-white/80"
                  >
                    +
                  </Button>
                </div>
                <div className="text-sm font-semibold">
                  {INR.format(qty * product.price)}
                </div>
              </div>
            </div>
          ))}

          <div className="rounded-2xl bg-white p-5">
            <div className="flex items-center justify-between text-sm">
              <span className="text-[var(--color-muted)]">Subtotal</span>
              <span className="font-semibold">{INR.format(cartSubtotal)}</span>
            </div>
            <div className="mt-2 text-xs text-[var(--color-muted)]">
              Shipping and taxes calculated at checkout.
            </div>
            {authEnabled && authCheckoutButton ? (
              authCheckoutButton
            ) : (
              <Button
                className="mt-4 w-full rounded-full bg-[var(--color-ink)] hover:bg-[var(--color-ink)]/90"
                onClick={onCheckout}
              >
                Checkout
              </Button>
            )}
            <Button
              variant="outline"
              className="mt-3 w-full rounded-full border-[var(--color-line)] text-[var(--color-accent-brown)]"
              onClick={onTestRazorpay}
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
    </Sheet>
  );
}
