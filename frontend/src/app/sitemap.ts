import type { MetadataRoute } from "next";
import { siteUrl } from "@/lib/site-url";
import { graphqlBaseUrl } from "@/lib/env/server";

type BackendSitemapUrl = { loc: string; lastmod: string | null };

async function fetchBackendSitemapUrls(): Promise<BackendSitemapUrl[]> {
  try {
    const res = await fetch(`${graphqlBaseUrl()}/sitemap.xml`, { cache: "no-store" });
    if (!res.ok) return [];
    const xml = await res.text();
    const urls: BackendSitemapUrl[] = [];
    const urlBlocks = xml.match(/<url>[\s\S]*?<\/url>/g) ?? [];
    for (const block of urlBlocks) {
      const loc = block.match(/<loc>(.*?)<\/loc>/)?.[1]?.trim();
      if (!loc) continue;
      const lastmod = block.match(/<lastmod>(.*?)<\/lastmod>/)?.[1]?.trim() ?? null;
      urls.push({ loc, lastmod });
    }
    return urls;
  } catch {
    return [];
  }
}

function normalizeBackendLocToFrontend(loc: string, base: string): string {
  try {
    const parsed = new URL(loc);
    // Backend SEO currently emits `/products/{slug}` while frontend route is `/product/{idOrSlug}`.
    if (parsed.pathname.startsWith("/products/")) {
      const tail = parsed.pathname.replace(/^\/products\//, "");
      return `${base}/product/${tail}`;
    }
    return `${base}${parsed.pathname}`;
  } catch {
    return loc;
  }
}

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const base = siteUrl();
  const now = new Date();
  const backendUrls = await fetchBackendSitemapUrls();
  const productUrls: MetadataRoute.Sitemap = backendUrls.map((u) => ({
    url: normalizeBackendLocToFrontend(u.loc, base),
    lastModified: u.lastmod ? new Date(u.lastmod) : now,
    changeFrequency: "daily",
    priority: 0.9,
  }));

  return [
    {
      url: `${base}/`,
      lastModified: now,
      changeFrequency: "daily",
      priority: 1,
    },
    {
      url: `${base}/wishlist`,
      lastModified: now,
      changeFrequency: "daily",
      priority: 0.8,
    },
    {
      url: `${base}/bag`,
      lastModified: now,
      changeFrequency: "daily",
      priority: 0.8,
    },
    {
      url: `${base}/profile`,
      lastModified: now,
      changeFrequency: "weekly",
      priority: 0.6,
    },
    ...productUrls,
  ];
}



