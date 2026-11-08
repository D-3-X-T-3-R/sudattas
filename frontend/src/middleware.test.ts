// @vitest-environment node

import { NextRequest } from "next/server";
import { ACCOUNT_DEACTIVATED_PATH, middleware } from "@/middleware";
import {
  STOREFRONT_COMING_SOON_PATH,
  STOREFRONT_GATE_HEADER,
  STOREFRONT_GATE_REASON_HEADER,
  STOREFRONT_GATE_REASON_PARAM,
  resetStorefrontBackendAvailabilityCache,
} from "@/lib/storefront-readiness";

const getTokenMock = vi.hoisted(() => vi.fn());

vi.mock("next-auth/jwt", () => ({
  getToken: getTokenMock,
}));

function requestFor(pathname: string): NextRequest {
  return new NextRequest(`https://launch.sudattas.com${pathname}`);
}

function requestOverrideHeader(response: Response, name: string): string | null {
  return response.headers.get(`x-middleware-request-${name}`);
}

function rewriteUrl(response: Response): URL | null {
  const rewrite = response.headers.get("x-middleware-rewrite");
  return rewrite ? new URL(rewrite) : null;
}

function mockBackendHealth(status = 200) {
  const fetchMock = vi.fn(() =>
    Promise.resolve(new Response(status === 200 ? "OK" : "Unavailable", { status }))
  );
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

function restoreEnv(name: string, value: string | undefined): void {
  if (value === undefined) {
    delete process.env[name];
    return;
  }

  process.env[name] = value;
}

describe("middleware storefront gate", () => {
  const originalReady = process.env.IS_STOREFRONT_READY;
  const originalGraphqlUrl = process.env.GRAPHQL_URL;
  const originalPublicGraphqlUrl = process.env.NEXT_PUBLIC_GRAPHQL_URL;

  beforeEach(() => {
    getTokenMock.mockReset();
    resetStorefrontBackendAvailabilityCache();
    process.env.GRAPHQL_URL = "https://backend.sudattas.test/v2";
    delete process.env.NEXT_PUBLIC_GRAPHQL_URL;
  });

  afterEach(() => {
    restoreEnv("IS_STOREFRONT_READY", originalReady);
    restoreEnv("GRAPHQL_URL", originalGraphqlUrl);
    restoreEnv("NEXT_PUBLIC_GRAPHQL_URL", originalPublicGraphqlUrl);
    resetStorefrontBackendAvailabilityCache();
    vi.unstubAllGlobals();
  });

  it("rewrites public storefront routes to the coming-soon page when storefront is not ready", async () => {
    process.env.IS_STOREFRONT_READY = "0";
    const fetchMock = mockBackendHealth();

    const response = await middleware(requestFor("/collections"));
    const rewrite = rewriteUrl(response);

    expect(rewrite?.origin).toBe("https://launch.sudattas.com");
    expect(rewrite?.pathname).toBe(STOREFRONT_COMING_SOON_PATH);
    expect(rewrite?.searchParams.get(STOREFRONT_GATE_REASON_PARAM)).toBe(
      "not-ready"
    );
    expect(requestOverrideHeader(response, STOREFRONT_GATE_HEADER)).toBe("1");
    expect(requestOverrideHeader(response, STOREFRONT_GATE_REASON_HEADER)).toBe(
      "not-ready"
    );
    expect(fetchMock).not.toHaveBeenCalled();
    expect(getTokenMock).not.toHaveBeenCalled();
  });

  it("keeps admin login outside the storefront gate", async () => {
    process.env.IS_STOREFRONT_READY = "0";
    const fetchMock = mockBackendHealth(503);

    const response = await middleware(requestFor("/imtheboss/login"));

    expect(response.headers.get("x-middleware-rewrite")).toBeNull();
    expect(response.headers.get("location")).toBeNull();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(getTokenMock).not.toHaveBeenCalled();
  });

  it("keeps authenticated admin pages accessible when storefront is gated", async () => {
    process.env.IS_STOREFRONT_READY = "0";
    getTokenMock.mockResolvedValue({ isAdmin: true });

    const response = await middleware(requestFor("/imtheboss"));

    expect(response.headers.get("x-middleware-rewrite")).toBeNull();
    expect(response.headers.get("location")).toBeNull();
    expect(getTokenMock).toHaveBeenCalledTimes(1);
  });

  it("does not rewrite public storefront routes when storefront is ready and backend is healthy", async () => {
    process.env.IS_STOREFRONT_READY = "1";
    const fetchMock = mockBackendHealth();

    const response = await middleware(requestFor("/product/1"));

    expect(response.headers.get("x-middleware-rewrite")).toBeNull();
    expect(response.headers.get("location")).toBeNull();
    expect(fetchMock).toHaveBeenCalledWith(
      "https://backend.sudattas.test/ready",
      expect.objectContaining({
        cache: "no-store",
        method: "GET",
      })
    );
    // Checked here now (no active session in this test, so getToken resolves to null/undefined
    // and nothing redirects) — see the "account deactivation" describe block below for the
    // actual redirect behavior this check exists for.
    expect(getTokenMock).toHaveBeenCalledTimes(1);
  });

  it("rewrites public storefront routes when storefront is ready but backend is unavailable", async () => {
    process.env.IS_STOREFRONT_READY = "1";
    const fetchMock = mockBackendHealth(503);

    const response = await middleware(requestFor("/bag"));
    const rewrite = rewriteUrl(response);

    expect(rewrite?.pathname).toBe(STOREFRONT_COMING_SOON_PATH);
    expect(rewrite?.searchParams.get(STOREFRONT_GATE_REASON_PARAM)).toBe(
      "service-unavailable"
    );
    expect(requestOverrideHeader(response, STOREFRONT_GATE_HEADER)).toBe("1");
    expect(requestOverrideHeader(response, STOREFRONT_GATE_REASON_HEADER)).toBe(
      "service-unavailable"
    );
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(getTokenMock).not.toHaveBeenCalled();
  });

  it("treats backend health check errors as unavailable for public storefront routes", async () => {
    process.env.IS_STOREFRONT_READY = "1";
    const fetchMock = vi.fn(() => Promise.reject(new Error("network down")));
    vi.stubGlobal("fetch", fetchMock);

    const response = await middleware(requestFor("/wishlist"));
    const rewrite = rewriteUrl(response);

    expect(rewrite?.pathname).toBe(STOREFRONT_COMING_SOON_PATH);
    expect(rewrite?.searchParams.get(STOREFRONT_GATE_REASON_PARAM)).toBe(
      "service-unavailable"
    );
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("keeps static assets outside backend availability gating", async () => {
    process.env.IS_STOREFRONT_READY = "1";
    const fetchMock = mockBackendHealth(503);

    const response = await middleware(requestFor("/logo.png"));

    expect(response.headers.get("x-middleware-rewrite")).toBeNull();
    expect(response.headers.get("location")).toBeNull();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(getTokenMock).not.toHaveBeenCalled();
  });
});

describe("middleware account-deactivation gate", () => {
  const originalReady = process.env.IS_STOREFRONT_READY;
  const originalGraphqlUrl = process.env.GRAPHQL_URL;
  const originalInternalSecret = process.env.INTERNAL_API_SECRET;

  // This is a live check (internal-service auth against the backend), not a cached JWT claim —
  // see the comment on `isAccountDeactivatedLive` in middleware.ts for why. Each test uses its
  // own customerUserId to avoid colliding with the (deliberately short-lived) in-process cache
  // that function keeps across calls within this same test file run.
  function mockAccountStatusBackend(deactivated: boolean | null) {
    const fetchMock = vi.fn((url: unknown) => {
      if (typeof url === "string" && url.endsWith("/ready")) {
        return Promise.resolve(new Response("OK", { status: 200 }));
      }
      if (deactivated === null) {
        return Promise.resolve(
          new Response(JSON.stringify({ errors: [{ message: "boom" }] }), { status: 200 })
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({ data: { authInfo: { accountDeactivated: deactivated } } }),
          { status: 200 }
        )
      );
    });
    vi.stubGlobal("fetch", fetchMock);
    return fetchMock;
  }

  beforeEach(() => {
    getTokenMock.mockReset();
    process.env.IS_STOREFRONT_READY = "1";
    process.env.GRAPHQL_URL = "https://backend.sudattas.test/v2";
    process.env.INTERNAL_API_SECRET = "test-internal-secret";
  });

  afterEach(() => {
    restoreEnv("IS_STOREFRONT_READY", originalReady);
    restoreEnv("GRAPHQL_URL", originalGraphqlUrl);
    restoreEnv("INTERNAL_API_SECRET", originalInternalSecret);
    vi.unstubAllGlobals();
  });

  it("redirects a deactivated customer's session away from a normal storefront page", async () => {
    mockAccountStatusBackend(true);
    getTokenMock.mockResolvedValue({ customerUserId: "deact-1" });

    const response = await middleware(requestFor("/profile"));

    expect(response.headers.get("location")).toBe(
      `https://launch.sudattas.com${ACCOUNT_DEACTIVATED_PATH}`
    );
  });

  it("does not redirect an active customer's session", async () => {
    mockAccountStatusBackend(false);
    getTokenMock.mockResolvedValue({ customerUserId: "active-1" });

    const response = await middleware(requestFor("/profile"));

    expect(response.headers.get("location")).toBeNull();
  });

  it("does not redirect a guest with no session", async () => {
    mockAccountStatusBackend(false);
    getTokenMock.mockResolvedValue(null);

    const response = await middleware(requestFor("/profile"));

    expect(response.headers.get("location")).toBeNull();
  });

  it("does not redirect when the live status check can't be confirmed (fails open on backend errors)", async () => {
    mockAccountStatusBackend(null);
    getTokenMock.mockResolvedValue({ customerUserId: "unknown-1" });

    const response = await middleware(requestFor("/profile"));

    expect(response.headers.get("location")).toBeNull();
  });

  it("does not redirect when INTERNAL_API_SECRET is not configured", async () => {
    delete process.env.INTERNAL_API_SECRET;
    mockAccountStatusBackend(true);
    getTokenMock.mockResolvedValue({ customerUserId: "no-secret-1" });

    const response = await middleware(requestFor("/profile"));

    expect(response.headers.get("location")).toBeNull();
  });

  it("does not loop a deactivated customer back onto the account-deactivated page itself", async () => {
    getTokenMock.mockResolvedValue({ customerUserId: "deact-2" });

    const response = await middleware(requestFor(ACCOUNT_DEACTIVATED_PATH));

    expect(response.headers.get("location")).toBeNull();
    expect(getTokenMock).not.toHaveBeenCalled();
  });

  it("does not gate the admin login page for a deactivated customer session", async () => {
    getTokenMock.mockResolvedValue({ customerUserId: "deact-3" });

    const response = await middleware(requestFor("/imtheboss/login"));

    expect(response.headers.get("location")).toBeNull();
    expect(getTokenMock).not.toHaveBeenCalled();
  });
});
