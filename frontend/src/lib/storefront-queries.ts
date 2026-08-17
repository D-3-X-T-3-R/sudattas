/**
 * Storefront GraphQL queries. Use guest session only (no admin key).
 * Used by API routes that serve the public storefront (e.g. /api/products, /api/storefront-filters).
 */

import { gqlWithSession } from "./graphql-client";
import type {
  CategoryRow,
  OccasionRow,
  ProductListRow,
  ProductListRowWithVariantStock,
  ProductRatingSummary,
} from "./graphql-types";

const CATEGORIES_QUERY = `query Categories { searchCategory(search: {}) { categoryId name } }`;

const OCCASIONS_MUTATION = `mutation Occasions { searchOccasion(input: { occasionId: "0" }) { occasionId occasionName } }`;
/** Moods tied to the newest products (see backend ShopHighlightMoods). */
const SHOP_HIGHLIGHT_MOODS_QUERY = `query ShopHighlightMoods($recentProductLimit: Int, $maxMoods: Int) {
  shopHighlightMoods(recentProductLimit: $recentProductLimit, maxMoods: $maxMoods) {
    moodId moodName
  }
}`;

const PRODUCTS_QUERY = `query SearchProductsList($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    fabric weave occasion hasBlousePiece
    images { thumbnailUrl url }
    variantStock { variantId sizeId sizeName quantity }
  }
}`;

const PRODUCT_BY_ID_QUERY = `query ProductById($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    fabric occasion
    images { thumbnailUrl url }
    variantStock { variantId sizeId sizeName quantity }
  }
}`;

const SIZES_QUERY = `mutation Sizes { searchSize(input: { sizeId: "0" }) { sizeId sizeName } }`;

export type { CategoryRow, OccasionRow, ProductListRow, ProductListRowWithVariantStock };

export interface SizeRow {
  sizeId: string;
  sizeName: string;
}

export interface StorefrontMoodRow {
  moodId: string;
  moodName: string;
}

/** Fetch all sizes (for storefront size selector). */
export async function fetchSizesWithSession(
  sessionId: string,
  extraHeaders: Record<string, string> = {}
): Promise<SizeRow[]> {
  const data = await gqlWithSession<{ searchSize?: SizeRow[] }>(
    sessionId,
    SIZES_QUERY,
    {},
    extraHeaders
  );
  return data?.searchSize ?? [];
}

export async function fetchCategoriesWithSession(
  sessionId: string,
  extraHeaders: Record<string, string> = {}
): Promise<CategoryRow[]> {
  const data = await gqlWithSession<{ searchCategory?: CategoryRow[] }>(
    sessionId,
    CATEGORIES_QUERY,
    {},
    extraHeaders
  );
  return data?.searchCategory ?? [];
}

export async function fetchOccasionsWithSession(
  sessionId: string,
  extraHeaders: Record<string, string> = {}
): Promise<OccasionRow[]> {
  const data = await gqlWithSession<{ searchOccasion?: OccasionRow[] }>(
    sessionId,
    OCCASIONS_MUTATION,
    {},
    extraHeaders
  );
  return data?.searchOccasion ?? [];
}

export async function fetchProductsListWithSession(
  sessionId: string,
  params: { limit?: string; moodId?: string; categoryId?: string } = {},
  extraHeaders: Record<string, string> = {}
): Promise<ProductListRowWithVariantStock[]> {
  const search: { limit?: string; moodId?: string; categoryId?: string } = params.limit
    ? { limit: params.limit }
    : { limit: "200" };
  if (params.moodId) search.moodId = params.moodId;
  if (params.categoryId) search.categoryId = params.categoryId;
  const data = await gqlWithSession<{ searchProduct?: ProductListRowWithVariantStock[] }>(
    sessionId,
    PRODUCTS_QUERY,
    { search },
    extraHeaders
  );
  return data?.searchProduct ?? [];
}

/** All moods (for fallback when shop highlight is empty). */
const MOODS_ALL_QUERY = `query MoodsAll($input: SearchProductMoodInput!) {
  searchProductMood(input: $input) { moodId moodName }
}`;

export async function fetchProductMoodsWithSession(
  sessionId: string,
  extraHeaders: Record<string, string> = {}
): Promise<StorefrontMoodRow[]> {
  const data = await gqlWithSession<{ searchProductMood?: StorefrontMoodRow[] }>(
    sessionId,
    MOODS_ALL_QUERY,
    { input: {} },
    extraHeaders
  );
  return data?.searchProductMood ?? [];
}

/** Up to `maxMoods` distinct moods from the newest `recentProductLimit` products. */
export async function fetchShopHighlightMoodsWithSession(
  sessionId: string,
  opts: { recentProductLimit?: number; maxMoods?: number } = {},
  extraHeaders: Record<string, string> = {}
): Promise<StorefrontMoodRow[]> {
  const data = await gqlWithSession<{ shopHighlightMoods?: StorefrontMoodRow[] }>(
    sessionId,
    SHOP_HIGHLIGHT_MOODS_QUERY,
    {
      recentProductLimit: opts.recentProductLimit ?? 100,
      maxMoods: opts.maxMoods ?? 12,
    },
    extraHeaders
  );
  return data?.shopHighlightMoods ?? [];
}

/** Fetch a single product by id with variant stock (for detail page size selector). */
export async function fetchProductByIdWithVariantStock(
  sessionId: string,
  productId: string,
  extraHeaders: Record<string, string> = {}
): Promise<ProductListRowWithVariantStock | null> {
  const data = await gqlWithSession<{
    searchProduct?: ProductListRowWithVariantStock[];
  }>(
    sessionId,
    PRODUCT_BY_ID_QUERY,
    {
      search: { productId, limit: "1" },
    },
    extraHeaders
  );
  const list = data?.searchProduct ?? [];
  return list[0] ?? null;
}

/**
 * Ratings are star-only (1-5), no written review text at this time. The average is computed
 * server-side (real SQL AVG/COUNT via the `productRatingSummary` query) and pre-rounded up to a
 * whole star: a raw average of 3.2 or 3.8 both come back as 4, per product decision. It includes
 * ratings of every moderation status since there is currently no admin review-moderation UI to
 * ever approve them.
 */
const PRODUCT_RATING_SUMMARY_QUERY = `query ProductRatingSummary($productId: String!) {
  productRatingSummary(productId: $productId) { averageRating ratingCount }
}`;

export async function fetchProductRatingSummaryWithSession(
  sessionId: string,
  productId: string,
  extraHeaders: Record<string, string> = {}
): Promise<ProductRatingSummary> {
  const data = await gqlWithSession<{
    productRatingSummary?: { averageRating: number; ratingCount: number };
  }>(sessionId, PRODUCT_RATING_SUMMARY_QUERY, { productId }, extraHeaders);
  const summary = data?.productRatingSummary;
  return { average: summary?.averageRating ?? 0, count: summary?.ratingCount ?? 0 };
}
