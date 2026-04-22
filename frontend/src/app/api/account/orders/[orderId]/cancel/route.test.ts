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

import { POST } from "@/app/api/account/orders/[orderId]/cancel/route";

describe("POST /api/account/orders/[orderId]/cancel", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await POST(new Request("http://localhost/api/account/orders/7/cancel"), {
      params: Promise.resolve({ orderId: "7" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns validation error when order id is empty", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");

    const res = await POST(new Request("http://localhost/api/account/orders/ /cancel"), {
      params: Promise.resolve({ orderId: "   " }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(400);
    expect(json.errorCode).toBe("VALIDATION_ERROR");
  });

  it("forwards deleteOrder mutation and returns success payload", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: { deleteOrder: [{ orderId: "7", statusId: "10" }] },
    });

    const res = await POST(new Request("http://localhost/api/account/orders/7/cancel"), {
      params: Promise.resolve({ orderId: " 7 " }),
    });
    const json = (await res.json()) as {
      ok: boolean;
      data: { orderId: string; statusId: string };
    };

    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
    const [, , variables] = mocks.callGraphqlAsCustomer.mock.calls[0];
    expect(variables).toEqual({ orderId: "7" });

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data).toEqual({ orderId: "7", statusId: "10" });
  });

  it("maps failed_precondition GraphQL errors to 409", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "failed_precondition: cancellation window closed" }],
    });

    const res = await POST(new Request("http://localhost/api/account/orders/7/cancel"), {
      params: Promise.resolve({ orderId: "7" }),
    });
    const json = (await res.json()) as { errorCode: string; message: string };

    expect(res.status).toBe(409);
    expect(json.errorCode).toBe("GRAPHQL_ERROR");
    expect(json.message).toContain("failed_precondition");
  });

  it("maps not-found GraphQL errors to 404", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "order not found" }],
    });

    const res = await POST(new Request("http://localhost/api/account/orders/404/cancel"), {
      params: Promise.resolve({ orderId: "404" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(404);
    expect(json.errorCode).toBe("GRAPHQL_ERROR");
  });
});
