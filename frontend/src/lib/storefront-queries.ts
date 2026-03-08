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
} from "./graphql-types";

const CATEGORIES_QUERY = `query Categories { searchCategory(search: {}) { categoryId name } }`;

const OCCASIONS_MUTATION = `mutation Occasions { searchOccasion(input: { occasionId: "0" }) { occasionId occasionName } }`;

const PRODUCTS_QUERY = `query SearchProductsList($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId
    images { thumbnailUrl url }
  }
}`;

const PRODUCT_BY_ID_QUERY = `query ProductById($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId
    images { thumbnailUrl url }
    variantStock { sizeId sizeName quantity }
  }
}`;

const SIZES_QUERY = `mutation Sizes { searchSize(input: { sizeId: "0" }) { sizeId sizeName } }`;

export type { CategoryRow, OccasionRow, ProductListRow, ProductListRowWithVariantStock };

export interface SizeRow {
  sizeId: string;
  sizeName: string;
}

/** Fetch all sizes (for storefront size selector). */
export async function fetchSizesWithSession(sessionId: string): Promise<SizeRow[]> {
  const data = await gqlWithSession<{ searchSize?: SizeRow[] }>(
    sessionId,
    SIZES_QUERY
  );
  return data?.searchSize ?? [];
}

export async function fetchCategoriesWithSession(sessionId: string): Promise<CategoryRow[]> {
  const data = await gqlWithSession<{ searchCategory?: CategoryRow[] }>(
    sessionId,
    CATEGORIES_QUERY
  );
  return data?.searchCategory ?? [];
}

export async function fetchOccasionsWithSession(sessionId: string): Promise<OccasionRow[]> {
  const data = await gqlWithSession<{ searchOccasion?: OccasionRow[] }>(
    sessionId,
    OCCASIONS_MUTATION
  );
  return data?.searchOccasion ?? [];
}

export async function fetchProductsListWithSession(
  sessionId: string,
  params: { limit?: string } = {}
): Promise<ProductListRow[]> {
  const search = params.limit ? { limit: params.limit } : { limit: "200" };
  const data = await gqlWithSession<{ searchProduct?: ProductListRow[] }>(
    sessionId,
    PRODUCTS_QUERY,
    { search }
  );
  return data?.searchProduct ?? [];
}

/** Fetch a single product by id with variant stock (for detail page size selector). */
export async function fetchProductByIdWithVariantStock(
  sessionId: string,
  productId: string
): Promise<ProductListRowWithVariantStock | null> {
  const data = await gqlWithSession<{
    searchProduct?: ProductListRowWithVariantStock[];
  }>(sessionId, PRODUCT_BY_ID_QUERY, {
    search: { productId, limit: "1" },
  });
  const list = data?.searchProduct ?? [];
  return list[0] ?? null;
}
