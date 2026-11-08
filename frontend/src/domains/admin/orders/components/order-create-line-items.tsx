"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Trash2, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { fetchProductsList } from "@/lib/admin-queries";
import { formatInrFromPaise, paiseToRupeesInput, rupeesInputToPaise } from "@/lib/money";

export interface OrderLineDraft {
  key: string;
  productName: string;
  variantId: string;
  sizeName: string;
  quantity: string;
  unitPricePaise: string;
}

interface OrderCreateLineItemsProps {
  lines: OrderLineDraft[];
  onAdd: (line: OrderLineDraft) => void;
  onRemove: (key: string) => void;
}

export function OrderCreateLineItems({ lines, onAdd, onRemove }: OrderCreateLineItemsProps) {
  const [search, setSearch] = useState("");
  const [selectedProductId, setSelectedProductId] = useState("");
  const [selectedVariantId, setSelectedVariantId] = useState("");
  const [quantity, setQuantity] = useState("1");
  /** What the admin sees/types — rupees. Converted to paise only when the line is added. */
  const [unitPriceInput, setUnitPriceInput] = useState("");

  const searchQuery = useQuery({
    queryKey: ["admin", "order-create-product-search", search],
    queryFn: () => fetchProductsList({ name: search.trim() || undefined, limit: "20" }),
    enabled: search.trim().length > 0,
  });

  const selectedProduct = (searchQuery.data ?? []).find((p) => p.productId === selectedProductId);
  const variantOptions = selectedProduct?.variantStock ?? [];

  const subtotalPaise = lines.reduce(
    (sum, l) => sum + (Number(l.unitPricePaise) || 0) * (Number(l.quantity) || 0),
    0
  );

  const canAdd = selectedVariantId && Number(quantity) > 0 && Number(unitPriceInput) >= 0;

  return (
    <AdminTableCard title="Line items" icon={<Package className="h-4 w-4 text-[var(--color-green)]" />}>
      <div className="space-y-3 rounded-lg border border-[var(--color-line)] p-3">
        <Input
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setSelectedProductId("");
            setSelectedVariantId("");
          }}
          placeholder="Search products by name…"
          className="rounded-lg text-[15px]"
        />

        {search.trim() && (
          <div className="max-h-40 overflow-y-auto rounded-lg border border-[var(--color-line)]">
            {searchQuery.isLoading ? (
              <p className="p-2.5 text-sm text-[var(--color-muted)]">Searching…</p>
            ) : (searchQuery.data ?? []).length === 0 ? (
              <p className="p-2.5 text-sm text-[var(--color-muted)]">No matching products.</p>
            ) : (
              <ul>
                {(searchQuery.data ?? []).map((p) => (
                  <li key={p.productId}>
                    <button
                      type="button"
                      onClick={() => {
                        setSelectedProductId(p.productId);
                        setSelectedVariantId("");
                        setUnitPriceInput(paiseToRupeesInput(p.amountPaise));
                      }}
                      className={`w-full px-2.5 py-2 text-left text-[15px] hover:bg-[var(--color-surface-soft)] ${
                        selectedProductId === p.productId ? "bg-[var(--color-surface-soft)] font-medium" : ""
                      }`}
                    >
                      {p.name} <span className="text-[var(--color-muted)]">({p.formatted})</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {selectedProduct && (
          <div className="flex flex-wrap items-end gap-2.5">
            <label className="text-sm text-[var(--color-muted)]">
              Size
              <select
                value={selectedVariantId}
                onChange={(e) => setSelectedVariantId(e.target.value)}
                className="mt-1 block h-10 min-w-[10rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              >
                <option value="">Select…</option>
                {variantOptions.map((v) => (
                  <option key={v.variantId} value={v.variantId} disabled={v.quantity <= 0}>
                    {v.sizeName || "Free Size"} {v.quantity <= 0 ? "(out of stock)" : `(${v.quantity} in stock)`}
                  </option>
                ))}
              </select>
            </label>
            <label className="text-sm text-[var(--color-muted)]">
              Qty
              <Input
                value={quantity}
                onChange={(e) => setQuantity(e.target.value)}
                className="mt-1 h-10 w-20 rounded-lg text-[15px]"
              />
            </label>
            <label className="text-sm text-[var(--color-muted)]">
              Unit price (₹)
              <Input
                value={unitPriceInput}
                onChange={(e) => setUnitPriceInput(e.target.value)}
                className="mt-1 h-10 w-32 rounded-lg text-[15px]"
              />
            </label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!canAdd}
              onClick={() => {
                if (!selectedProduct) return;
                const variant = variantOptions.find((v) => v.variantId === selectedVariantId);
                onAdd({
                  key: `${selectedVariantId}-${Date.now()}`,
                  productName: selectedProduct.name,
                  variantId: selectedVariantId,
                  sizeName: variant?.sizeName || "Free Size",
                  quantity,
                  unitPricePaise: String(rupeesInputToPaise(unitPriceInput)),
                });
                setSearch("");
                setSelectedProductId("");
                setSelectedVariantId("");
                setQuantity("1");
                setUnitPriceInput("");
              }}
            >
              Add line
            </Button>
          </div>
        )}
      </div>

      {lines.length > 0 ? (
        <div className="mt-4 overflow-x-auto">
          <table className="w-full min-w-[560px] border-collapse text-[15px]">
            <caption className="sr-only">Order line items</caption>
            <thead>
              <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                <th className="pb-2 pr-4 font-medium">Product</th>
                <th className="pb-2 pr-4 font-medium">Size</th>
                <th className="pb-2 pr-4 font-medium">Qty</th>
                <th className="pb-2 pr-4 font-medium">Unit price</th>
                <th className="pb-2 pr-4 font-medium">Line total</th>
                <th className="pb-2 font-medium sr-only">Actions</th>
              </tr>
            </thead>
            <tbody>
              {lines.map((l) => (
                <tr key={l.key} className="border-b border-[var(--color-line)] last:border-0">
                  <td className="py-2.5 pr-4 text-[var(--color-ink)]">{l.productName}</td>
                  <td className="py-2.5 pr-4 text-[var(--color-muted)]">{l.sizeName}</td>
                  <td className="py-2.5 pr-4 text-[var(--color-ink)]">{l.quantity}</td>
                  <td className="py-2.5 pr-4 text-[var(--color-ink)]">
                    {formatInrFromPaise(Number(l.unitPricePaise) || 0)}
                  </td>
                  <td className="py-2.5 pr-4 text-[var(--color-ink)]">
                    {formatInrFromPaise((Number(l.unitPricePaise) || 0) * (Number(l.quantity) || 0))}
                  </td>
                  <td className="py-2.5 text-right">
                    <button
                      type="button"
                      onClick={() => onRemove(l.key)}
                      aria-label={`Remove ${l.productName} — ${l.sizeName}`}
                      className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className="mt-3 text-right text-[15px] font-semibold text-[var(--color-ink)]">
            Subtotal: {formatInrFromPaise(subtotalPaise)}
          </p>
        </div>
      ) : (
        <p className="mt-3 text-sm text-[var(--color-muted)]">No line items added yet.</p>
      )}
    </AdminTableCard>
  );
}
