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
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
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

  it("rejects guest session header/body mismatches", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-a" },
      body: JSON.stringify({ guestSessionId: "guest-b" }),
    });

    const res = await POST(req);
    const json = await res.json();

    expect(res.status).toBe(403);
    expect(json.message).toContain("active guest session");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
    expect(mocks.callGraphqlAsInternalService).not.toHaveBeenCalled();
  });

  it("merges guest cart exactly once across repeated attempts", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");

    mocks.callGraphqlAsInternalService
      .mockResolvedValueOnce({
        data: { getCartItems: [{ cartId: "g1", userId: "0", variantId: "v1", quantity: "2" }] },
      })
      .mockResolvedValueOnce({ data: { deleteCartItem: [{ cartId: "g1" }] } })
      .mockResolvedValueOnce({ data: { getCartItems: [] } });

    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: { getCartItems: [{ cartId: "u1", userId: "12", variantId: "v1", quantity: "1" }] },
      })
      .mockResolvedValueOnce({
        data: { updateCartItem: [{ cartId: "u1", userId: "12", variantId: "v1", quantity: "3" }] },
      })
      .mockResolvedValueOnce({
        data: { getCartItems: [{ cartId: "u1", userId: "12", variantId: "v1", quantity: "3" }] },
      });

    const requestFor = () =>
      new Request("http://localhost/api/account/cart/merge", {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
        body: JSON.stringify({ guestSessionId: "guest-1", idempotencyKey: "merge-key" }),
      });

    const first = await POST(requestFor());
    const firstJson = await first.json();
    const second = await POST(requestFor());
    const secondJson = await second.json();

    expect(first.status).toBe(200);
    expect(firstJson.data.merged).toBe(1);
    expect(second.status).toBe(200);
    expect(secondJson.data.merged).toBe(0);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(3);
    expect(mocks.callGraphqlAsInternalService).toHaveBeenCalledTimes(3);
  });

  it("restores the guest cart if the customer merge write fails", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");

    mocks.callGraphqlAsInternalService
      .mockResolvedValueOnce({
        data: { getCartItems: [{ cartId: "g1", userId: "0", variantId: "v1", quantity: "2" }] },
      })
      .mockResolvedValueOnce({ data: { deleteCartItem: [{ cartId: "g1" }] } })
      .mockResolvedValueOnce({ data: { addCartItem: [{ cartId: "g2", userId: "0", variantId: "v1", quantity: "2" }] } });

    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({ data: { getCartItems: [] } })
      .mockResolvedValueOnce({ errors: [{ message: "customer add failed" }] });

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
      body: JSON.stringify({ guestSessionId: "guest-1", idempotencyKey: "merge-key" }),
    });

    const res = await POST(req);

    expect(res.status).toBe(400);
    expect(mocks.callGraphqlAsInternalService).toHaveBeenNthCalledWith(
      3,
      expect.stringContaining("AddCartItem"),
      {
        input: {
          userId: null,
          variantId: "v1",
          quantity: "2",
          sessionId: "guest-1",
        },
      },
      { "Idempotency-Key": "merge-key:restore:g1" }
    );
  });
});
