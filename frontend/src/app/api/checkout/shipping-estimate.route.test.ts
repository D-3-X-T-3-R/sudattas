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

import { POST } from "@/app/api/checkout/shipping-estimate/route";

describe("checkout shipping-estimate selected cart line contract", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("forwards selected cart line ids to GraphQL", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: {
        estimateCheckoutShipping: {
          shippingAmountPaise: "500",
          courierName: "Shiprocket",
          estimatedDeliveryDays: 3,
          itemSubtotalPaise: "2500",
          orderTotalPaise: "3000",
          quoteAvailable: true,
          note: null,
        },
      },
    });

    const req = new Request("http://localhost/api/checkout/shipping-estimate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ shippingAddressId: "10", selectedCartLineIds: ["c1", "c2"] }),
    });

    const res = await POST(req);
    expect(res.status).toBe(200);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledWith(
      "44",
      expect.stringContaining("EstimateCheckoutShipping"),
      {
        input: {
          shippingAddressId: "10",
          couponCode: null,
          selectedCartIds: ["c1", "c2"],
        },
      }
    );
  });

  it("rejects empty selectedCartLineIds", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("44");

    const req = new Request("http://localhost/api/checkout/shipping-estimate", {
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
