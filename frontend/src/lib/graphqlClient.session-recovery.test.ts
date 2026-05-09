import { beforeEach, describe, expect, it, vi } from "vitest";

const fetchWithResilienceMock = vi.fn();
const normalizeNetworkErrorMock = vi.fn((error: unknown) => String(error));
const publicGraphqlUrlMock = vi.fn(() => "http://127.0.0.1:8080/v2");

let accessToken: string | null = null;

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

describe("graphqlClient stale guest session recovery (storage integration)", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    localStorage.clear();
    accessToken = null;
  });

  it("clears stale localStorage session, mints a new session, and retries once after 401", async () => {
    localStorage.setItem("sudattas_guest_session", "stale-session-1");

    let graphqlAttempts = 0;
    let guestMintCalls = 0;
    const sessionHeaders: Array<string | null> = [];
    const sessionIdsInPayload: Array<string | null> = [];
    let stalePresentWhenMinted: string | null = null;

    fetchWithResilienceMock.mockImplementation(
      async (
        url: string,
        init?: { headers?: Record<string, string>; body?: BodyInit | null }
      ) => {
        if (url.endsWith("/session/guest")) {
          guestMintCalls += 1;
          stalePresentWhenMinted = localStorage.getItem("sudattas_guest_session");
          return new Response(JSON.stringify({ session_id: "fresh-session-2" }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          });
        }

        if (url.endsWith("/v2")) {
          graphqlAttempts += 1;
          const sid =
            (init?.headers?.["X-Session-Id"] as string | undefined) ??
            (init?.headers?.["x-session-id"] as string | undefined) ??
            null;
          sessionHeaders.push(sid);
          const body = JSON.parse(String(init?.body ?? "{}")) as {
            variables?: { input?: { sessionId?: string | null } };
          };
          sessionIdsInPayload.push(body.variables?.input?.sessionId ?? null);

          if (graphqlAttempts === 1) {
            return new Response("UNAUTHORIZED", { status: 401 });
          }

          return new Response(JSON.stringify({ data: { ok: true } }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          });
        }

        throw new Error(`unexpected URL: ${url}`);
      }
    );

    const { gql } = await import("@/lib/graphqlClient");
    const result = await gql<{ ok: boolean }>(
      "mutation AddCartItem($input: NewCart!) { addCartItem(cartItem: $input) { cartId } }",
      {
        input: {
          userId: "",
          variantId: "1",
          quantity: "1",
          sessionId: "stale-session-1",
        },
      }
    );

    expect(result).toEqual({ ok: true });
    expect(graphqlAttempts).toBe(2);
    expect(guestMintCalls).toBe(1);
    expect(sessionHeaders).toEqual(["stale-session-1", "fresh-session-2"]);
    expect(sessionIdsInPayload).toEqual(["stale-session-1", "fresh-session-2"]);
    expect(stalePresentWhenMinted).toBeNull();
    expect(localStorage.getItem("sudattas_guest_session")).toBe("fresh-session-2");
  });
});
