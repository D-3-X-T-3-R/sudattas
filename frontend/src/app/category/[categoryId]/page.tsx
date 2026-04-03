import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { StorefrontCollectionPageContent } from "@/components/storefront-collection-page-content";
import { siteUrl } from "@/lib/site-url";
import { loadCollectionByCategoryIdRoute } from "@/lib/storefront-collection-page";

export const dynamic = "force-dynamic";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ categoryId: string }>;
}): Promise<Metadata> {
  const { categoryId } = await params;
  const data = await loadCollectionByCategoryIdRoute(categoryId).catch(() => null);
  if (!data) {
    return {
      title: "Category Not Found | Sudatta's",
      description: "The requested category is unavailable.",
    };
  }
  return {
    title: `${data.categoryName} Category | Sudatta's`,
    description: `Browse ${data.categoryName} products at Sudatta's.`,
    alternates: {
      canonical: `${siteUrl()}/collections/${encodeURIComponent(data.categorySlug)}`,
    },
    openGraph: {
      title: `${data.categoryName} Category | Sudatta's`,
      description: `Browse ${data.categoryName} products at Sudatta's.`,
      type: "website",
      url: `${siteUrl()}/category/${encodeURIComponent(data.categoryId)}`,
    },
  };
}

export default async function CategoryPage({
  params,
}: {
  params: Promise<{ categoryId: string }>;
}) {
  const { categoryId } = await params;
  const data = await loadCollectionByCategoryIdRoute(categoryId).catch(() => null);
  if (!data) notFound();
  return <StorefrontCollectionPageContent data={data} />;
}
