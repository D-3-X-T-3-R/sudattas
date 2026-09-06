import { notFound, permanentRedirect } from "next/navigation";
import { loadCollectionByCategoryIdRoute } from "@/lib/storefront-collection-page";

export const dynamic = "force-dynamic";

// /category/{id} and /collections/{slug} used to be two independently-crawlable live copies
// of the same page (search engines were indexing both) — this route now only exists to send
// the (still-linked-from-the-wild) old id-based URL on to its canonical slug-based address
// with a 308, consolidating SEO signal there. No internal navigation links to /category/
// anymore; this is purely for old bookmarks/backlinks/search results.
export default async function CategoryPage({
  params,
}: {
  params: Promise<{ categoryId: string }>;
}) {
  const { categoryId } = await params;
  const data = await loadCollectionByCategoryIdRoute(categoryId).catch(() => null);
  if (!data) notFound();
  permanentRedirect(`/collections/${encodeURIComponent(data.categorySlug)}`);
}
