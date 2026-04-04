import { NextResponse } from "next/server";
import type { Product } from "@/lib/schemas";
import type { ProductListRowWithVariantStock } from "@/lib/graphql-types";
import { parsePaise, paiseToRupeesNumber } from "@/lib/money";
import {
  fetchProductByIdWithVariantStock,
  fetchCategoriesWithSession,
  fetchSizesWithSession,
} from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { forwardedIpHeadersFromRequest } from "@/lib/forwarded-ip";

function mapToStorefrontProduct(
  row: ProductListRowWithVariantStock,
  categoryNameById: Record<string, string>
): Product {
  const pricePaise = parsePaise(row.amountPaise);
  const priceRupees = paiseToRupeesNumber(pricePaise);
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
    pricePaise,
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
  const forwardedHeaders = forwardedIpHeadersFromRequest(request);
  const headerSessionId = request.headers.get("x-session-id")?.trim() || null;
  let sessionId = headerSessionId;
  if (!sessionId) {
    sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  }
  /** When we recover from a stale browser session, tell the client to replace localStorage. */
  let refreshedGuestSessionId: string | null = null;

  try {
    const recovered = await withRecoveredGuestSession(
      sessionId,
      forwardedHeaders,
      async (activeSessionId) => {
        const [resolvedRow, resolvedCategories, resolvedSizes] = await Promise.all([
          fetchProductByIdWithVariantStock(activeSessionId, id, forwardedHeaders),
          fetchCategoriesWithSession(activeSessionId, forwardedHeaders),
          fetchSizesWithSession(activeSessionId, forwardedHeaders),
        ]);
        return {
          resolvedRow,
          resolvedCategories,
          resolvedSizes,
        };
      }
    );
    refreshedGuestSessionId = recovered.refreshedSessionId;
    const row = recovered.value.resolvedRow;
    const categories = recovered.value.resolvedCategories;
    const sizes = recovered.value.resolvedSizes;

    if (!row) {
      return NextResponse.json({ product: null, sizes: [], error: "Not found" }, { status: 404 });
    }

    const categoryNameById: Record<string, string> = {};
    for (const c of categories) {
      categoryNameById[c.categoryId] = c.name;
    }

    const product = mapToStorefrontProduct(row, categoryNameById);
    const res = NextResponse.json({ product, sizes, error: null });
    if (refreshedGuestSessionId) {
      res.headers.set("X-Set-Guest-Session", refreshedGuestSessionId);
    }
    return res;
  } catch (e) {
    const message =
      e instanceof Error ? e.message : "Failed to load product";
    return NextResponse.json(
      { product: null, sizes: [], error: message },
      { status: 500 }
    );
  }
}
