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

import { POST } from "@/app/api/account/orders/[orderId]/returns/route";

describe("POST /api/account/orders/[orderId]/returns", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/returns", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"], reason: "Too big" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns validation errors for missing line ids and reason", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");

    const resNoReason = await POST(
      new Request("http://localhost/api/account/orders/7/returns", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"], reason: "  " }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(resNoReason.status).toBe(400);

    const resNoLines = await POST(
      new Request("http://localhost/api/account/orders/7/returns", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: [], reason: "Too big" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(resNoLines.status).toBe(400);
  });

  it("sends selected line ids and reason for partial return", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: {
        requestReturn: [
          {
            returnId: "14",
            orderId: "7",
            status: "requested",
            reason: "Color mismatch",
            items: [
              {
                orderDetailId: "91",
                quantity: "1",
                status: "requested",
                refundAmountMinor: "150000",
              },
            ],
          },
        ],
      },
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/returns", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          orderDetailIds: ["91", "92"],
          reason: "Color mismatch",
        }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );

    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
    const [, , variables] = mocks.callGraphqlAsCustomer.mock.calls[0];
    expect(variables).toEqual({
      input: {
        orderId: "7",
        reason: "Color mismatch",
        items: [{ orderDetailId: "91" }, { orderDetailId: "92" }],
      },
    });

    const json = (await res.json()) as { ok: boolean; data: { returnId: string } };
    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data.returnId).toBe("14");
  });

  it("maps failed_precondition GraphQL errors to 409", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "failed_precondition: Return window has closed" }],
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/returns", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"], reason: "Late request" }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    expect(res.status).toBe(409);
  });
});
