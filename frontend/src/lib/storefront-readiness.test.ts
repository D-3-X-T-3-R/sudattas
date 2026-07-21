// @vitest-environment node

import {
  STOREFRONT_COMING_SOON_PATH,
  isStorefrontReady,
  normalizeStorefrontGateReason,
  shouldGateStorefrontPath,
  storefrontBackendHealthUrl,
  storefrontGateReasonForPath,
} from "@/lib/storefront-readiness";

describe("storefront readiness", () => {
  const originalReady = process.env.IS_STOREFRONT_READY;

  afterEach(() => {
    process.env.IS_STOREFRONT_READY = originalReady;
  });

  it("treats only IS_STOREFRONT_READY=1 as ready", () => {
    process.env.IS_STOREFRONT_READY = "1";
    expect(isStorefrontReady()).toBe(true);

    process.env.IS_STOREFRONT_READY = "0";
    expect(isStorefrontReady()).toBe(false);

    delete process.env.IS_STOREFRONT_READY;
    expect(isStorefrontReady()).toBe(false);
  });

  it("gates public storefront routes while excluding admin, API, assets, and the coming-soon page", () => {
    expect(shouldGateStorefrontPath("/", false)).toBe(true);
    expect(shouldGateStorefrontPath("/collections/silk", false)).toBe(true);
    expect(shouldGateStorefrontPath("/product/1", false)).toBe(true);
    expect(shouldGateStorefrontPath("/bag", false)).toBe(true);
    expect(shouldGateStorefrontPath("/wishlist", false)).toBe(true);
    expect(shouldGateStorefrontPath("/checkout/success", false)).toBe(true);
    expect(shouldGateStorefrontPath("/profile/orders", false)).toBe(true);
    expect(shouldGateStorefrontPath("/privacy-policy", false)).toBe(true);

    expect(shouldGateStorefrontPath("/imtheboss", false)).toBe(false);
    expect(shouldGateStorefrontPath("/imtheboss/login", false)).toBe(false);
    expect(shouldGateStorefrontPath("/api/admin/graphql", false)).toBe(false);
    expect(shouldGateStorefrontPath("/_next/static/chunks/app.js", false)).toBe(false);
    expect(shouldGateStorefrontPath("/logo.png", false)).toBe(false);
    expect(shouldGateStorefrontPath(STOREFRONT_COMING_SOON_PATH, false)).toBe(false);

    expect(shouldGateStorefrontPath("/collections", true)).toBe(false);
    expect(shouldGateStorefrontPath("/collections", true, false)).toBe(true);
    expect(storefrontGateReasonForPath("/collections", true, false)).toBe(
      "service-unavailable"
    );
    expect(storefrontGateReasonForPath("/collections", false, true)).toBe(
      "not-ready"
    );
  });

  it("derives the backend readiness endpoint from the configured GraphQL URL", () => {
    expect(storefrontBackendHealthUrl("http://127.0.0.1:8080/v2")).toBe(
      "http://127.0.0.1:8080/ready"
    );
    expect(storefrontBackendHealthUrl("https://api.sudattas.com/graphql/v2")).toBe(
      "https://api.sudattas.com/graphql/ready"
    );
    expect(storefrontBackendHealthUrl("not a url")).toBeNull();
  });

  it("normalizes unknown gate reasons to the launch gate copy", () => {
    expect(normalizeStorefrontGateReason("service-unavailable")).toBe(
      "service-unavailable"
    );
    expect(normalizeStorefrontGateReason("not-ready")).toBe("not-ready");
    expect(normalizeStorefrontGateReason(undefined)).toBe("not-ready");
  });
});
