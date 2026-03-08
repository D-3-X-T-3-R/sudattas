import { NextResponse } from "next/server";
import type { Product } from "@/lib/schemas";
import { fetchProductById, fetchCategories, fetchSizes } from "@/lib/admin-queries";
import type { ProductListRowWithVariantStock } from "@/lib/graphql-types";
import {
  fetchProductByIdWithVariantStock,
  fetchCategoriesWithSession,
  fetchSizesWithSession,
} from "@/lib/storefront-queries";

function mapToStorefrontProduct(
  row: ProductListRowWithVariantStock,
  categoryNameById: Record<string, string>
): Product {
  const pricePaise = row.amountPaise ? parseInt(row.amountPaise, 10) : 0;
  const priceRupees = Math.round((pricePaise / 100) * 100) / 100;
  const priceFormatted = row.formatted?.trim() || undefined;
  const imageList = row.images?.filter(
    (i) => i.url || i.thumbnailUrl
  ) as { url?: string | null; thumbnailUrl?: string | null }[] | undefined;
  const allUrls =
    imageList?.map((i) => i.url || i.thumbnailUrl || "").filter(Boolean) ?? [];
  const imageUrl = allUrls[0] ?? "";
  const hoverUrl = allUrls[1] ?? imageUrl;

  return {
    id: row.productId,
    name: row.name,
    collection:
      (row.categoryId && categoryNameById[row.categoryId]) || "Collection",
    price: priceRupees,
    priceFormatted,
    rating: 4.5,
    reviews: 0,
    fabric: row.fabric ?? "",
    occasion: row.occasion ?? "",
    description: row.description ?? "",
    image: imageUrl,
    hoverImage: hoverUrl || undefined,
    images: allUrls.length > 0 ? allUrls : undefined,
    imageAlt: row.name,
    variantStock: row.variantStock ?? undefined,
  };
}

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params;
  const sessionId = request.headers.get("x-session-id")?.trim() || null;

  try {
    let row: ProductListRowWithVariantStock | null;
    let categories: { categoryId: string; name: string }[];
    let sizes: { sizeId: string; sizeName: string }[];

    if (sessionId) {
      [row, categories, sizes] = await Promise.all([
        fetchProductByIdWithVariantStock(sessionId, id),
        fetchCategoriesWithSession(sessionId),
        fetchSizesWithSession(sessionId),
      ]);
    } else {
      [row, categories, sizes] = await Promise.all([
        fetchProductById(id),
        fetchCategories(),
        fetchSizes(),
      ]);
    }

    if (!row) {
      return NextResponse.json({ product: null, sizes: [], error: "Not found" }, { status: 404 });
    }

    const categoryNameById: Record<string, string> = {};
    for (const c of categories) {
      categoryNameById[c.categoryId] = c.name;
    }

    const product = mapToStorefrontProduct(row, categoryNameById);
    return NextResponse.json({ product, sizes, error: null });
  } catch (e) {
    const message =
      e instanceof Error ? e.message : "Failed to load product";
    return NextResponse.json(
      { product: null, sizes: [], error: message },
      { status: 500 }
    );
  }
}
