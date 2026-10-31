/**
 * Admin product-catalog GraphQL queries (categories, products, sizes, colors, fabrics, weaves,
 * occasions, moods, variants, inventory). Split out of admin-queries.ts to keep that file under
 * the max-lines limit — re-exported from there via `export * from "./admin-product-queries"`, so
 * existing `import { fetchProductsList } from "@/lib/admin-queries"` call sites are unaffected.
 */

import { gqlAdmin } from "./graphql-client";
import { normalizeLimit } from "./admin-query-utils";
import type {
  CategoryRow,
  OccasionRow,
  ProductListRow,
  ProductListRowWithVariantStock,
  ProductImageListItem,
} from "./graphql-types";

export type { CategoryRow, OccasionRow, ProductListRow, ProductListRowWithVariantStock, ProductImageListItem };

const CATEGORIES_QUERY = `query Categories { searchCategory(search: {}) { categoryId name exchangeEligible } }`;
const PRODUCTS_QUERY = `query SearchProductsList($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId metaTitle metaDescription
    images { imageId thumbnailUrl url }
    variantStock { variantId sizeId sizeName quantity }
  }
}`;

const PRODUCT_BY_ID_QUERY = `query ProductById($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId metaTitle metaDescription
    images { imageId thumbnailUrl url }
    variantStock { variantId sizeId sizeName quantity }
  }
}`;

/** Fetch all categories for admin dropdowns. */
export async function fetchCategories(): Promise<CategoryRow[]> {
  const data = await gqlAdmin<{ searchCategory?: CategoryRow[] }>(CATEGORIES_QUERY);
  return data?.searchCategory ?? [];
}

/** Fetch products for admin list with optional filters. */
export async function fetchProductsList(params: {
  name?: string;
  categoryId?: string;
  fabric?: string;
  weave?: string;
  occasion?: string;
  limit?: string;
  productStatusId?: string;
  startingPricePaise?: string;
  endingPricePaise?: string;
  moodId?: string;
}): Promise<ProductListRowWithVariantStock[]> {
  const search: Record<string, string> = {};
  if (params.name) search.name = params.name;
  if (params.categoryId) search.categoryId = params.categoryId;
  if (params.fabric) search.fabric = params.fabric;
  if (params.weave) search.weave = params.weave;
  if (params.occasion) search.occasion = params.occasion;
  if (params.limit) search.limit = normalizeLimit(params.limit);
  if (params.productStatusId) search.productStatusId = params.productStatusId;
  if (params.startingPricePaise) search.startingPricePaise = params.startingPricePaise;
  if (params.endingPricePaise) search.endingPricePaise = params.endingPricePaise;
  if (params.moodId) search.moodId = params.moodId;

  const variables = { search: Object.keys(search).length ? search : { limit: "50" } };
  const data = await gqlAdmin<{ searchProduct?: ProductListRowWithVariantStock[] }>(
    PRODUCTS_QUERY,
    variables
  );
  const raw = data?.searchProduct ?? [];
  return raw.map(normalizeProductImages);
}

/** Ensure each product's images have url/imageId/thumbnailUrl in a consistent shape (camelCase). */
function normalizeProductImages(p: ProductListRowWithVariantStock): ProductListRowWithVariantStock {
  const rawImages = p.images;
  if (!rawImages?.length) return p;
  const images: ProductImageListItem[] = rawImages.map((img: unknown) => {
    const o = img as Record<string, unknown>;
    const url =
      (o.url as string) ?? (o.thumbnailUrl as string) ?? (o.thumbnail_url as string) ?? "";
    const imageId = (o.imageId as string) ?? (o.image_id as string) ?? "";
    const thumbnailUrl = (o.thumbnailUrl as string) ?? (o.thumbnail_url as string) ?? url;
    return { imageId: imageId || undefined, thumbnailUrl: thumbnailUrl || null, url: url || null };
  });
  return { ...p, images };
}

/** Fetch a single product by id with variant stock (for storefront detail when no session). */
export async function fetchProductById(
  productId: string
): Promise<ProductListRowWithVariantStock | null> {
  const data = await gqlAdmin<{ searchProduct?: ProductListRowWithVariantStock[] }>(
    PRODUCT_BY_ID_QUERY,
    { search: { productId, limit: "1" } }
  );
  const list = data?.searchProduct ?? [];
  const first = list[0];
  return first ? normalizeProductImages(first) as ProductListRowWithVariantStock : null;
}

/** Delete a product by ID. Returns remaining products (backend returns deleted product in list). */
export async function deleteProduct(productId: string): Promise<void> {
  await gqlAdmin<{ deleteProduct?: unknown[] }>(
    `mutation DeleteProduct($productId: String!) {
      deleteProduct(productId: $productId) { productId }
    }`,
    { productId }
  );
}

/** Soft delete: sets the product's status to archived server-side (resolved there, not a
 * hardcoded id here) — hides it from the store but keeps everything else intact, unlike
 * permanentlyDeleteProduct. Reversible via activateProduct below. */
export async function archiveProduct(productId: string): Promise<void> {
  await gqlAdmin<{ archiveProduct?: unknown[] }>(
    `mutation ArchiveProduct($productId: String!) {
      archiveProduct(productId: $productId) { productId }
    }`,
    { productId }
  );
}

/** Reverses archiveProduct: sets the product's status back to active server-side (resolved
 * there, not a hardcoded id here). Touches nothing else on the product. */
export async function activateProduct(productId: string): Promise<void> {
  await gqlAdmin<{ activateProduct?: unknown[] }>(
    `mutation ActivateProduct($productId: String!) {
      activateProduct(productId: $productId) { productId }
    }`,
    { productId }
  );
}

export interface PermanentlyDeleteProductResult {
  productId: string;
  name: string;
  variantsDeleted: string;
  imagesDeleted: string;
  /** Images whose R2 object couldn't be purged — the product is fully gone from the DB
   * regardless; this is a storage-hygiene signal, not a failure indicator. */
  imagesPurgeFailed: string;
}

/**
 * Irreversible hard delete — cascades variants/inventory/cart/reviews/wishlist/images and
 * purges images from R2. The backend refuses outright (FailedPrecondition) if the product has
 * real order history; use Archive for that case instead.
 */
export async function permanentlyDeleteProduct(
  productId: string
): Promise<PermanentlyDeleteProductResult> {
  const data = await gqlAdmin<{ permanentlyDeleteProduct?: PermanentlyDeleteProductResult }>(
    `mutation PermanentlyDeleteProduct($productId: String!) {
      permanentlyDeleteProduct(productId: $productId) {
        productId
        name
        variantsDeleted
        imagesDeleted
        imagesPurgeFailed
      }
    }`,
    { productId }
  );
  if (!data?.permanentlyDeleteProduct) {
    throw new Error("permanentlyDeleteProduct returned empty payload");
  }
  return data.permanentlyDeleteProduct;
}

/** Search product images (one row per image). Filter by productId to load images for a product. */
export async function searchProductImage(params: {
  productId?: string;
}): Promise<{ imageId: string; productId: string; url?: string | null }[]> {
  const data = await gqlAdmin<{
    searchProductImage?: Array<{ imageId: string; productId: string; url?: string | null }>;
  }>(
    `query SearchProductImage($search: SearchProductImage!) {
      searchProductImage(search: $search) { imageId productId url }
    }`,
    { search: { productId: params.productId ?? undefined } }
  );
  return data?.searchProductImage ?? [];
}

/** Delete a product image by id. */
export async function deleteProductImage(imageId: string): Promise<void> {
  await gqlAdmin<{ deleteProductImage?: unknown[] }>(
    `mutation DeleteProductImage($imageId: String!) {
      deleteProductImage(imageId: $imageId) { imageId productId }
    }`,
    { imageId }
  );
}

/** Re-parent an existing image to a different product — the only thing update_product_image
 * actually does server-side (display_order/url are carried over unchanged; there's no alt-text
 * column on ProductImages to edit despite the mutation accepting one). image_base64 is a required
 * field on the input the handler never reads; sent empty, same pattern as reviews' unused
 * `comment` field elsewhere in this codebase. */
export async function moveProductImageToProduct(
  imageId: string,
  targetProductId: string
): Promise<void> {
  await gqlAdmin<{ updateProductImage?: unknown[] }>(
    `mutation MoveProductImage($productImage: ProductImageMutation!) {
      updateProductImage(productImage: $productImage) { imageId productId }
    }`,
    { productImage: { imageId, productId: targetProductId, imageBase64: "" } }
  );
}

/** Size for variant dropdowns (id "0" returns all) */
export interface SizeRow {
  sizeId: string;
  sizeName: string;
}

export async function fetchSizes(): Promise<SizeRow[]> {
  const data = await gqlAdmin<{ searchSize?: SizeRow[] }>(
    `mutation Sizes { searchSize(input: { sizeId: "0" }) { sizeId sizeName } }`
  );
  return data?.searchSize ?? [];
}

export async function createSize(sizeName: string): Promise<SizeRow | null> {
  const data = await gqlAdmin<{ createSize?: SizeRow[] }>(
    `mutation CreateSize($input: NewSize!) { createSize(input: $input) { sizeId sizeName } }`,
    { input: { sizeName: sizeName.trim() } }
  );
  return data?.createSize?.[0] ?? null;
}

export async function updateSize(sizeId: string, sizeName: string): Promise<SizeRow | null> {
  const data = await gqlAdmin<{ updateSize?: SizeRow[] }>(
    `mutation UpdateSize($input: SizeMutation!) { updateSize(input: $input) { sizeId sizeName } }`,
    { input: { sizeId, sizeName: sizeName.trim() } }
  );
  return data?.updateSize?.[0] ?? null;
}

export async function deleteSize(sizeId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteSize($input: DeleteSizeInput!) { deleteSize(input: $input) { sizeId } }`,
    { input: { sizeId } }
  );
}

/** Color for variant dropdowns (id "0" returns all) */
export interface ColorRow {
  colorId: string;
  colorName: string;
}

export async function fetchColors(): Promise<ColorRow[]> {
  const data = await gqlAdmin<{ searchColor?: ColorRow[] }>(
    `mutation Colors { searchColor(input: { colorId: "0" }) { colorId colorName } }`
  );
  return data?.searchColor ?? [];
}

export async function createColor(colorName: string): Promise<ColorRow | null> {
  const data = await gqlAdmin<{ createColor?: ColorRow[] }>(
    `mutation CreateColor($input: NewColor!) { createColor(input: $input) { colorId colorName } }`,
    { input: { colorName: colorName.trim() } }
  );
  return data?.createColor?.[0] ?? null;
}

export async function updateColor(colorId: string, colorName: string): Promise<ColorRow | null> {
  const data = await gqlAdmin<{ updateColor?: ColorRow[] }>(
    `mutation UpdateColor($input: ColorMutation!) { updateColor(input: $input) { colorId colorName } }`,
    { input: { colorId, colorName: colorName.trim() } }
  );
  return data?.updateColor?.[0] ?? null;
}

export async function deleteColor(colorId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteColor($input: DeleteColorInput!) { deleteColor(input: $input) { colorId } }`,
    { input: { colorId } }
  );
}

/** Fabric options for products (id "0" returns all) */
export interface FabricRow {
  fabricId: string;
  fabricName: string;
}

export async function fetchFabrics(): Promise<FabricRow[]> {
  const data = await gqlAdmin<{ searchFabric?: FabricRow[] }>(
    `mutation Fabrics { searchFabric(input: { fabricId: "0" }) { fabricId fabricName } }`
  );
  return data?.searchFabric ?? [];
}

export async function createFabric(fabricName: string): Promise<FabricRow | null> {
  const data = await gqlAdmin<{ createFabric?: FabricRow[] }>(
    `mutation CreateFabric($input: NewFabric!) { createFabric(input: $input) { fabricId fabricName } }`,
    { input: { fabricName: fabricName.trim() } }
  );
  return data?.createFabric?.[0] ?? null;
}

export async function updateFabric(fabricId: string, fabricName: string): Promise<FabricRow | null> {
  const data = await gqlAdmin<{ updateFabric?: FabricRow[] }>(
    `mutation UpdateFabric($input: FabricMutation!) { updateFabric(input: $input) { fabricId fabricName } }`,
    { input: { fabricId, fabricName: fabricName.trim() } }
  );
  return data?.updateFabric?.[0] ?? null;
}

export async function deleteFabric(fabricId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteFabric($input: DeleteFabricInput!) { deleteFabric(input: $input) { fabricId } }`,
    { input: { fabricId } }
  );
}

/** Weave options for products (id "0" returns all) */
export interface WeaveRow {
  weaveId: string;
  weaveName: string;
}

export async function fetchWeaves(): Promise<WeaveRow[]> {
  const data = await gqlAdmin<{ searchWeave?: WeaveRow[] }>(
    `mutation Weaves { searchWeave(input: { weaveId: "0" }) { weaveId weaveName } }`
  );
  return data?.searchWeave ?? [];
}

export async function createWeave(weaveName: string): Promise<WeaveRow | null> {
  const data = await gqlAdmin<{ createWeave?: WeaveRow[] }>(
    `mutation CreateWeave($input: NewWeave!) { createWeave(input: $input) { weaveId weaveName } }`,
    { input: { weaveName: weaveName.trim() } }
  );
  return data?.createWeave?.[0] ?? null;
}

export async function updateWeave(weaveId: string, weaveName: string): Promise<WeaveRow | null> {
  const data = await gqlAdmin<{ updateWeave?: WeaveRow[] }>(
    `mutation UpdateWeave($input: WeaveMutation!) { updateWeave(input: $input) { weaveId weaveName } }`,
    { input: { weaveId, weaveName: weaveName.trim() } }
  );
  return data?.updateWeave?.[0] ?? null;
}

export async function deleteWeave(weaveId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteWeave($input: DeleteWeaveInput!) { deleteWeave(input: $input) { weaveId } }`,
    { input: { weaveId } }
  );
}

const OCCASIONS_MUTATION = `mutation Occasions { searchOccasion(input: { occasionId: "0" }) { occasionId occasionName } }`;

export async function fetchOccasions(): Promise<OccasionRow[]> {
  const data = await gqlAdmin<{ searchOccasion?: OccasionRow[] }>(OCCASIONS_MUTATION);
  return data?.searchOccasion ?? [];
}

export async function createOccasion(occasionName: string): Promise<OccasionRow | null> {
  const data = await gqlAdmin<{ createOccasion?: OccasionRow[] }>(
    `mutation CreateOccasion($input: NewOccasion!) { createOccasion(input: $input) { occasionId occasionName } }`,
    { input: { occasionName: occasionName.trim() } }
  );
  return data?.createOccasion?.[0] ?? null;
}

export async function updateOccasion(
  occasionId: string,
  occasionName: string
): Promise<OccasionRow | null> {
  const data = await gqlAdmin<{ updateOccasion?: OccasionRow[] }>(
    `mutation UpdateOccasion($input: OccasionMutation!) { updateOccasion(input: $input) { occasionId occasionName } }`,
    { input: { occasionId, occasionName: occasionName.trim() } }
  );
  return data?.updateOccasion?.[0] ?? null;
}

export async function deleteOccasion(occasionId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteOccasion($input: DeleteOccasionInput!) { deleteOccasion(input: $input) { occasionId } }`,
    { input: { occasionId } }
  );
}

/** Product mood (id + name) for linking to products */
export interface ProductMoodRow {
  moodId: string;
  moodName: string;
}

export async function searchProductMoods(params?: {
  moodId?: string;
  moodName?: string;
}): Promise<ProductMoodRow[]> {
  const input: Record<string, string> = {};
  if (params?.moodId) input.moodId = params.moodId;
  if (params?.moodName) input.moodName = params.moodName;
  const data = await gqlAdmin<{ searchProductMood?: ProductMoodRow[] }>(
    `query SearchProductMoods($input: SearchProductMoodInput!) {
      searchProductMood(input: $input) { moodId moodName }
    }`,
    { input: Object.keys(input).length ? input : {} }
  );
  return data?.searchProductMood ?? [];
}

/** Moods from newest products (storefront; backend walks recent products for distinct mood ids). */
export async function fetchShopHighlightMoods(opts?: {
  recentProductLimit?: number;
  maxMoods?: number;
}): Promise<ProductMoodRow[]> {
  const data = await gqlAdmin<{ shopHighlightMoods?: ProductMoodRow[] }>(
    `query ShopHighlightMoods($recentProductLimit: Int, $maxMoods: Int) {
      shopHighlightMoods(recentProductLimit: $recentProductLimit, maxMoods: $maxMoods) {
        moodId moodName
      }
    }`,
    {
      recentProductLimit: opts?.recentProductLimit ?? 100,
      maxMoods: opts?.maxMoods ?? 12,
    }
  );
  return data?.shopHighlightMoods ?? [];
}

/** Create a new product mood. Returns the created mood (with moodId and moodName). */
export async function createProductMood(moodName: string): Promise<ProductMoodRow | null> {
  const name = moodName?.trim();
  if (!name) return null;
  const data = await gqlAdmin<{ createProductMood?: ProductMoodRow[] }>(
    `mutation CreateProductMood($input: NewProductMood!) {
      createProductMood(input: $input) { moodId moodName }
    }`,
    { input: { moodName: name } }
  );
  const list = data?.createProductMood ?? [];
  return list.length > 0 ? list[0] : null;
}

export async function updateProductMood(
  moodId: string,
  moodName: string
): Promise<ProductMoodRow | null> {
  const data = await gqlAdmin<{ updateProductMood?: ProductMoodRow[] }>(
    `mutation UpdateProductMood($input: ProductMoodMutation!) {
      updateProductMood(input: $input) { moodId moodName }
    }`,
    { input: { moodId, moodName: moodName.trim() } }
  );
  return data?.updateProductMood?.[0] ?? null;
}

export async function deleteProductMood(moodId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteProductMood($input: DeleteProductMoodInput!) {
      deleteProductMood(input: $input) { moodId }
    }`,
    { input: { moodId } }
  );
}

/** List mood mappings for a product (for edit form). Returns array of { productId, moodId }. */
export async function searchProductMoodMappingsByProduct(
  productId: string
): Promise<{ productId: string; moodId: string }[]> {
  const data = await gqlAdmin<{ searchProductMoodMapping?: { productId: string; moodId: string }[] }>(
    `mutation SearchProductMoodMappings($input: SearchProductMoodMappingInput!) {
      searchProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId } }
  );
  return data?.searchProductMoodMapping ?? [];
}

/** Link a product to a mood. */
export async function createProductMoodMapping(
  productId: string,
  moodId: string
): Promise<void> {
  await gqlAdmin<{ createProductMoodMapping?: unknown[] }>(
    `mutation CreateProductMoodMapping($input: NewProductMoodMapping!) {
      createProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId, moodId } }
  );
}

/** Unlink a product from a mood. */
export async function deleteProductMoodMapping(
  productId: string,
  moodId: string
): Promise<void> {
  await gqlAdmin<{ deleteProductMoodMapping?: unknown[] }>(
    `mutation DeleteProductMoodMapping($input: DeleteProductMoodMappingInput!) {
      deleteProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId, moodId } }
  );
}

/** Create a product variant. Returns created variant with variantId. */
export interface ProductVariantRow {
  variantId: string;
  productId: string;
  sizeId?: string | null;
  colorId?: string | null;
  additionalPricePaise?: string | null;
}

export async function createProductVariant(params: {
  productId: string;
  sizeId?: string;
  colorId?: string;
  additionalPricePaise?: string;
}): Promise<ProductVariantRow | null> {
  const input: Record<string, string> = { productId: params.productId };
  if (params.sizeId) input.sizeId = params.sizeId;
  if (params.colorId) input.colorId = params.colorId;
  if (params.additionalPricePaise != null) input.additionalPricePaise = params.additionalPricePaise;
  const data = await gqlAdmin<{ createProductVariant?: ProductVariantRow[] }>(
    `mutation CreateProductVariant($input: NewProductVariant!) {
      createProductVariant(input: $input) { variantId productId sizeId colorId additionalPricePaise }
    }`,
    { input }
  );
  return data?.createProductVariant?.[0] ?? null;
}

/** Update a product variant (size/color/additional price). */
export async function updateProductVariant(params: {
  variantId: string;
  productId?: string;
  sizeId?: string;
  colorId?: string;
  additionalPricePaise?: string;
}): Promise<void> {
  const input: Record<string, string> = { variantId: params.variantId };
  if (params.productId) input.productId = params.productId;
  if (params.sizeId !== undefined) input.sizeId = params.sizeId;
  if (params.colorId !== undefined) input.colorId = params.colorId;
  if (params.additionalPricePaise !== undefined) {
    input.additionalPricePaise = params.additionalPricePaise;
  }
  await gqlAdmin<{ updateProductVariant?: unknown[] }>(
    `mutation UpdateProductVariant($input: ProductVariantMutation!) {
      updateProductVariant(input: $input) {
        variantId
        productId
        sizeId
        colorId
        additionalPricePaise
      }
    }`,
    { input }
  );
}

/** Delete a variant by id. */
export async function deleteProductVariant(variantId: string): Promise<void> {
  await gqlAdmin<{ deleteProductVariant?: unknown[] }>(
    `mutation DeleteProductVariant($input: DeleteProductVariantInput!) {
      deleteProductVariant(input: $input) { variantId }
    }`,
    { input: { variantId } }
  );
}

/** Create inventory for a variant (quantity and optional reorder level). */
export async function createInventoryItem(params: {
  variantId: string;
  quantityAvailable: string;
  reorderLevel?: string;
}): Promise<void> {
  const input: Record<string, string> = {
    variantId: params.variantId,
    quantityAvailable: params.quantityAvailable,
  };
  if (params.reorderLevel != null) input.reorderLevel = params.reorderLevel;
  await gqlAdmin<{ createInventoryItem?: unknown[] }>(
    `mutation CreateInventoryItem($input: NewInventoryItem!) {
      createInventoryItem(input: $input) { inventoryId variantId quantityAvailable reorderLevel }
    }`,
    { input }
  );
}

export interface InventoryItemRow {
  inventoryId: string;
  variantId: string;
  quantityAvailable: string;
  reorderLevel: string;
}

/** Find inventory rows by variant id (usually 0 or 1 row). */
export async function searchInventoryByVariantId(
  variantId: string
): Promise<InventoryItemRow[]> {
  const data = await gqlAdmin<{ searchInventoryItem?: InventoryItemRow[] }>(
    `query SearchInventoryByVariant($input: SearchInventoryItem!) {
      searchInventoryItem(input: $input) {
        inventoryId
        variantId
        quantityAvailable
        reorderLevel
      }
    }`,
    { input: { variantId } }
  );
  return data?.searchInventoryItem ?? [];
}

/** Update inventory row quantity/reorder level. */
export async function updateInventoryItem(params: {
  inventoryId: string;
  quantityAvailable?: string;
  reorderLevel?: string;
}): Promise<void> {
  const input: Record<string, string> = { inventoryId: params.inventoryId };
  if (params.quantityAvailable != null) input.quantityAvailable = params.quantityAvailable;
  if (params.reorderLevel != null) input.reorderLevel = params.reorderLevel;
  await gqlAdmin<{ updateInventoryItem?: unknown[] }>(
    `mutation UpdateInventoryItem($input: InventoryItemMutation!) {
      updateInventoryItem(input: $input) {
        inventoryId
        variantId
        quantityAvailable
        reorderLevel
      }
    }`,
    { input }
  );
}

/** All inventory rows across every variant (search_inventory_item with no filters = no WHERE
 * clause server-side, i.e. everything) — for the standalone stock/low-stock page. No pagination
 * on this query server-side; fine at boutique catalog scale, would need a real limit/offset
 * added backend-side before this scales further. */
export async function fetchAllInventoryItems(): Promise<InventoryItemRow[]> {
  const data = await gqlAdmin<{ searchInventoryItem?: InventoryItemRow[] }>(
    `query SearchAllInventory($input: SearchInventoryItem!) {
      searchInventoryItem(input: $input) {
        inventoryId
        variantId
        quantityAvailable
        reorderLevel
      }
    }`,
    { input: {} }
  );
  return data?.searchInventoryItem ?? [];
}

export async function deleteInventoryItem(inventoryId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteInventoryItem($inventoryId: String!) {
      deleteInventoryItem(inventoryId: $inventoryId) { inventoryId }
    }`,
    { inventoryId }
  );
}

export interface InventoryLogRow {
  logId: string;
  variantId: string;
  changeQuantity: string;
  logTime: string;
  reason: string;
}

/** All inventory log entries, newest first. Note: nothing in the backend writes these
 * automatically (no stock-changing operation — orders, cancellations, admin restocks — calls
 * create_inventory_log) — this table only has rows if an admin manually adds them here. */
export async function fetchAllInventoryLogs(): Promise<InventoryLogRow[]> {
  const data = await gqlAdmin<{ searchInventoryLog?: InventoryLogRow[] }>(
    `query SearchAllInventoryLogs($input: SearchInventoryLogInput!) {
      searchInventoryLog(input: $input) {
        logId
        variantId
        changeQuantity
        logTime
        reason
      }
    }`,
    { input: {} }
  );
  const rows = data?.searchInventoryLog ?? [];
  return [...rows].sort((a, b) => new Date(b.logTime).getTime() - new Date(a.logTime).getTime());
}

export async function createInventoryLog(params: {
  variantId: string;
  changeQuantity: string;
  reason: string;
}): Promise<InventoryLogRow | null> {
  const data = await gqlAdmin<{ createInventoryLog?: InventoryLogRow[] }>(
    `mutation CreateInventoryLog($input: NewInventoryLog!) {
      createInventoryLog(input: $input) {
        logId
        variantId
        changeQuantity
        logTime
        reason
      }
    }`,
    { input: params }
  );
  return data?.createInventoryLog?.[0] ?? null;
}

export async function deleteInventoryLog(logId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteInventoryLog($input: DeleteInventoryLogInput!) {
      deleteInventoryLog(input: $input) { logId }
    }`,
    { input: { logId } }
  );
}
