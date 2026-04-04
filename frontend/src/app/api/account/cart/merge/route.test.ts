import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn(),
  callGraphqlAsInternalService: vi.fn(),
}));

vi.mock("@/lib/server-session-auth", () => ({
  apiError: (message: string, status: number, errorCode: string) =>
    Response.json({ ok: false, data: null, errorCode, message, fieldErrors: null, retryable: status >= 500 }, { status }),
  requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
  callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
  callGraphqlAsInternalService: mocks.callGraphqlAsInternalService,
}));

import { POST } from "@/app/api/account/cart/merge/route";

describe("cart merge route", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
    mocks.callGraphqlAsInternalService.mockReset();
  });

  it("merges guest lines into customer cart and deletes guest lines", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");

    mocks.callGraphqlAsInternalService.mockImplementation(async (query: string) => {
      if (query.includes("GetCartItems")) {
        return { data: { getCartItems: [{ cartId: "g1", userId: "0", variantId: "v1", quantity: "2" }] } };
      }
      return { data: { deleteCartItem: [{ cartId: "g1" }] } };
    });

    mocks.callGraphqlAsCustomer.mockImplementation(async (_uid: string, query: string) => {
      if (query.includes("GetCartItems")) {
        return { data: { getCartItems: [{ cartId: "u1", userId: "12", variantId: "v1", quantity: "1" }] } };
      }
      return { data: { updateCartItem: [{ cartId: "u1", userId: "12", variantId: "v1", quantity: "3" }] } };
    });

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ guestSessionId: "guest-1", idempotencyKey: "merge-key" }),
    });

    const res = await POST(req);
    const json = (await res.json()) as { ok: boolean; data: { merged: number; deletedGuestItems: number } };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data.merged).toBe(1);
    expect(json.data.deletedGuestItems).toBe(1);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledWith(
      "12",
      expect.stringContaining("UpdateCartItem"),
      expect.any(Object),
      { "Idempotency-Key": "merge-key:up:g1" }
    );
    expect(mocks.callGraphqlAsInternalService).toHaveBeenCalledWith(
      expect.stringContaining("DeleteCartItem"),
      expect.any(Object),
      { "Idempotency-Key": "merge-key:del:g1" }
    );
  });
});