import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { StorefrontCollectionPageContent } from "@/components/storefront-collection-page-content";
import { siteUrl } from "@/lib/site-url";
import { loadCollectionByCategorySlug } from "@/lib/storefront-collection-page";

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
  return <StorefrontCollectionPageContent data={data} />;
}
