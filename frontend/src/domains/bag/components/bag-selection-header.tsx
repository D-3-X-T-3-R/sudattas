import { cn } from "@/lib/utils";
import { CheckIcon } from "@/domains/bag/components/bag-shared";

type BagSelectionHeaderProps = {
  allSelected: boolean;
  selectedCount: number;
  totalCount: number;
  onToggleAll: () => void;
};

export function BagSelectionHeader({
  allSelected,
  selectedCount,
  totalCount,
  onToggleAll,
}: BagSelectionHeaderProps) {
  return (
    <div className="flex flex-row items-stretch justify-between gap-2 border-b border-[#0F3D2E]/8 pb-8 sm:gap-3">
      <div className="flex min-w-0">
        <p className="inline-flex h-full min-h-[44px] items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px] sm:tracking-[0.28em]">
          Shopping Bag
        </p>
      </div>
      <div className="inline-flex shrink-0 items-center gap-2 rounded-full border border-[#0F3D2E]/10 bg-[#FFFDF8] px-2.5 py-2 sm:gap-2.5 sm:px-3">
        <button
          type="button"
          onClick={onToggleAll}
          aria-label="Select all items"
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors",
            allSelected
              ? "bg-[#0F3D2E] text-[#F6F3EA] shadow-[0_6px_12px_rgba(15,61,46,0.18)]"
              : "border border-[var(--color-line)] bg-white text-[var(--color-muted)]"
          )}
        >
          <CheckIcon className="h-3 w-3" />
        </button>
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">Selected Items</p>
          <p className="text-[11px] font-semibold text-[#162019]">
            {selectedCount} / {totalCount} selected
          </p>
        </div>
      </div>
    </div>
  );
}
