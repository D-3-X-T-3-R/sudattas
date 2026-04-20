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

import { POST } from "@/app/api/account/orders/[orderId]/cancel-items/route";

describe("POST /api/account/orders/[orderId]/cancel-items", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns 400 when no orderDetailIds are provided", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    const res = await POST(
      new Request("http://localhost/api/account/orders/7/cancel-items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: [] }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    const json = (await res.json()) as { errorCode: string };
    expect(res.status).toBe(400);
    expect(json.errorCode).toBe("VALIDATION_ERROR");
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/cancel-items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"] }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("forwards cancelOrderItems mutation and returns success payload", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: { cancelOrderItems: [{ orderId: "7", statusId: "10" }] },
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/cancel-items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"] }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );

    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
    const [, , variables] = mocks.callGraphqlAsCustomer.mock.calls[0];
    expect(variables).toEqual({ input: { orderId: "7", orderDetailIds: ["91"] } });

    const json = (await res.json()) as {
      ok: boolean;
      data: { orderId: string; statusId: string };
    };
    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data).toEqual({ orderId: "7", statusId: "10" });
  });

  it("maps failed_precondition GraphQL error to 409", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "failed_precondition: cancellation window closed" }],
    });

    const res = await POST(
      new Request("http://localhost/api/account/orders/7/cancel-items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ orderDetailIds: ["91"] }),
      }),
      { params: Promise.resolve({ orderId: "7" }) }
    );
    const json = (await res.json()) as { errorCode: string; message: string };

    expect(res.status).toBe(409);
    expect(json.errorCode).toBe("GRAPHQL_ERROR");
    expect(json.message).toContain("failed_precondition");
  });
});
