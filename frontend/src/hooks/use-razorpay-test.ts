"use client";

import { useState, useCallback } from "react";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import {
  paymentIntentSchema,
  verifyRazorpayPayloadSchema,
} from "@/lib/schemas";

declare global {
  interface Window {
    Razorpay?: new (options: Record<string, unknown>) => {
      open: () => void;
      on: (event: string, handler: () => void) => void;
    };
  }
}

function loadRazorpayScript(): Promise<void> {
  if (typeof window !== "undefined" && window.Razorpay)
    return Promise.resolve();
  return new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = "https://checkout.razorpay.com/v1/checkout.js";
    s.onload = () => resolve();
    s.onerror = () => reject(new Error("Failed to load Razorpay"));
    document.body.appendChild(s);
  });
}

export function useRazorpayTest() {
  const [paymentMessage, setPaymentMessage] = useState<string | null>(null);
  const [paymentLoading, setPaymentLoading] = useState(false);

  const runCheckout = useCallback(async (input?: { shippingAddressId?: string }) => {
    const shippingAddressId = input?.shippingAddressId?.trim();
    if (!shippingAddressId) {
      setPaymentMessage("Select a shipping address first.");
      return;
    }

    setPaymentMessage(null);
    setPaymentLoading(true);
    try {
      const placeOrderKey = `checkout-place-${crypto.randomUUID()}`;
      const start = await fetchApiEnvelope<{
        order: {
          orderId: string;
        };
        paymentIntent: {
          intentId?: string;
          razorpayOrderId: string;
          razorpayKeyId: string;
          orderId: string;
          amountPaise: string;
          currency: string;
        };
        idempotency: { placeOrderKey: string; verifyKey: string };
      }>("/api/checkout/place-order", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ shippingAddressId, idempotencyKey: placeOrderKey }),
      });

      const raw = start?.paymentIntent;
      if (!raw?.razorpayKeyId || !raw?.razorpayOrderId) {
        setPaymentMessage(
          "No Razorpay key/order returned. Please retry in a moment."
        );
        return;
      }
      const parsed = paymentIntentSchema.safeParse(raw);
      if (!parsed.success) {
        setPaymentMessage("Invalid payment intent response.");
        return;
      }
      const intent = parsed.data;
      await loadRazorpayScript();
      const orderId = intent.orderId;
      const verifyKey = start?.idempotency?.verifyKey;
      const options = {
        key: intent.razorpayKeyId,
        amount: intent.amountPaise,
        currency: intent.currency || "INR",
        order_id: intent.razorpayOrderId,
        name: "Sudatta's",
        description: "Test payment (₹100)",
        handler: async function (response: {
          razorpay_payment_id: string;
          razorpay_order_id: string;
          razorpay_signature: string;
        }) {
          try {
            const verifyResult = await fetchApiEnvelope<{
              verified: boolean;
              paymentState: string;
              orderStatusId: string | null;
              orderUiState: string;
            }>("/api/checkout/verify-payment", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({
                orderId,
                razorpayPaymentId: response.razorpay_payment_id,
                razorpayOrderId: response.razorpay_order_id,
                razorpaySignature: response.razorpay_signature,
                idempotencyKey: verifyKey,
              }),
            });
            const verifyParsed = verifyRazorpayPayloadSchema.safeParse({
              verified: verifyResult.verified,
              paymentIntent: { status: verifyResult.paymentState },
            });
            if (verifyParsed.success && verifyResult.verified) {
              const finalPaymentStates = new Set([
                "paid",
                "failed",
                "refunded",
                "needs_review",
              ]);
              if (finalPaymentStates.has(verifyResult.paymentState)) {
                setPaymentMessage(
                  `Payment ${verifyResult.paymentState}. Order state: ${verifyResult.orderUiState}.`
                );
              } else {
                setPaymentMessage(
                  `Payment verification received. Current state: ${verifyResult.paymentState}; awaiting final backend/webhook reconciliation.`
                );
              }
            } else {
              setPaymentMessage("Verify failed or invalid response.");
            }
          } catch (e) {
            setPaymentMessage(
              "Verify failed: " + ((e as Error).message || String(e))
            );
          }
        },
      };
      const rzp = new window.Razorpay!(options);
      rzp.on("payment.failed", () => {
        setPaymentMessage("Payment failed or was closed.");
      });
      rzp.open();
    } catch (e) {
      setPaymentMessage(
        "Error: " + ((e as Error).message || String(e))
      );
    } finally {
      setPaymentLoading(false);
    }
  }, []);

  const runTest = useCallback(async () => {
    setPaymentMessage("Use checkout flow from /checkout/address.");
  }, []);

  return { paymentMessage, paymentLoading, runTest, runCheckout };
}
