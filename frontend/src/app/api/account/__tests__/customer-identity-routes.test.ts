import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn<
    (userId: string, query: string, variables?: Record<string, unknown>) => Promise<{ data?: unknown; errors?: Array<{ message?: string }> }>
  >(),
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

import { GET as getProfile } from "@/app/api/account/profile/route";
import { GET as getOrders } from "@/app/api/account/orders/route";
import { GET as getAddresses } from "@/app/api/account/addresses/route";

describe("account API routes use canonical customer identity", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("profile route resolves through customerUserId server auth", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: { exportMyPii: { userId: "42", email: "u@example.com", createDate: "2026-01-01T00:00:00Z" } },
    });

    const res = await getProfile();
    expect(res.status).toBe(200);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledWith("42", expect.stringContaining("query AccountProfile"));
  });

  it("orders route uses same canonical user id across downstream calls", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("77");
    mocks.callGraphqlAsCustomer.mockImplementation(async (_uid, query) => {
      if (query.includes("searchOrderStatus")) {
        return { data: { searchOrderStatus: [{ statusId: "1", statusName: "Pending" }] } };
      }
      return {
        data: {
          searchOrder: [
            {
              orderId: "1001",
              userId: "77",
              orderDate: "2026-01-01T00:00:00Z",
              totalAmountPaise: "1000",
              totalAmountFormatted: "Rs 10.00",
              statusId: "1",
            },
          ],
        },
      };
    });

    const res = await getOrders();
    const json = (await res.json()) as { ok: boolean };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(2);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(1, "77", expect.stringContaining("query AccountOrders"), expect.any(Object));
    expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(2, "77", expect.stringContaining("query AccountOrderStatuses"));
  });

  it("addresses route is unauthorized when canonical identity is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await getAddresses();
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });
});
