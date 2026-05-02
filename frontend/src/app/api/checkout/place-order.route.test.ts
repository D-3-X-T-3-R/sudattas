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
              razorpayOrderId: "order_1",
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
          paymentMode: "prepaid",
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

  it("rejects missing shippingAddressId", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");

    const req = new Request("http://localhost/api/checkout/place-order", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ selectedCartLineIds: ["c1"] }),
    });

    const res = await POST(req);
    const body = await res.json();

    expect(res.status).toBe(400);
    expect(body.errorCode).toBe("VALIDATION_ERROR");
    expect(body.message).toContain("shippingAddressId");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns COD payload without fetching payment intent", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer.mockResolvedValueOnce({
      data: {
        placeOrder: [
          {
            orderId: "200",
            totalAmountPaise: "10000",
            totalAmountFormatted: "Rs 100.00",
            statusId: "1",
            paymentMethod: "cod",
          },
        ],
      },
    });

    const req = new Request("http://localhost/api/checkout/place-order", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        shippingAddressId: "10",
        selectedCartLineIds: ["c1"],
        paymentMode: "cod",
        idempotencyKey: "place-key-cod-1",
      }),
    });

    const res = await POST(req);
    const body = await res.json();

    expect(res.status).toBe(200);
    expect(body.ok).toBe(true);
    expect(body.data.checkoutMode).toBe("cod");
    expect(body.data.order.totalAmountPaise).toBe("10000");
    expect(body.data.paymentIntent).toBeNull();
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
  });

  it("rejects prepaid response when gateway order id is placeholder-like", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: {
          placeOrder: [
            {
              orderId: "300",
              totalAmountPaise: "5000",
              totalAmountFormatted: "Rs 50.00",
              statusId: "1",
              paymentMethod: "prepaid",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          getPaymentIntent: [
            {
              intentId: "pi1",
              razorpayOrderId: "rzp_pending_300",
              razorpayKeyId: "k_live_1",
              orderId: "300",
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
      body: JSON.stringify({
        shippingAddressId: "10",
        selectedCartLineIds: ["c1"],
        paymentMode: "prepaid",
        idempotencyKey: "place-key-prepaid-1",
      }),
    });

    const res = await POST(req);
    const body = await res.json();

    expect(res.status).toBe(502);
    expect(body.errorCode).toBe("PAYMENT_PROVIDER_ERROR");
    expect(body.message).toContain("valid Razorpay order ID");
  });

  it("chooses a valid active payment intent instead of blindly using the first row", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: {
          placeOrder: [
            {
              orderId: "301",
              totalAmountPaise: "9000",
              totalAmountFormatted: "Rs 90.00",
              statusId: "1",
              paymentMethod: "prepaid",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          getPaymentIntent: [
            {
              intentId: "30",
              razorpayOrderId: "rzp_pending_301",
              razorpayKeyId: "k_live_invalid",
              orderId: "301",
              amountPaise: "9000",
              currency: "INR",
              status: "pending",
            },
            {
              intentId: "31",
              razorpayOrderId: "order_301_real",
              razorpayKeyId: "k_live_valid",
              orderId: "301",
              amountPaise: "9000",
              currency: "INR",
              status: "pending",
            },
          ],
        },
      });

    const req = new Request("http://localhost/api/checkout/place-order", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        shippingAddressId: "10",
        selectedCartLineIds: ["c1"],
        paymentMode: "prepaid",
      }),
    });

    const res = await POST(req);
    const body = await res.json();

    expect(res.status).toBe(200);
    expect(body.data.paymentIntent.razorpayOrderId).toBe("order_301_real");
    expect(body.data.paymentIntent.razorpayKeyId).toBe("k_live_valid");
  });
});
