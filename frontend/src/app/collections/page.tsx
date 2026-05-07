import type { Metadata } from "next";
import Link from "next/link";
import { SiteHeader } from "@/components/site-header";
import { Footer } from "@/components/footer";
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
      <main className="mx-auto w-full max-w-[1200px] px-4 py-10 sm:px-6 lg:px-8">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbJsonLd) }}
        />
        <div className="border-b border-[var(--color-line)] pb-6">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
            Storefront
          </p>
          <h1 className="mt-2 font-display text-3xl tracking-tight text-[var(--color-ink)] sm:text-4xl">
            Collections
          </h1>
        </div>

        {categories.length === 0 ? (
          <section className="rounded-sm border border-[var(--color-line)] bg-[var(--background)] px-6 py-10 text-center text-[var(--color-muted)]">
            Collections are not available right now.
          </section>
        ) : (
          <section className="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {categories.map((category) => (
              <Link
                key={category.categoryId}
                href={`/collections/${encodeURIComponent(slugifyCategoryName(category.name))}`}
                className="rounded-sm border border-[var(--color-line)] bg-white px-5 py-4 transition hover:border-[var(--color-accent-gold)] hover:shadow-[0_8px_24px_rgba(26,24,20,0.08)]"
              >
                <p className="font-display text-xl tracking-tight text-[var(--color-ink)]">
                  {category.name}
                </p>
              </Link>
            ))}
          </section>
        )}
      </main>
      <Footer />
    </div>
  );
}
