import type { MetadataRoute } from "next";

function siteUrl(): string {
  const raw = process.env.NEXT_PUBLIC_SITE_URL?.trim();
  if (!raw) return "https://www.sudattas.com";
  return raw.endsWith("/") ? raw.slice(0, -1) : raw;
}

export default function sitemap(): MetadataRoute.Sitemap {
  const base = siteUrl();
  const now = new Date();

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
  ];
}

