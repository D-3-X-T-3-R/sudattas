/**
 * Shared GraphQL response types used by both admin and storefront.
 */

export interface ProductImageListItem {
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
