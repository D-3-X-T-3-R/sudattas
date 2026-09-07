import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { StorefrontCollectionPageContent } from "@/components/storefront-collection-page-content";
import { siteUrl } from "@/lib/site-url";
import { loadCollectionByCategorySlug } from "@/lib/storefront-collection-page";
import { safeJsonLd } from "@/lib/json-ld";

export const dynamic = "force-dynamic";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const data = await loadCollectionByCategorySlug(slug).catch(() => null);
  if (!data) {
    return {
      title: "Collection Not Found | Sudatta's",
      description: "The requested collection is unavailable.",
    };
  }
  return {
    title: `${data.categoryName} Collection | Sudatta's`,
    description: `Shop the ${data.categoryName} collection at Sudatta's.`,
    alternates: {
      canonical: `${siteUrl()}/collections/${encodeURIComponent(data.categorySlug)}`,
    },
    openGraph: {
      title: `${data.categoryName} Collection | Sudatta's`,
      description: `Shop the ${data.categoryName} collection at Sudatta's.`,
      type: "website",
      url: `${siteUrl()}/collections/${encodeURIComponent(data.categorySlug)}`,
    },
  };
}

export default async function CollectionSlugPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const data = await loadCollectionByCategorySlug(slug).catch(() => null);
  if (!data) notFound();

  const base = siteUrl();
  const collectionUrl = `${base}/collections/${encodeURIComponent(data.categorySlug)}`;
  const breadcrumbJsonLd = {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: [
      { "@type": "ListItem", position: 1, name: "Home", item: `${base}/` },
      { "@type": "ListItem", position: 2, name: "Collections", item: `${base}/collections` },
      { "@type": "ListItem", position: 3, name: data.categoryName, item: collectionUrl },
    ],
  };
  const collectionJsonLd = {
    "@context": "https://schema.org",
    "@type": "CollectionPage",
    name: `${data.categoryName} Collection`,
    url: collectionUrl,
    mainEntity: {
      "@type": "ItemList",
      itemListElement: data.products.map((product, index) => ({
        "@type": "ListItem",
        position: index + 1,
        url: `${base}/product/${encodeURIComponent(product.id)}`,
        name: product.name,
      })),
    },
  };

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: safeJsonLd(breadcrumbJsonLd) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: safeJsonLd(collectionJsonLd) }}
      />
      <StorefrontCollectionPageContent data={data} />
    </>
  );
}
