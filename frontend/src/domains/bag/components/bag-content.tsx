import { AnimatePresence } from "framer-motion";
import type { CartLine } from "@/lib/schemas";
import { BagSelectionHeader } from "@/domains/bag/components/bag-selection-header";
import { BagLineCard } from "@/domains/bag/components/bag-line-card";
import { OrderSummaryPanel } from "@/domains/bag/components/order-summary-panel";

type CatalogSize = { sizeId: string; sizeName: string };

type BagContentProps = {
  cartLines: CartLine[];
  selectedLineIds: Set<string>;
  selectedLines: CartLine[];
  selectedSubtotal: number;
  selectedCount: number;
  shippingAmount: number;
  shippingLoading: boolean;
  shippingNote?: string | null;
  allSelected: boolean;
  catalogSizes: CatalogSize[];
  openSizeForId: string | null;
  reduceMotion: boolean;
  wishlist: Record<string, boolean>;
  onToggleAll: () => void;
  onToggleOne: (id: string) => void;
  onSetOpenSizeForId: (id: string | null) => void;
  onDecCart: (id: string) => void;
  onIncCart: (id: string) => void;
  onRemoveCart: (id: string) => Promise<void> | void;
  onToggleWish: (product: CartLine["product"]) => void;
  onAddToCart: (product: CartLine["product"], qty?: number, sizeName?: string | null) => Promise<void> | void;
  onCheckout: () => void;
};

export function BagContent({
  cartLines,
  selectedLineIds,
  selectedLines,
  selectedSubtotal,
  selectedCount,
  shippingAmount,
  shippingLoading,
  shippingNote,
  allSelected,
  catalogSizes,
  openSizeForId,
  reduceMotion,
  wishlist,
  onToggleAll,
  onToggleOne,
  onSetOpenSizeForId,
  onDecCart,
  onIncCart,
  onRemoveCart,
  onToggleWish,
  onAddToCart,
  onCheckout,
}: BagContentProps) {
  return (
    <div className="space-y-6 pb-28 md:pb-6 lg:flex lg:h-full lg:min-h-0 lg:flex-col lg:pb-0">
      <BagSelectionHeader
        allSelected={allSelected}
        selectedCount={selectedLineIds.size}
        totalCount={cartLines.length}
        onToggleAll={onToggleAll}
      />

      <div className="mt-8 grid min-h-0 flex-1 gap-7 xl:grid-cols-[minmax(0,1fr)_390px]">
        <div className="flex h-full min-h-0 flex-col">
          <div className="min-h-0 flex-1 space-y-6 overflow-y-auto pr-2 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
            <AnimatePresence initial={false} mode="popLayout">
              {cartLines.map((line) => (
                <BagLineCard
                  key={line.id}
                  line={line}
                  catalogSizes={catalogSizes}
                  isSelected={selectedLineIds.has(line.id)}
                  isSizeMenuOpen={openSizeForId === line.id}
                  reduceMotion={reduceMotion}
                  onOpenSizeChange={(isOpen) => onSetOpenSizeForId(isOpen ? line.id : null)}
                  onSelect={() => onToggleOne(line.id)}
                  onDec={onDecCart}
                  onInc={onIncCart}
                  onRemove={onRemoveCart}
                  onMoveToWishlist={async (currentLine) => {
                    if (!wishlist[currentLine.product.id]) {
                      onToggleWish(currentLine.product);
                    }
                    await onRemoveCart(currentLine.id);
                  }}
                  onReplaceSize={async (currentLine, newSize) => {
                    if (!newSize || newSize === currentLine.sizeName) return;
                    await onRemoveCart(currentLine.id);
                    await onAddToCart(currentLine.product, currentLine.qty, newSize);
                  }}
                />
              ))}
            </AnimatePresence>
          </div>
        </div>

        <div className="xl:min-h-0 xl:overflow-y-auto [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
          <OrderSummaryPanel
            cartLines={selectedLines}
            cartSubtotal={selectedSubtotal}
            cartCount={selectedCount}
            shippingAmount={shippingAmount}
            shippingLoading={shippingLoading}
            shippingNote={shippingNote}
            onCheckout={onCheckout}
          />
        </div>
      </div>
    </div>
  );
}
