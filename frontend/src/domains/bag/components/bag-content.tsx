import { AnimatePresence } from "framer-motion";
import type { CartLine } from "@/lib/schemas";
import { BagSelectionHeader } from "@/domains/bag/components/bag-selection-header";
import { BagLineCard } from "@/domains/bag/components/bag-line-card";
import { TrustStrip } from "@/components/trust-strip";

type CatalogSize = { sizeId: string; sizeName: string };

type BagContentProps = {
  cartLines: CartLine[];
  selectedLineIds: Set<string>;
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
  onAddToCart: (
    product: CartLine["product"],
    qty?: number,
    sizeName?: string | null
  ) => Promise<void> | void;
};

export function BagContent({
  cartLines,
  selectedLineIds,
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
}: BagContentProps) {
  return (
    <div className="space-y-4">
      <BagSelectionHeader
        allSelected={allSelected}
        selectedCount={selectedLineIds.size}
        totalCount={cartLines.length}
        onToggleAll={onToggleAll}
      />

      <div className="space-y-3 md:space-y-4">
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

      <TrustStrip className="hidden lg:block" />
    </div>
  );
}
