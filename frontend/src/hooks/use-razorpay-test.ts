"use client";

import { useState, useCallback } from "react";
import { gql } from "@/lib/graphqlClient";
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

  const runTest = useCallback(async () => {
    setPaymentMessage(null);
    setPaymentLoading(true);
    try {
      const data = await gql<{
        createPaymentIntent?: Array<{
          intentId?: string;
          razorpayOrderId: string;
          razorpayKeyId: string;
          orderId: string;
          amountPaise: string;
          currency: string;
        }>;
      }>(
        `mutation CreatePaymentIntent {
          createPaymentIntent(input: { orderId: "1", userId: "1", amountPaise: "10000", currency: "INR" }) {
            intentId
            razorpayOrderId
            razorpayKeyId
            orderId
            amountPaise
            currency
          }
        }`
      );
      const raw = data?.createPaymentIntent?.[0];
      if (!raw?.razorpayKeyId || !raw?.razorpayOrderId) {
        setPaymentMessage(
          "No Razorpay key/order returned. Check backend RAZORPAY_KEY_ID and order 1 exists."
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
      const orderId = intent.orderId || "1";
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
            const esc = (s: string) => JSON.stringify(String(s ?? ""));
            const verifyData = await gql<{
              verifyRazorpayPayment?: Array<{
                verified: boolean;
                paymentIntent?: { status: string };
              }>;
            }>(
              `mutation VerifyRazorpay {
                verifyRazorpayPayment(input: {
                  orderId: ${esc(orderId)},
                  razorpayPaymentId: ${esc(response.razorpay_payment_id)},
                  razorpayOrderId: ${esc(response.razorpay_order_id)},
                  razorpaySignature: ${esc(response.razorpay_signature)}
                }) { verified paymentIntent { status } }
              }`
            );
            const verifyRaw = verifyData?.verifyRazorpayPayment?.[0];
            const verifyParsed = verifyRazorpayPayloadSchema.safeParse(verifyRaw);
            if (verifyParsed.success && verifyParsed.data.verified) {
              setPaymentMessage("Payment verified successfully.");
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

  return { paymentMessage, paymentLoading, runTest };
}
