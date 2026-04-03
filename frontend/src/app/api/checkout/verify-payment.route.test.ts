import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn(),
}));

vi.mock("@/lib/server-session-auth", () => ({
  apiError: (message: string, status: number, errorCode: string) =>
    Response.json({ ok: false, data: null, errorCode, message, fieldErrors: null, retryable: status >= 500 }, { status }),
  requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
  callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
}));

import { POST } from "@/app/api/checkout/verify-payment/route";

describe("checkout verify-payment contract", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("forwards verify idempotency key and returns normalized envelope", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: {
          verifyRazorpayPayment: [
            { verified: true, paymentIntent: { intentId: "pi1", status: "captured", razorpayPaymentId: "pay_1" } },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          searchOrder: [{ orderId: "o1", statusId: "2" }],
        },
      });

    const req = new Request("http://localhost/api/checkout/verify-payment", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        orderId: "o1",
        razorpayPaymentId: "pay_1",
        razorpayOrderId: "order_1",
        razorpaySignature: "sig_1",
        idempotencyKey: "verify-key-1",
      }),
    });

    const res = await POST(req);
    const json = (await res.json()) as {
      ok: boolean;
      errorCode: string | null;
      retryable: boolean;
      data: { verified: boolean; paymentState: string; orderUiState: string; verifyKey: string };
    };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.errorCode).toBeNull();
    expect(json.retryable).toBe(false);
    expect(json.data.verified).toBe(true);
    expect(json.data.paymentState).toBe("paid");
    expect(json.data.orderUiState).toBe("processing");
    expect(json.data.verifyKey).toBe("verify-key-1");

    expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(
      1,
      "44",
      expect.stringContaining("VerifyRazorpayPayment"),
      expect.any(Object),
      { "Idempotency-Key": "verify-key-1" }
    );
  });
});

