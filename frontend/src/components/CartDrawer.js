import React from "react";
import Drawer from "./Drawer";
import CheckoutButton from "./CheckoutButton";

export default function CartDrawer({
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
  theme,
  INR,
  authEnabled,
}) {
  return (
    <Drawer
      open={open}
      title={`BAG (${cartLines.reduce((s, l) => s + l.qty, 0)})`}
      onClose={onClose}
      side="right"
      theme={theme}
    >
      {cartLines.length === 0 ? (
        <div className="rounded-2xl bg-white p-6 text-sm text-[#374151]">Your bag is empty.</div>
      ) : (
        <div className="space-y-5">
          {cartLines.map(({ product, qty }) => (
            <div key={product.id} className="border-b pb-5" style={{ borderColor: theme.line }}>
              <div className="text-[11px] tracking-[0.18em] text-[#6B7280]">
                {product.collection.toUpperCase()}
              </div>
              <div className="mt-1 text-sm font-semibold text-[#111]">{product.name}</div>
              <div className="mt-1 text-xs text-[#6B7280]">
                {product.fabric} • {product.occasion}
              </div>
              <div className="mt-4 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => onDecCart(product.id)}
                    className="grid h-10 w-10 place-items-center rounded-full border bg-white hover:bg-white/80"
                    style={{ borderColor: theme.line }}
                    aria-label="Decrease"
                  >
                    −
                  </button>
                  <div className="min-w-10 text-center text-sm font-semibold">{qty}</div>
                  <button
                    onClick={() => onIncCart(product.id)}
                    className="grid h-10 w-10 place-items-center rounded-full border bg-white hover:bg-white/80"
                    style={{ borderColor: theme.line }}
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
            <div className="mt-2 text-xs text-[#6B7280]">
              Shipping and taxes calculated at checkout.
            </div>
            {authEnabled ? (
              <CheckoutButton onCheckout={onCheckout} theme={theme} />
            ) : (
              <button
                type="button"
                onClick={onCheckout}
                className="mt-4 w-full rounded-full px-5 py-3 text-sm font-semibold text-white"
                style={{ background: theme.ink }}
              >
                Checkout
              </button>
            )}
            <button
              type="button"
              onClick={onTestRazorpay}
              disabled={paymentLoading}
              className="mt-3 w-full rounded-full border px-5 py-3 text-sm font-semibold disabled:opacity-50"
              style={{ borderColor: theme.line, color: theme.accentBrown }}
            >
              {paymentLoading ? "Opening Razorpay…" : "Test Razorpay (₹100)"}
            </button>
            {paymentMessage && (
              <p className="mt-3 text-xs" style={{ color: theme.muted }}>
                {paymentMessage}
              </p>
            )}
          </div>
        </div>
      )}
    </Drawer>
  );
}

