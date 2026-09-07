export interface InventoryDisplayRow {
  inventoryId: string;
  variantId: string;
  productName: string;
  sizeName: string;
  quantityAvailable: number;
  reorderLevel: number;
  isLowStock: boolean;
}

/** variantId -> "Product Name — Size" label, built once from the product list's embedded
 * variantStock and reused by both the stock table and the log-entry form's variant picker. */
export type VariantLabelMap = Map<string, { productName: string; sizeName: string }>;
