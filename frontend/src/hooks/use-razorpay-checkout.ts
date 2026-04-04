"use client";

import { useState, useCallback } from "react";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { ApiEnvelopeError } from "@/lib/api-envelope";
import {
  paymentIntentSchema,
  verifyRazorpayPayloadSchema,
} from "@/lib/schemas";
import { toRouteFailureUi } from "@/lib/route-state";

declare global {
  interface Window {
    Razorpay?: new (options: Record<string, unknown>) => {
      open: () => void;
      on: (event: string, handler: () => void) => void;
    };
  }
}

type OrderReconcilePayload = {
  statusName: string;
  paymentState: string;
  fulfillmentState: string;
};

const FINAL_PAYMENT_STATES = new Set([
  "paid",
  "failed",
  "refunded",
  "needs_review",
]);

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
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

// eslint-disable-next-line max-lines-per-function
export function useRazorpayCheckout() {
  const [paymentMessage, setPaymentMessage] = useState<string | null>(null);
  const [paymentLoading, setPaymentLoading] = useState(false);
  const { announce } = useLiveAnnouncer();

  const setPaymentMessageWithAnnounce = useCallback(
    (message: string | null, politeness: "polite" | "assertive" = "polite") => {
      setPaymentMessage(message);
      if (message) announce(message, politeness);
    },
    [announce]
  );

  const accountErrorMessage = useCallback((error: unknown): string => {
    if (error instanceof ApiEnvelopeError && error.message?.trim()) {
      return error.message;
    }
    return toRouteFailureUi("account", error).message;
  }, []);

  const pollOrderReconciliation = useCallback(async (orderId: string) => {
    const maxAttempts = 6;
    const delayMs = 4000;

    for (let i = 0; i < maxAttempts; i += 1) {
      await sleep(delayMs);
      const detail = await fetchApiEnvelope<{
        paymentState: string;
        fulfillmentState: string;
        statusName: string;
      }>(`/api/account/orders/${encodeURIComponent(orderId)}`, {
        cache: "no-store",
      });
      if (FINAL_PAYMENT_STATES.has(detail.paymentState)) {
        return detail as OrderReconcilePayload;
      }
    }
    return null;
  }, []);

  const runCheckout = useCallback(async (input?: {
    shippingAddressId?: string;
    onSuccess?: (payload: { orderId: string; paymentState: string; orderUiState: string }) => void;
    onFailure?: (payload: { orderId: string; reason?: string }) => void;
  }) => {
    const shippingAddressId = input?.shippingAddressId?.trim();
    if (!shippingAddressId) {
      setPaymentMessageWithAnnounce("Select a shipping address first.", "assertive");
      return;
    }

    setPaymentMessageWithAnnounce(null);
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
        setPaymentMessageWithAnnounce(
          "No Razorpay key/order returned. Please retry in a moment."
        );
        return;
      }
      const parsed = paymentIntentSchema.safeParse(raw);
      if (!parsed.success) {
        setPaymentMessageWithAnnounce("Invalid payment intent response.", "assertive");
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
              const successPayload = {
                orderId,
                paymentState: verifyResult.paymentState,
                orderUiState: verifyResult.orderUiState,
              };
              if (verifyResult.paymentState === "failed") {
                if (input?.onFailure) {
                  input.onFailure({
                    orderId,
                    reason: "Payment verification marked this payment as failed.",
                  });
                  return;
                }
                setPaymentMessageWithAnnounce(
                  "Payment failed. Please try again with a different payment method.",
                  "assertive"
                );
                return;
              }
              if (input?.onSuccess) {
                input.onSuccess(successPayload);
                return;
              }
              if (FINAL_PAYMENT_STATES.has(verifyResult.paymentState)) {
                setPaymentMessageWithAnnounce(
                  `Payment ${verifyResult.paymentState}. Order state: ${verifyResult.orderUiState}.`
                );
              } else {
                setPaymentMessageWithAnnounce(
                  `Payment verification received. Current state: ${verifyResult.paymentState}; awaiting final backend/webhook reconciliation.`
                );
                try {
                  const reconciled = await pollOrderReconciliation(orderId);
                  if (reconciled) {
                    setPaymentMessageWithAnnounce(
                      `Payment ${reconciled.paymentState}. Order state: ${reconciled.statusName}.`
                    );
                  } else {
                    setPaymentMessageWithAnnounce(
                      "Verification received. Final payment state is still processing; please refresh your profile orders shortly."
                    );
                  }
                } catch (pollError) {
                  setPaymentMessageWithAnnounce(
                    toRouteFailureUi("account", pollError).message,
                    "assertive"
                  );
                }
              }
            } else {
              setPaymentMessageWithAnnounce("Verify failed or invalid response.", "assertive");
            }
          } catch (e) {
            setPaymentMessageWithAnnounce(accountErrorMessage(e), "assertive");
          }
        },
      };
      const rzp = new window.Razorpay!(options);
      rzp.on("payment.failed", (failure: unknown) => {
        const reason =
          typeof failure === "object" &&
          failure !== null &&
          "error" in failure &&
          typeof (failure as { error?: { description?: unknown } }).error?.description ===
            "string"
            ? (failure as { error: { description: string } }).error.description
            : "Payment failed or was closed.";
        if (input?.onFailure) {
          input.onFailure({ orderId, reason });
          return;
        }
        setPaymentMessageWithAnnounce("Payment failed or was closed.", "assertive");
      });
      rzp.open();
    } catch (e) {
      setPaymentMessageWithAnnounce(accountErrorMessage(e), "assertive");
    } finally {
      setPaymentLoading(false);
    }
  }, [accountErrorMessage, pollOrderReconciliation, setPaymentMessageWithAnnounce]);

  const runTest = useCallback(async () => {
    setPaymentMessageWithAnnounce("Use checkout flow from /checkout/address.");
  }, [setPaymentMessageWithAnnounce]);

  return { paymentMessage, paymentLoading, runTest, runCheckout };
}
