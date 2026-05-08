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
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 shadow-[var(--shadow-subtle)]">
      <div>
        <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
          Selected Pieces
        </p>
        <p className="mt-1 text-sm text-[var(--color-ink)]">
          {selectedCount} of {totalCount} selected
        </p>
      </div>
      <button
        type="button"
        onClick={onToggleAll}
        className={cn(
          "inline-flex items-center gap-2 rounded-md border px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.14em]",
          allSelected
            ? "border-[var(--color-green)] bg-[var(--color-green)] text-white"
            : "border-[var(--color-line)] bg-white text-[var(--color-ink)]"
        )}
      >
        <span className="inline-flex h-4 w-4 items-center justify-center rounded-sm border border-current">
          <CheckIcon className="h-3 w-3" />
        </span>
        {allSelected ? "Deselect All" : "Select All"}
      </button>
    </div>
  );
}
