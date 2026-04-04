import { NextResponse } from "next/server";
import type { Product } from "@/lib/schemas";
import type { ProductListRowWithVariantStock } from "@/lib/graphql-types";
import { parsePaise, paiseToRupeesNumber } from "@/lib/money";
import {
  fetchProductsListWithSession,
  fetchCategoriesWithSession,
} from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { forwardedIpHeadersFromRequest } from "@/lib/forwarded-ip";

type ProductsApiPayload = { products: Product[]; error: string | null };
type CacheEntry = { ts: number; payload: ProductsApiPayload };

const CATALOG_CACHE_HEADERS = { "Cache-Control": "public, s-maxage=60, stale-while-revalidate=30" };

const CACHE_TTL_MS = 60_000; // 60 s — products don't change per-request
const cache = new Map<string, CacheEntry>();
const inflight = new Map<string, Promise<ProductsApiPayload>>();

/** Map backend product list row + category names to storefront Product shape. */
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

  const variantStock = row.variantStock?.map((v) => ({
    variantId: v.variantId,
    sizeId: v.sizeId,
    sizeName: v.sizeName,
    quantity:
      typeof v.quantity === "number"
        ? v.quantity
        : parseInt(String(v.quantity ?? "0"), 10) || 0,
  }));

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
    ...(variantStock && variantStock.length > 0 ? { variantStock } : {}),
  };
}

export async function GET(request: Request) {
  const forwardedHeaders = forwardedIpHeadersFromRequest(request);
  const headerSessionId = request.headers.get("x-session-id")?.trim() || null;
  let sessionId = headerSessionId;
  if (!sessionId) {
    sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  }
  const moodId =
    new URL(request.url).searchParams.get("moodId")?.trim() || undefined;
  // Catalog data is identical for every session — key on request shape only,
  // not on sessionId. This makes the cache effective across all concurrent users.
  const cacheKey = `${moodId ?? "all"}::200`;

  const now = Date.now();
  const cached = cache.get(cacheKey);
  if (cached && now - cached.ts < CACHE_TTL_MS) {
    return NextResponse.json(cached.payload, { headers: CATALOG_CACHE_HEADERS });
  }

  const pending = inflight.get(cacheKey);
  if (pending) {
    const payload = await pending;
    return NextResponse.json(payload, { headers: payload.error ? {} : CATALOG_CACHE_HEADERS });
  }

  const loader = (async (): Promise<ProductsApiPayload> => {
    const listParams = { limit: "200" as const, ...(moodId ? { moodId } : {}) };

    const recovered = await withRecoveredGuestSession(
      sessionId,
      forwardedHeaders,
      async (activeSessionId) => {
        const [list, cats] = await Promise.all([
          fetchProductsListWithSession(activeSessionId, listParams, forwardedHeaders),
          fetchCategoriesWithSession(activeSessionId, forwardedHeaders),
        ]);
        return { list, cats };
      }
    );
    const productsList = recovered.value.list;
    const categories = recovered.value.cats;

    const categoryNameById: Record<string, string> = {};
    for (const c of categories) {
      categoryNameById[c.categoryId] = c.name;
    }

    const products: Product[] = productsList.map((row) =>
      mapToStorefrontProduct(row, categoryNameById)
    );

    const payload: ProductsApiPayload = { products, error: null };
    cache.set(cacheKey, { ts: Date.now(), payload });
    return payload;
  })()
    .catch((e): ProductsApiPayload => {
      const message =
        e instanceof Error ? e.message : "Failed to load products";
      const isAuthMissing =
        typeof message === "string" &&
        (message.includes("Unauthorized") || message.includes("Guest session unavailable"));
      if (isAuthMissing) {
        console.warn("API products: no valid guest session available.");
      } else {
        console.error("API products:", e);
      }
      return { products: [], error: message };
    })
    .finally(() => {
      inflight.delete(cacheKey);
    });

  inflight.set(cacheKey, loader);
  const payload = await loader;
  if (payload.error == null) {
    return NextResponse.json(payload, { headers: CATALOG_CACHE_HEADERS });
  }
  return NextResponse.json(payload, { status: 200 });
}
