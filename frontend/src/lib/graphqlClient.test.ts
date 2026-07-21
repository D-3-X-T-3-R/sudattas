import { beforeEach, describe, expect, it, vi } from "vitest";

const fetchWithResilienceMock = vi.fn();
const normalizeNetworkErrorMock = vi.fn((error: unknown) => String(error));
const publicGraphqlUrlMock = vi.fn(() => "http://127.0.0.1:8080/v2");

let accessToken: string | null = null;
let guestSessionId: string | null = null;

const ensureGuestSessionMock = vi.fn(async () => guestSessionId);
const refreshGuestSessionMock = vi.fn(async () => {
  guestSessionId = "guest-refreshed";
  return guestSessionId;
});
const getGuestSessionIdMock = vi.fn(() => guestSessionId);

vi.mock("@/lib/network-resilience", () => ({
  fetchWithResilience: (...args: unknown[]) => fetchWithResilienceMock(...args),
  normalizeNetworkError: (...args: unknown[]) => normalizeNetworkErrorMock(...args),
}));

vi.mock("@/lib/env/public", () => ({
  publicGraphqlUrl: () => publicGraphqlUrlMock(),
}));

vi.mock("@/lib/authStore", () => ({
  getAccessToken: () => accessToken,
}));

vi.mock("@/lib/session", () => ({
  ensureGuestSession: (...args: unknown[]) => ensureGuestSessionMock(...args),
  refreshGuestSession: (...args: unknown[]) => refreshGuestSessionMock(...args),
  getGuestSessionId: () => getGuestSessionIdMock(),
}));

describe("graphqlClient guest session recovery", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    accessToken = null;
    guestSessionId = "guest-initial";
  });

  it("refreshes and retries once on 401 for guest session auth", async () => {
    fetchWithResilienceMock
      .mockResolvedValueOnce(new Response("UNAUTHORIZED", { status: 401 }))
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ data: { ok: true } }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        })
      );

    const { gql } = await import("@/lib/graphqlClient");
    const result = await gql<{ ok: boolean }>("query Ping { apiVersion }");

    expect(result).toEqual({ ok: true });
    expect(refreshGuestSessionMock).toHaveBeenCalledTimes(1);
    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(2);
  });

  it("refreshes and retries once when auth failure text is surfaced with non-401 status", async () => {
    fetchWithResilienceMock
      .mockResolvedValueOnce(new Response("UNAUTHORIZED", { status: 500 }))
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ data: { ok: true } }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        })
      );

    const { gql } = await import("@/lib/graphqlClient");
    const result = await gql<{ ok: boolean }>("query Ping { apiVersion }");

    expect(result).toEqual({ ok: true });
    expect(refreshGuestSessionMock).toHaveBeenCalledTimes(1);
    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(2);
  });

  it("does not retry on generic 5xx for guest session requests", async () => {
    fetchWithResilienceMock.mockResolvedValueOnce(
      new Response("INTERNAL_SERVER_ERROR", { status: 500 })
    );

    const { gql } = await import("@/lib/graphqlClient");
    await expect(gql("query Ping { apiVersion }")).rejects.toThrow("INTERNAL_SERVER_ERROR");
    expect(refreshGuestSessionMock).not.toHaveBeenCalled();
    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(1);
  });

  it("does not run guest-session recovery when bearer auth is used", async () => {
    accessToken = "Bearer token-123";
    fetchWithResilienceMock.mockResolvedValueOnce(
      new Response("INTERNAL_SERVER_ERROR", { status: 500 })
    );

    const { gql } = await import("@/lib/graphqlClient");

    await expect(gql("query Ping { apiVersion }")).rejects.toThrow("INTERNAL_SERVER_ERROR");
    expect(refreshGuestSessionMock).not.toHaveBeenCalled();
    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(1);
  });
});
