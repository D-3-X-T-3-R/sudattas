"use client";

import { useState, useCallback, useRef } from "react";
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
      /** Razorpay passes e.g. `{ error: { description, code, ... } }` for `payment.failed`. */
      on: (event: string, handler: (...args: unknown[]) => void) => void;
    };
  }
}

type CheckoutOutcomePayload = {
  orderId: string;
  paymentState: string;
  orderUiState: string;
  checkoutState: "paid" | "failed" | "pending" | "needs_review" | "cod";
};

type CheckoutInput = {
  shippingAddressId?: string;
  selectedCartLineIds?: string[];
  paymentMode?: "prepaid" | "cod";
  onSuccess?: (payload: CheckoutOutcomePayload) => void;
  onPending?: (payload: CheckoutOutcomePayload) => void;
  onNeedsReview?: (payload: CheckoutOutcomePayload) => void;
  onFailure?: (payload: { orderId: string; reason?: string }) => void;
};

function isValidRazorpayOrderId(value: string): boolean {
  return value.trim().startsWith("order_");
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

function normalizeVerifyPaymentState(rawStatus?: string | null): "paid" | "failed" | "pending" | "needs_review" {
  const status = (rawStatus ?? "").trim().toLowerCase();
  if (status.includes("needs_review")) return "needs_review";
  if (status.includes("failed")) return "failed";
  if (
    status.includes("paid") ||
    status.includes("captured") ||
    status.includes("confirmed") ||
    status.includes("processed")
  ) {
    return "paid";
  }
  return "pending";
}

function deriveCheckoutState(
  paymentState?: string | null,
  orderUiState?: string | null
): "paid" | "failed" | "pending" | "needs_review" {
  const normalizedOrderUiState = (orderUiState ?? "").trim().toLowerCase();
  if (normalizedOrderUiState.includes("needs_review")) return "needs_review";
  if (normalizedOrderUiState.includes("failed")) return "failed";
  if (normalizedOrderUiState.includes("pending")) return "pending";
  return normalizeVerifyPaymentState(paymentState);
}

// eslint-disable-next-line max-lines-per-function
export function useRazorpayCheckout() {
  const [paymentMessage, setPaymentMessage] = useState<string | null>(null);
  const [paymentLoading, setPaymentLoading] = useState(false);
  const { announce } = useLiveAnnouncer();
  const checkoutInFlightRef = useRef(false);
  const checkoutAttemptRef = useRef<{
    placeOrderKey: string;
    verifyKey?: string | null;
  } | null>(null);

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

  const resetAttempt = useCallback(() => {
    checkoutAttemptRef.current = null;
    checkoutInFlightRef.current = false;
    setPaymentLoading(false);
  }, []);

  // eslint-disable-next-line max-lines-per-function
  const runCheckout = useCallback(async (input?: CheckoutInput) => {
    if (checkoutInFlightRef.current) {
      setPaymentMessageWithAnnounce("Checkout is already in progress. Please wait.", "polite");
      return;
    }

    const shippingAddressId = input?.shippingAddressId?.trim();
    const selectedCartLineIds = (input?.selectedCartLineIds ?? []).map((id) => id.trim()).filter(Boolean);
    if (!shippingAddressId) {
      setPaymentMessageWithAnnounce("Select a shipping address first.", "assertive");
      return;
    }
    if (selectedCartLineIds.length === 0) {
      setPaymentMessageWithAnnounce("Select at least one bag item to checkout.", "assertive");
      return;
    }

    const paymentMode: "prepaid" | "cod" =
      input?.paymentMode === "cod" ? "cod" : "prepaid";

    setPaymentMessageWithAnnounce(null);
    checkoutInFlightRef.current = true;
    setPaymentLoading(true);
    setPaymentMessageWithAnnounce("Processing...", "polite");

    if (!checkoutAttemptRef.current) {
      checkoutAttemptRef.current = {
        placeOrderKey: `checkout-place-${crypto.randomUUID()}`,
        verifyKey: null,
      };
    }
    const activeAttempt = checkoutAttemptRef.current;

    try {
      const start = await fetchApiEnvelope<{
        checkoutMode?: "prepaid" | "cod";
        order: {
          orderId: string;
          paymentMethod?: string | null;
        };
        paymentIntent: {
          intentId?: string;
          razorpayOrderId: string;
          razorpayKeyId: string;
          orderId: string;
          amountPaise: string;
          currency: string;
        } | null;
        idempotency: { placeOrderKey: string; verifyKey?: string | null };
      }>("/api/checkout/place-order", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          shippingAddressId,
          selectedCartLineIds,
          idempotencyKey: activeAttempt.placeOrderKey,
          paymentMode,
        }),
      });
      if (start?.idempotency?.verifyKey) {
        activeAttempt.verifyKey = start.idempotency.verifyKey;
      }

      const orderId = start?.order?.orderId;
      if (!orderId) {
        setPaymentMessageWithAnnounce("Order creation failed. Please retry.", "assertive");
        resetAttempt();
        return;
      }

      if ((start?.checkoutMode ?? "").toLowerCase() === "cod") {
        const successPayload: CheckoutOutcomePayload = {
          orderId,
          paymentState: "cod",
          orderUiState: "processing",
          checkoutState: "cod",
        };
        resetAttempt();
        if (input?.onSuccess) {
          input.onSuccess(successPayload);
          return;
        }
        setPaymentMessageWithAnnounce("Order placed with Cash on Delivery.");
        return;
      }

      const raw = start?.paymentIntent;
      if (!raw?.razorpayKeyId || !raw?.razorpayOrderId) {
        setPaymentMessageWithAnnounce(
          "No Razorpay key/order returned. Please retry in a moment."
        );
        resetAttempt();
        return;
      }
      const parsed = paymentIntentSchema.safeParse(raw);
      if (!parsed.success) {
        setPaymentMessageWithAnnounce("Invalid payment intent response.", "assertive");
        resetAttempt();
        return;
      }
      const intent = parsed.data;
      if (!isValidRazorpayOrderId(intent.razorpayOrderId)) {
        setPaymentMessageWithAnnounce(
          "Invalid Razorpay order ID. Please retry checkout.",
          "assertive"
        );
        resetAttempt();
        return;
      }

      await loadRazorpayScript();
      const verifyKey = activeAttempt.verifyKey;
      let finalized = false;
      const finalizeOnce = (fn: () => void) => {
        if (finalized) return;
        finalized = true;
        fn();
        resetAttempt();
      };

      const options = {
        key: intent.razorpayKeyId,
        amount: intent.amountPaise,
        currency: intent.currency || "INR",
        order_id: intent.razorpayOrderId,
        name: "Sudatta's",
        modal: {
          ondismiss: () => {
            finalizeOnce(() => {
              const reason = "Payment window was closed before completion.";
              if (input?.onFailure) {
                input.onFailure({ orderId, reason });
                return;
              }
              setPaymentMessageWithAnnounce("Payment was not completed. You can retry safely.", "assertive");
            });
          },
        },
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

            const checkoutState = deriveCheckoutState(
              verifyResult.paymentState,
              verifyResult.orderUiState
            );
            if (verifyParsed.success && verifyResult.verified) {
              const basePayload = {
                orderId,
                paymentState: checkoutState,
                orderUiState: verifyResult.orderUiState,
              };

              if (checkoutState === "failed") {
                finalizeOnce(() => {
                  if (input?.onFailure) {
                    input.onFailure({
                      orderId,
                      reason: "Payment was not completed. You can retry safely.",
                    });
                    return;
                  }
                  setPaymentMessageWithAnnounce("Payment was not completed. You can retry safely.", "assertive");
                });
                return;
              }

              if (checkoutState === "needs_review") {
                finalizeOnce(() => {
                  const payload: CheckoutOutcomePayload = {
                    ...basePayload,
                    checkoutState: "needs_review",
                  };
                  if (input?.onNeedsReview) {
                    input.onNeedsReview(payload);
                    return;
                  }
                  setPaymentMessageWithAnnounce(
                    "We received your payment update, but it needs manual verification. We'll contact you if action is needed.",
                    "polite"
                  );
                });
                return;
              }

              if (checkoutState === "pending") {
                finalizeOnce(() => {
                  const payload: CheckoutOutcomePayload = {
                    ...basePayload,
                    checkoutState: "pending",
                  };
                  if (input?.onPending) {
                    input.onPending(payload);
                    return;
                  }
                  setPaymentMessageWithAnnounce(
                    "We're confirming your payment. Please don't place another order yet.",
                    "polite"
                  );
                });
                return;
              }

              finalizeOnce(() => {
                const payload: CheckoutOutcomePayload = {
                  ...basePayload,
                  checkoutState: "paid",
                };
                if (input?.onSuccess) {
                  input.onSuccess(payload);
                  return;
                }
                setPaymentMessageWithAnnounce("Payment verified and order confirmed.");
              });
              return;
            }

            finalizeOnce(() => {
              if (input?.onFailure) {
                input.onFailure({
                  orderId,
                  reason: "Payment was not completed. You can retry safely.",
                });
                return;
              }
              setPaymentMessageWithAnnounce("Payment was not completed. You can retry safely.", "assertive");
            });
          } catch (e) {
            const message = accountErrorMessage(e);
            finalizeOnce(() => {
              if (input?.onFailure) {
                input.onFailure({ orderId, reason: message });
                return;
              }
              setPaymentMessageWithAnnounce(message, "assertive");
            });
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
            : "Payment was not completed. You can retry safely.";
        finalizeOnce(() => {
          if (input?.onFailure) {
            input.onFailure({ orderId, reason });
            return;
          }
          setPaymentMessageWithAnnounce("Payment was not completed. You can retry safely.", "assertive");
        });
      });

      setPaymentMessageWithAnnounce("Opening payment...", "polite");
      rzp.open();
    } catch (e) {
      setPaymentMessageWithAnnounce(accountErrorMessage(e), "assertive");
      resetAttempt();
    }
  }, [accountErrorMessage, resetAttempt, setPaymentMessageWithAnnounce]);

  const runTest = useCallback(async () => {
    setPaymentMessageWithAnnounce("Use checkout flow from /bag.");
  }, [setPaymentMessageWithAnnounce]);

  return { paymentMessage, paymentLoading, runTest, runCheckout };
}
