// @vitest-environment node

describe("sitemap", () => {
  const originalSiteUrl = process.env.NEXT_PUBLIC_SITE_URL;
  const fetchMock = vi.fn();

  beforeEach(() => {
    process.env.NEXT_PUBLIC_SITE_URL = "https://launch.sudattas.com/";
    fetchMock.mockReset();
    vi.resetModules();
    vi.unstubAllGlobals();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    process.env.NEXT_PUBLIC_SITE_URL = originalSiteUrl;
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  it("includes public pages and excludes private/session/admin/api routes", async () => {
    vi.doMock("@/lib/env/server", () => ({
      graphqlBaseUrl: () => "https://backend.example",
    }));

    fetchMock.mockResolvedValue({
      ok: true,
      text: async () => `
        <urlset>
          <url><loc>https://backend.example/products/amber-saree</loc><lastmod>2026-05-01</lastmod></url>
          <url><loc>https://backend.example/bag</loc></url>
          <url><loc>https://backend.example/api/private</loc></url>
        </urlset>
      `,
    });

    const { default: sitemap } = await import("@/app/sitemap");
    const rows = await sitemap();
    const urls = rows.map((row) => row.url);

    expect(urls).toContain("https://launch.sudattas.com");
    expect(urls).toContain("https://launch.sudattas.com/about");
    expect(urls).toContain("https://launch.sudattas.com/cancellation-policy");
    expect(urls).toContain("https://launch.sudattas.com/payment-guide");
    expect(urls).toContain("https://launch.sudattas.com/size-fit-guide");
    expect(urls).toContain("https://launch.sudattas.com/product/amber-saree");

    expect(urls).not.toContain("https://launch.sudattas.com/bag");
    expect(urls).not.toContain("https://launch.sudattas.com/wishlist");
    expect(urls).not.toContain("https://launch.sudattas.com/profile");
    expect(urls).not.toContain("https://launch.sudattas.com/checkout");
    expect(urls).not.toContain("https://launch.sudattas.com/imtheboss");
    expect(urls).not.toContain("https://launch.sudattas.com/api/private");
  });
});
