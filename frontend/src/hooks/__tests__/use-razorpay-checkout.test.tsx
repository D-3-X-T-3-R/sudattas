import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  announce: vi.fn(),
  fetchApiEnvelope: vi.fn(),
}));

vi.mock("@/components/ui/live-announcer", () => ({
  useLiveAnnouncer: () => ({ announce: mocks.announce }),
}));

vi.mock("@/lib/api-envelope", () => ({
  fetchApiEnvelope: (...args: unknown[]) => mocks.fetchApiEnvelope(...args),
}));

import { useRazorpayCheckout } from "@/hooks/use-razorpay-checkout";

describe("useRazorpayCheckout async reconciliation", () => {
  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  beforeEach(() => {
    vi.useFakeTimers();
    mocks.announce.mockReset();
    mocks.fetchApiEnvelope.mockReset();
  });

  it("eventually reflects reconciled payment state after verify returns intermediate status", async () => {
    const razorpay = vi.fn((options: Record<string, unknown>) => ({
      open: () => {
        const handler = options.handler as ((response: {
          razorpay_payment_id: string;
          razorpay_order_id: string;
          razorpay_signature: string;
        }) => Promise<void>) | undefined;
        if (handler) {
          void handler({
            razorpay_payment_id: "pay_1",
            razorpay_order_id: "order_1",
            razorpay_signature: "sig_1",
          });
        }
      },
      on: () => {},
    }));
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).Razorpay = razorpay;

    mocks.fetchApiEnvelope
      .mockResolvedValueOnce({
        order: { orderId: "o1" },
        paymentIntent: {
          razorpayOrderId: "order_1",
          razorpayKeyId: "key_1",
          orderId: "o1",
          amountPaise: "10000",
          currency: "INR",
        },
        idempotency: { placeOrderKey: "place_1", verifyKey: "verify_1" },
      })
      .mockResolvedValueOnce({
        verified: true,
        paymentState: "verified",
        orderStatusId: "1",
        orderUiState: "pending",
      })
      .mockResolvedValueOnce({
        paymentState: "verified",
        fulfillmentState: "pending",
        statusName: "pending",
      })
      .mockResolvedValueOnce({
        paymentState: "paid",
        fulfillmentState: "processing",
        statusName: "processing",
      });

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({ shippingAddressId: "10", selectedCartLineIds: ["c1"] });
    });

    for (let i = 0; i < 4; i += 1) {
      await act(async () => {
        await Promise.resolve();
        vi.runOnlyPendingTimers();
      });
    }

    expect(result.current.paymentMessage).toContain("Payment paid");
    expect(result.current.paymentMessage).toContain("Order state: processing");
  });

  it("blocks checkout when no cart lines are selected", async () => {
    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({ shippingAddressId: "10", selectedCartLineIds: [] });
    });

    expect(result.current.paymentMessage).toContain("Select at least one bag item");
    expect(mocks.fetchApiEnvelope).not.toHaveBeenCalled();
  });
});
