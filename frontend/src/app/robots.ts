import type { MetadataRoute } from "next";

function siteUrl(): string {
  const raw = process.env.NEXT_PUBLIC_SITE_URL?.trim();
  if (!raw) return "https://www.sudattas.com";
  return raw.endsWith("/") ? raw.slice(0, -1) : raw;
}

export default function robots(): MetadataRoute.Robots {
  const base = siteUrl();
  return {
    rules: {
      userAgent: "*",
      allow: "/",
      disallow: ["/imtheboss", "/api"],
    },
    sitemap: `${base}/sitemap.xml`,
  };
}

