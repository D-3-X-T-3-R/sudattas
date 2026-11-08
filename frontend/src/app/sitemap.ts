import type { MetadataRoute } from "next";
import { siteUrl } from "@/lib/site-url";
import { graphqlBaseUrl } from "@/lib/env/server";
import { fetchCategoriesWithSession } from "@/lib/storefront-queries";
import { isPublicCatalogName, slugifyCategoryName } from "@/lib/storefront-collection-page";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";

type BackendSitemapUrl = { loc: string; lastmod: string | null };

const PRIVATE_PREFIXES = [
  "/bag",
  "/wishlist",
  "/profile",
  "/checkout",
  "/imtheboss",
  "/api",
  "/account",
];

const STATIC_PUBLIC_ROUTES = [
  "/",
  "/collections",
  "/journal",
  "/shipping-policy",
  "/returns-exchanges",
  "/privacy-policy",
  "/terms-conditions",
  "/contact-support",
  "/about",
  "/cancellation-policy",
  "/payment-guide",
  "/size-fit-guide",
];

/** Category/collection pages — the canonical URL is /collections/{slug}; /category/{id}
 * redirects there permanently (see app/category/[categoryId]/page.tsx), so only the
 * collections form is submitted here. */
async function fetchCategorySitemapUrls(): Promise<{ slug: string }[]> {
  try {
    const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
    const sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
    const recovered = await withRecoveredGuestSession(sessionId, forwardedHeaders, async (activeSessionId) =>
      fetchCategoriesWithSession(activeSessionId, forwardedHeaders)
    );
    const seen = new Set<string>();
    const slugs: { slug: string }[] = [];
    for (const category of recovered.value) {
      if (!isPublicCatalogName(category.name)) continue;
      const slug = slugifyCategoryName(category.name);
      if (!slug || seen.has(slug)) continue;
      seen.add(slug);
      slugs.push({ slug });
    }
    return slugs;
  } catch {
    return [];
  }
}

function isIndexablePath(pathname: string): boolean {
  if (!pathname.startsWith("/")) return false;
  return !PRIVATE_PREFIXES.some(
    (prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`)
  );
}

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

function normalizeBackendLocToPath(loc: string): string | null {
  try {
    const parsed = new URL(loc);
    // Backend SEO currently emits `/products/{slug}` while frontend route is `/product/{idOrSlug}`.
    if (parsed.pathname.startsWith("/products/")) {
      const tail = parsed.pathname.replace(/^\/products\//, "").trim();
      return tail ? `/product/${tail}` : null;
    }
    return parsed.pathname;
  } catch {
    return null;
  }
}

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const base = siteUrl();
  const now = new Date();

  const staticUrls: MetadataRoute.Sitemap = STATIC_PUBLIC_ROUTES.map((route) => ({
    url: `${base}${route === "/" ? "" : route}`,
    lastModified: now,
    changeFrequency: route === "/" ? "daily" : "weekly",
    priority: route === "/" ? 1 : 0.7,
  }));

  const categorySlugs = await fetchCategorySitemapUrls();
  const categoryUrls: MetadataRoute.Sitemap = categorySlugs.map(({ slug }) => ({
    url: `${base}/collections/${encodeURIComponent(slug)}`,
    lastModified: now,
    changeFrequency: "weekly" as const,
    priority: 0.8,
  }));

  const backendUrls = await fetchBackendSitemapUrls();
  const productUrls: MetadataRoute.Sitemap = backendUrls
    .map((entry) => ({
      path: normalizeBackendLocToPath(entry.loc),
      lastmod: entry.lastmod,
    }))
    .filter((entry): entry is { path: string; lastmod: string | null } => Boolean(entry.path))
    .filter((entry) => isIndexablePath(entry.path))
    .map((entry) => ({
      url: `${base}${entry.path}`,
      lastModified: entry.lastmod ? new Date(entry.lastmod) : now,
      changeFrequency: "daily" as const,
      priority: 0.9,
    }));

  const deduped = new Map<string, MetadataRoute.Sitemap[number]>();
  for (const row of [...staticUrls, ...categoryUrls, ...productUrls]) {
    deduped.set(row.url, row);
  }

  return [...deduped.values()];
}
