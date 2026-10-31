import type { MetadataRoute } from "next";
import { siteUrl } from "@/lib/site-url";

export default function robots(): MetadataRoute.Robots {
  const base = siteUrl();
  return {
    rules: {
      userAgent: "*",
      allow: ["/", "/collections", "/product", "/category", "/journal", "/shipping-policy", "/returns-exchanges", "/privacy-policy", "/terms-conditions", "/contact-support", "/about", "/cancellation-policy", "/payment-guide", "/size-fit-guide"],
      disallow: [
        "/bag",
        "/wishlist",
        "/profile",
        "/checkout",
        "/imtheboss",
        "/api",
        "/account",
      ],
    },
    sitemap: `${base}/sitemap.xml`,
  };
}
