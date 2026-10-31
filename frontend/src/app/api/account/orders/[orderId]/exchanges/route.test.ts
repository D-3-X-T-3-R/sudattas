import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn(),
}));

vi.mock("@/lib/server-session-auth", () => ({
  apiError: (message: string, status: number, errorCode: string) =>
    Response.json(
      {
        ok: false,
        data: null,
        errorCode,
        message,
        fieldErrors: null,
        retryable: status >= 500,
      },
      { status }
    ),
  requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
  callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
}));

import { POST } from "@/app/api/account/orders/[orderId]/exchanges/route";

describe("POST /api/account/orders/[orderId]/exchanges", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailId: "91", desiredVariantId: "55", reason: "Too small" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns validation errors for missing fields", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");

    const resNoDetail = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ desiredVariantId: "55", reason: "Too small" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(resNoDetail.status).toBe(400);

    const resNoVariant = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailId: "91", reason: "Too small" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(resNoVariant.status).toBe(400);

    const resNoReason = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailId: "91", desiredVariantId: "55", reason: "  " }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(resNoReason.status).toBe(400);

    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("sends order id, order detail id, desired variant and reason", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: {
        requestExchange: [
          {
            exchangeId: "3",
            orderId: "7",
            orderDetailId: "91",
            desiredVariantId: "55",
            quantity: "1",
            status: "requested",
            reason: "Too small",
          },
        ],
      },
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailId: "91", desiredVariantId: "55", reason: "Too small" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );

    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
    const [, , variables] = mocks.callGraphqlAsCustomer.mock.calls[0];
    expect(variables).toEqual({
      input: {
        orderId: "7",
        orderDetailId: "91",
        desiredVariantId: "55",
        reason: "Too small",
      },
    });

    const json = (await res.json()) as { ok: boolean; data: { exchangeId: string } };
    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data.exchangeId).toBe("3");
  });

  it("maps failed_precondition GraphQL errors to 409", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "failed_precondition: Exchange window has closed" }],
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailId: "91", desiredVariantId: "55", reason: "Too small" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(res.status).toBe(409);
  });
});
