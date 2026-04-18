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

import { POST } from "@/app/api/checkout/place-order/route";

describe("checkout place-order idempotency", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("forwards stable idempotency key for order placement", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: { placeOrder: [{ orderId: "100", totalAmountPaise: "5000", totalAmountFormatted: "Rs 50.00", statusId: "1" }] },
      })
      .mockResolvedValueOnce({
        data: {
          getPaymentIntent: [
            {
              intentId: "pi1",
              razorpayOrderId: "r1",
              razorpayKeyId: "k1",
              orderId: "100",
              amountPaise: "5000",
              currency: "INR",
              status: "pending",
            },
          ],
        },
      });

    const req = new Request("http://localhost/api/checkout/place-order", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ shippingAddressId: "10", selectedCartLineIds: ["c1", "c2"], idempotencyKey: "place-key-1" }),
    });

    const res = await POST(req);
    expect(res.status).toBe(200);

    expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(
      1,
      "44",
      expect.stringContaining("PlaceOrder"),
      {
        order: {
          shippingAddressId: "10",
          couponCode: null,
          selectedCartIds: ["c1", "c2"],
        },
      },
      { "Idempotency-Key": "place-key-1" }
    );
    expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(
      2,
      "44",
      expect.stringContaining("GetPaymentIntent"),
      { input: { orderId: "100" } }
    );
  });

  it("rejects empty selectedCartLineIds", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");

    const req = new Request("http://localhost/api/checkout/place-order", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ shippingAddressId: "10", selectedCartLineIds: [] }),
    });

    const res = await POST(req);
    const body = await res.json();

    expect(res.status).toBe(400);
    expect(body.message).toContain("selectedCartLineIds");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });
});
