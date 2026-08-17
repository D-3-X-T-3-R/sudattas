/**
 * Shared GraphQL response types used by both admin and storefront.
 */

export interface ProductImageListItem {
  imageId?: string | null;
  thumbnailUrl?: string | null;
  url?: string | null;
}

export interface CategoryRow {
  categoryId: string;
  name: string;
}

export interface OccasionRow {
  occasionId: string;
  occasionName: string;
}

export interface ProductVariantStockRow {
  variantId?: string;
  sizeId: string;
  sizeName: string;
  quantity: number;
}

/** Product list row (searchProduct); used by admin list and storefront products API mapping. */
export interface ProductListRow {
  productId: string;
  name: string;
  description?: string | null;
  amountPaise?: string;
  formatted: string;
  stockQuantity?: string | null;
  categoryId?: string | null;
  sku?: string | null;
  slug?: string | null;
  fabric?: string | null;
  weave?: string | null;
  occasion?: string | null;
  hasBlousePiece?: boolean | null;
  careInstructions?: string | null;
  productStatusId?: string | null;
  images?: ProductImageListItem[] | null;
}

/** Product list row plus variant stock (for single-product fetch). */
export interface ProductListRowWithVariantStock extends ProductListRow {
  variantStock?: ProductVariantStockRow[] | null;
}

/** Server-computed star rating aggregate for a product (backend's `productRatingSummary` query). */
export interface ProductRatingSummary {
  /** CEIL(AVG(Rating)) — a raw average of 3.2 or 3.8 both come back as 4; 0 when unrated. */
  average: number;
  /** Total number of ratings included in the average. */
  count: number;
}
