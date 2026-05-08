import type { Metadata } from "next";
import Link from "next/link";
import { SiteHeader } from "@/components/site-header";
import { Footer } from "@/components/footer";
import { PageShell, SectionHeader, EmptyState } from "@/components/ui/page-shell";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { fetchCategoriesWithSession } from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { siteUrl } from "@/lib/site-url";
import { slugifyCategoryName } from "@/lib/storefront-collection-page";

export const metadata: Metadata = {
  title: "Collections | Sudatta's",
  description: "Browse collections at Sudatta's.",
};

export const dynamic = "force-dynamic";

export default async function CollectionsIndexPage() {
  let categories: { categoryId: string; name: string }[] = [];
  try {
    const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
    const sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
    const recovered = await withRecoveredGuestSession(
      sessionId,
      forwardedHeaders,
      async (activeSessionId) =>
        fetchCategoriesWithSession(activeSessionId, forwardedHeaders)
    );
    categories = recovered.value;
  } catch {
    categories = [];
  }
  const base = siteUrl();
  const breadcrumbJsonLd = {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: [
      {
        "@type": "ListItem",
        position: 1,
        name: "Home",
        item: `${base}/`,
      },
      {
        "@type": "ListItem",
        position: 2,
        name: "Collections",
        item: `${base}/collections`,
      },
    ],
  };

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <SiteHeader />
      <PageShell containerClassName="py-8 md:py-10">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbJsonLd) }}
        />
        <SectionHeader
          label="Storefront"
          title="Collections"
          description="Discover curated categories designed for occasions, gifting, and everyday elegance."
        />

        {categories.length === 0 ? (
          <div className="mt-8">
            <EmptyState
              title="Collections unavailable"
              description="Collections are not available right now."
            />
          </div>
        ) : (
          <section className="mt-8 grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 md:gap-5">
            {categories.map((category) => (
              <Link
                key={category.categoryId}
                href={`/collections/${encodeURIComponent(slugifyCategoryName(category.name))}`}
                className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-4 shadow-[var(--shadow-subtle)] transition hover:border-[var(--color-gold)]"
              >
                <p className="font-display text-lg tracking-tight text-[var(--color-ink)] sm:text-xl">
                  {category.name}
                </p>
                <p className="mt-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-green)]">
                  Explore
                </p>
              </Link>
            ))}
          </section>
        )}
      </PageShell>
      <Footer />
    </div>
  );
}
