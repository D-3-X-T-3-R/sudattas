"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { InventoryStockCard } from "@/domains/admin/inventory/components/inventory-stock-card";
import { InventoryLogsCard } from "@/domains/admin/inventory/components/inventory-logs-card";
import type { InventoryDisplayRow, VariantLabelMap } from "@/domains/admin/inventory/types";
import { fetchAllInventoryItems, fetchAllInventoryLogs, fetchProductsList } from "@/lib/admin-queries";

/** Cap on the product sweep used to build the variantId -> product/size label map. Products
 * beyond this won't show a friendly name here (falls back to "Variant #N") — same known
 * non-paginated-fetch limitation as the rest of the admin catalog screens (see P1-14). */
const PRODUCT_SWEEP_LIMIT = "500";

export default function AdminInventoryPage() {
  const [onlyLowStock, setOnlyLowStock] = useState(false);

  const productsQuery = useQuery({
    queryKey: ["admin", "inventory-products"],
    queryFn: () => fetchProductsList({ limit: PRODUCT_SWEEP_LIMIT }),
  });

  const inventoryQuery = useQuery({
    queryKey: ["admin", "inventory-all"],
    queryFn: fetchAllInventoryItems,
  });

  const logsQuery = useQuery({
    queryKey: ["admin", "inventory-logs"],
    queryFn: fetchAllInventoryLogs,
  });

  const variantLabels: VariantLabelMap = useMemo(() => {
    const map: VariantLabelMap = new Map();
    for (const product of productsQuery.data ?? []) {
      for (const v of product.variantStock ?? []) {
        if (v.variantId) {
          map.set(v.variantId, { productName: product.name, sizeName: v.sizeName || "Free Size" });
        }
      }
    }
    return map;
  }, [productsQuery.data]);

  const stockRows: InventoryDisplayRow[] = useMemo(() => {
    return (inventoryQuery.data ?? []).map((inv) => {
      const label = variantLabels.get(inv.variantId);
      const quantityAvailable = Number(inv.quantityAvailable) || 0;
      const reorderLevel = Number(inv.reorderLevel) || 0;
      return {
        inventoryId: inv.inventoryId,
        variantId: inv.variantId,
        productName: label?.productName ?? `Variant #${inv.variantId}`,
        sizeName: label?.sizeName ?? "-",
        quantityAvailable,
        reorderLevel,
        isLowStock: quantityAvailable <= reorderLevel,
      };
    });
  }, [inventoryQuery.data, variantLabels]);

  return (
    <AdminPageShell
      label="Inventory"
      title="Inventory"
      description="Stock levels across every product variant, and a manual change log."
    >
      <div className="space-y-6">
        <InventoryStockCard
          rows={stockRows}
          isLoading={productsQuery.isLoading || inventoryQuery.isLoading}
          isError={productsQuery.isError || inventoryQuery.isError}
          onlyLowStock={onlyLowStock}
          setOnlyLowStock={setOnlyLowStock}
        />

        <InventoryLogsCard
          logs={logsQuery.data ?? []}
          isLoading={logsQuery.isLoading}
          isError={logsQuery.isError}
          variantLabels={variantLabels}
        />
      </div>
    </AdminPageShell>
  );
}
