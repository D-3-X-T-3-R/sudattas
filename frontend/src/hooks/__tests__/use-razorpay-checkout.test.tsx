import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/* eslint-disable max-lines-per-function */

const mocks = vi.hoisted(() => ({
  announce: vi.fn(),
  fetchApiEnvelope: vi.fn(),
}));

vi.mock("@/components/ui/live-announcer", () => ({
  useLiveAnnouncer: () => ({ announce: mocks.announce }),
}));

vi.mock("@/lib/api-envelope", () => ({
  ApiEnvelopeError: class ApiEnvelopeError extends Error {},
  fetchApiEnvelope: (...args: unknown[]) => mocks.fetchApiEnvelope(...args),
}));

import { useRazorpayCheckout } from "@/hooks/use-razorpay-checkout";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

function mockRazorpayAutoSuccess() {
  const Razorpay = vi.fn((options: Record<string, unknown>) => ({
    open: () => {
      const handler = options.handler as ((payload: {
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
  (window as any).Razorpay = Razorpay;
  return Razorpay;
}

describe("useRazorpayCheckout launch blocker protections", () => {
  beforeEach(() => {
    mocks.announce.mockReset();
    mocks.fetchApiEnvelope.mockReset();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    delete (window as any).Razorpay;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("double-click checkout triggers only one place-order request", async () => {
    const placeOrder = deferred<{
      order: { orderId: string };
      paymentIntent: null;
      idempotency: { placeOrderKey: string; verifyKey?: string | null };
    }>();
    mocks.fetchApiEnvelope.mockReturnValueOnce(placeOrder.promise);

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
    });

    expect(mocks.fetchApiEnvelope).toHaveBeenCalledTimes(1);

    await act(async () => {
      placeOrder.resolve({
        order: { orderId: "" },
        paymentIntent: null,
        idempotency: { placeOrderKey: "place-1" },
      });
      await Promise.resolve();
    });
  });

  it("mobile double-tap checkout triggers only one place-order request", async () => {
    const placeOrder = deferred<{
      order: { orderId: string };
      paymentIntent: null;
      idempotency: { placeOrderKey: string; verifyKey?: string | null };
    }>();
    mocks.fetchApiEnvelope.mockReturnValueOnce(placeOrder.promise);

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
    });

    expect(mocks.fetchApiEnvelope).toHaveBeenCalledTimes(1);

    await act(async () => {
      placeOrder.resolve({
        order: { orderId: "" },
        paymentIntent: null,
        idempotency: { placeOrderKey: "place-1" },
      });
      await Promise.resolve();
    });
  });

  it("failed attempt can be retried with a new clean idempotency key", async () => {
    mocks.fetchApiEnvelope
      .mockRejectedValueOnce(new Error("Temporary gateway error"))
      .mockRejectedValueOnce(new Error("Temporary gateway error again"));

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
    });

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
      });
    });

    const firstCallBody = JSON.parse((mocks.fetchApiEnvelope.mock.calls[0][1] as { body: string }).body) as {
      idempotencyKey: string;
    };
    const secondCallBody = JSON.parse((mocks.fetchApiEnvelope.mock.calls[1][1] as { body: string }).body) as {
      idempotencyKey: string;
    };

    expect(firstCallBody.idempotencyKey).toMatch(/^checkout-place-/);
    expect(secondCallBody.idempotencyKey).toMatch(/^checkout-place-/);
    expect(secondCallBody.idempotencyKey).not.toBe(firstCallBody.idempotencyKey);
  });

  it("COD path cannot double-submit", async () => {
    const placeOrder = deferred<{
      checkoutMode: "cod";
      order: { orderId: string };
      paymentIntent: null;
      idempotency: { placeOrderKey: string; verifyKey?: string | null };
    }>();
    mocks.fetchApiEnvelope.mockReturnValueOnce(placeOrder.promise);
    const onSuccess = vi.fn();

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        paymentMode: "cod",
        onSuccess,
      });
      void result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        paymentMode: "cod",
        onSuccess,
      });
    });

    expect(mocks.fetchApiEnvelope).toHaveBeenCalledTimes(1);

    await act(async () => {
      placeOrder.resolve({
        checkoutMode: "cod",
        order: { orderId: "o-cod-1" },
        paymentIntent: null,
        idempotency: { placeOrderKey: "place-cod-1", verifyKey: null },
      });
      await Promise.resolve();
    });

    expect(onSuccess).toHaveBeenCalledTimes(1);
  });

  it("paid verify response routes to success callback", async () => {
    mockRazorpayAutoSuccess();
    const onSuccess = vi.fn();
    const onFailure = vi.fn();

    mocks.fetchApiEnvelope
      .mockResolvedValueOnce({
        order: { orderId: "o-paid-1" },
        paymentIntent: {
          razorpayOrderId: "order_1",
          razorpayKeyId: "key_1",
          orderId: "o-paid-1",
          amountPaise: "10000",
          currency: "INR",
        },
        idempotency: { placeOrderKey: "place-paid-1", verifyKey: "verify-paid-1" },
      })
      .mockResolvedValueOnce({
        verified: true,
        paymentState: "paid",
        orderStatusId: "2",
        orderUiState: "processing",
      });

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        onSuccess,
        onFailure,
      });
      await Promise.resolve();
    });

    expect(onSuccess).toHaveBeenCalledTimes(1);
    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({ orderId: "o-paid-1", checkoutState: "paid" })
    );
    expect(onFailure).not.toHaveBeenCalled();
  });

  it("failed verify response routes to failed callback", async () => {
    mockRazorpayAutoSuccess();
    const onSuccess = vi.fn();
    const onFailure = vi.fn();

    mocks.fetchApiEnvelope
      .mockResolvedValueOnce({
        order: { orderId: "o-failed-1" },
        paymentIntent: {
          razorpayOrderId: "order_1",
          razorpayKeyId: "key_1",
          orderId: "o-failed-1",
          amountPaise: "10000",
          currency: "INR",
        },
        idempotency: { placeOrderKey: "place-failed-1", verifyKey: "verify-failed-1" },
      })
      .mockResolvedValueOnce({
        verified: true,
        paymentState: "failed",
        orderStatusId: "9",
        orderUiState: "failed",
      });

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        onSuccess,
        onFailure,
      });
      await Promise.resolve();
    });

    expect(onFailure).toHaveBeenCalledTimes(1);
    expect(onSuccess).not.toHaveBeenCalled();
  });

  it("pending verify response does not route to success callback", async () => {
    mockRazorpayAutoSuccess();
    const onSuccess = vi.fn();
    const onPending = vi.fn();

    mocks.fetchApiEnvelope
      .mockResolvedValueOnce({
        order: { orderId: "o-pending-1" },
        paymentIntent: {
          razorpayOrderId: "order_1",
          razorpayKeyId: "key_1",
          orderId: "o-pending-1",
          amountPaise: "10000",
          currency: "INR",
        },
        idempotency: { placeOrderKey: "place-pending-1", verifyKey: "verify-pending-1" },
      })
      .mockResolvedValueOnce({
        verified: true,
        paymentState: "pending",
        orderStatusId: "1",
        orderUiState: "pending",
      });

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        onSuccess,
        onPending,
      });
      await Promise.resolve();
    });

    expect(onPending).toHaveBeenCalledTimes(1);
    expect(onSuccess).not.toHaveBeenCalled();
  });

  it("needs_review verify response does not route to success callback", async () => {
    mockRazorpayAutoSuccess();
    const onSuccess = vi.fn();
    const onNeedsReview = vi.fn();

    mocks.fetchApiEnvelope
      .mockResolvedValueOnce({
        order: { orderId: "o-review-1" },
        paymentIntent: {
          razorpayOrderId: "order_1",
          razorpayKeyId: "key_1",
          orderId: "o-review-1",
          amountPaise: "10000",
          currency: "INR",
        },
        idempotency: { placeOrderKey: "place-review-1", verifyKey: "verify-review-1" },
      })
      .mockResolvedValueOnce({
        verified: true,
        paymentState: "paid",
        orderStatusId: "99",
        orderUiState: "needs_review",
      });

    const { result } = renderHook(() => useRazorpayCheckout());

    await act(async () => {
      await result.current.runCheckout({
        shippingAddressId: "addr-1",
        selectedCartLineIds: ["cart-1"],
        onSuccess,
        onNeedsReview,
      });
      await Promise.resolve();
    });

    expect(onNeedsReview).toHaveBeenCalledTimes(1);
    expect(onSuccess).not.toHaveBeenCalled();
  });
});
