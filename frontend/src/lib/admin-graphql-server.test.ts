import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getAdminSession: vi.fn(),
  graphQlUrl: vi.fn(() => "http://localhost:8080/v2"),
  forwardedIpHeadersFromRequest: vi.fn(() => ({ "X-Forwarded-For": "127.0.0.1" })),
  fetchWithResilience: vi.fn(),
}));

vi.mock("server-only", () => ({}));

vi.mock("@/lib/admin-auth-server", () => ({
  getAdminSession: (...args: unknown[]) => mocks.getAdminSession(...args),
}));

vi.mock("@/lib/server-session-auth", () => ({
  graphQlUrl: (...args: unknown[]) => mocks.graphQlUrl(...args),
}));

vi.mock("@/lib/forwarded-ip", () => ({
  forwardedIpHeadersFromRequest: (...args: unknown[]) => mocks.forwardedIpHeadersFromRequest(...args),
}));

vi.mock("@/lib/network-resilience", () => ({
  fetchWithResilience: (...args: unknown[]) => mocks.fetchWithResilience(...args),
  normalizeNetworkError: () => "network failure",
}));

import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

describe("forwardAdminGraphql", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("rejects unauthenticated admin API requests", async () => {
    mocks.getAdminSession.mockResolvedValue(null);
    const request = new Request("http://localhost/api/admin/orders", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query: "query Q { searchOrder(search: { userId: \"\" }) { orderId } }" }),
    });

    const response = await forwardAdminGraphql(request);
    const body = (await response.json()) as { ok: boolean; errorCode: string };

    expect(response.status).toBe(401);
    expect(body.ok).toBe(false);
    expect(body.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.fetchWithResilience).not.toHaveBeenCalled();
  });

  it("rejects customer sessions (non-admin) before GraphQL forwarding", async () => {
    // getAdminSession contract returns null for non-admin users.
    mocks.getAdminSession.mockResolvedValue(null);
    const request = new Request("http://localhost/api/admin/orders", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query: "query Q { searchOrder(search: { userId: \"\" }) { orderId } }" }),
    });

    const response = await forwardAdminGraphql(request);
    expect(response.status).toBe(401);
    expect(mocks.fetchWithResilience).not.toHaveBeenCalled();
  });

  it("allows admin requests and forwards bearer auth without browser session header", async () => {
    mocks.getAdminSession.mockResolvedValue({
      user: { email: "admin@example.com" },
      idToken: "jwt.token.value",
    });
    mocks.fetchWithResilience.mockResolvedValue(
      new Response(
        JSON.stringify({
          data: {
            searchOrder: [{ orderId: "1001" }],
          },
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      )
    );

    const request = new Request("http://localhost/api/admin/orders", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-session-id": "guest-session-should-not-forward",
      },
      body: JSON.stringify({ query: "query Q { searchOrder(search: { userId: \"\" }) { orderId } }" }),
    });

    const response = await forwardAdminGraphql(request);
    const body = (await response.json()) as { ok: boolean; data?: { searchOrder?: Array<{ orderId: string }> } };

    expect(response.status).toBe(200);
    expect(body.ok).toBe(true);
    expect(body.data?.searchOrder?.[0]?.orderId).toBe("1001");
    expect(mocks.fetchWithResilience).toHaveBeenCalledTimes(1);

    const [url, init] = mocks.fetchWithResilience.mock.calls[0] as [string, RequestInit];
    const headers = new Headers(init.headers);
    expect(url).toBe("http://localhost:8080/v2");
    expect(headers.get("Authorization")).toBe("Bearer jwt.token.value");
    expect(headers.get("X-Session-Id")).toBeNull();
  });
});
