import { NextResponse } from "next/server";
import type { Product } from "@/lib/schemas";
import { fetchProductsList, fetchCategories } from "@/lib/admin-queries";
import type { ProductListRow } from "@/lib/graphql-types";
import {
  fetchProductsListWithSession,
  fetchCategoriesWithSession,
} from "@/lib/storefront-queries";

/** Map backend product list row + category names to storefront Product shape. */
function mapToStorefrontProduct(
  row: ProductListRow,
  categoryNameById: Record<string, string>
): Product {
  const pricePaise = row.amountPaise ? parseInt(row.amountPaise, 10) : 0;
  const priceRupees = Math.round((pricePaise / 100) * 100) / 100;
  const priceFormatted = row.formatted?.trim() || undefined;
  const imageList = row.images?.filter(
    (i) => i.url || i.thumbnailUrl
  ) as { url?: string | null; thumbnailUrl?: string | null }[] | undefined;
  const imageUrl =
    imageList?.[0]?.url || imageList?.[0]?.thumbnailUrl || "";
  const hoverUrl =
    imageList?.[1]?.url || imageList?.[1]?.thumbnailUrl || imageUrl;

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
    imageAlt: row.name,
  };
}

export async function GET(request: Request) {
  const sessionId = request.headers.get("x-session-id")?.trim() || null;

  try {
    let productsList: ProductListRow[];
    let categories: { categoryId: string; name: string }[];

    if (sessionId) {
      // Session only; no admin fallback when session fails
      [productsList, categories] = await Promise.all([
        fetchProductsListWithSession(sessionId, { limit: "200" }),
        fetchCategoriesWithSession(sessionId),
      ]);
    } else {
      [productsList, categories] = await Promise.all([
        fetchProductsList({ limit: "200" }),
        fetchCategories(),
      ]);
    }

    const categoryNameById: Record<string, string> = {};
    for (const c of categories) {
      categoryNameById[c.categoryId] = c.name;
    }

    const products: Product[] = productsList.map((row) =>
      mapToStorefrontProduct(row, categoryNameById)
    );

    return NextResponse.json({ products, error: null });
  } catch (e) {
    const message =
      e instanceof Error ? e.message : "Failed to load products";
    const isAuthMissing =
      typeof message === "string" &&
      (message.includes("Unauthorized") || message.includes("NEXT_PUBLIC_ADMIN_API_KEY"));
    if (isAuthMissing) {
      console.warn(
        "API products: no valid session and no admin key. Set NEXT_PUBLIC_ADMIN_API_KEY in .env.local to match backend ADMIN_API_KEY (or fix session)."
      );
    } else {
      console.error("API products:", e);
    }
    return NextResponse.json(
      { products: [], error: message },
      { status: 200 }
    );
  }
}
