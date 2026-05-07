// @vitest-environment node

describe("robots", () => {
  const originalSiteUrl = process.env.NEXT_PUBLIC_SITE_URL;

  beforeEach(() => {
    process.env.NEXT_PUBLIC_SITE_URL = "https://launch.sudattas.com/";
    vi.resetModules();
  });

  afterEach(() => {
    process.env.NEXT_PUBLIC_SITE_URL = originalSiteUrl;
    vi.resetModules();
  });

  it("disallows private, account, admin, and API paths", async () => {
    const { default: robots } = await import("@/app/robots");
    const rules = robots();
    const disallow = Array.isArray(rules.rules) ? rules.rules[0]?.disallow : rules.rules.disallow;

    expect(disallow).toEqual(
      expect.arrayContaining([
        "/bag",
        "/wishlist",
        "/profile",
        "/checkout",
        "/imtheboss",
        "/api",
        "/account",
      ])
    );
    expect(rules.sitemap).toBe("https://launch.sudattas.com/sitemap.xml");
  });
});
