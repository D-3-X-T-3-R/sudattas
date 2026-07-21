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

import { POST } from "@/app/api/account/cart/merge/route";

describe("cart merge route", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("calls the mergeCart mutation and returns the merged cart", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: {
        mergeCart: [
          { cartId: "u1", userId: "12", variantId: "v1", quantity: "3" },
          { cartId: "u2", userId: "12", variantId: "v2", quantity: "1" },
        ],
      },
    });

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
      body: JSON.stringify({ guestSessionId: "guest-1", idempotencyKey: "merge-key" }),
    });

    const res = await POST(req);
    const json = (await res.json()) as {
      ok: boolean;
      data: { items: Array<{ cartId: string; variantId: string; quantity: string }> };
    };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data.items).toHaveLength(2);
    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledWith(
      "12",
      expect.stringContaining("MergeCart"),
      { sessionId: "guest-1" },
      { "Idempotency-Key": "merge-key" }
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
  });

  it("rejects unauthenticated requests", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
      body: JSON.stringify({ guestSessionId: "guest-1" }),
    });

    const res = await POST(req);
    expect(res.status).toBe(401);
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("surfaces a GraphQL error from mergeCart", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("12");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "merge failed" }],
    });

    const req = new Request("http://localhost/api/account/cart/merge", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Guest-Session-Id": "guest-1" },
      body: JSON.stringify({ guestSessionId: "guest-1" }),
    });

    const res = await POST(req);
    const json = await res.json();

    expect(res.status).toBe(400);
    expect(json.message).toBe("merge failed");
  });
});
