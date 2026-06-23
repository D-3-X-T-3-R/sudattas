import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";

type BagSizeOption = { sizeId: string; sizeName: string };

type BagSizeDropdownProps = {
  options: BagSizeOption[];
  sizeName: string | null | undefined;
  hasCurrent: boolean;
  onSelectSize: (newSize: string) => void | Promise<void>;
  onOpenChange?: (open: boolean) => void;
};

export function BagSizeDropdown({
  options,
  sizeName,
  hasCurrent,
  onSelectSize,
  onOpenChange,
}: BagSizeDropdownProps) {
  const [open, setOpen] = useState(false);
  const [openUpward, setOpenUpward] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDoc = (event: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, []);

  useEffect(() => {
    onOpenChange?.(open);
    if (!open) return undefined;

    // Flip the menu above the trigger when there isn't enough room below — otherwise it
    // can get clipped at the bottom of the viewport on cards lower down the page.
    const root = rootRef.current;
    if (root) {
      const rect = root.getBoundingClientRect();
      const estimatedMenuHeight = Math.min(208, options.length * 44 + 16);
      const spaceBelow = window.innerHeight - rect.bottom;
      setOpenUpward(spaceBelow < estimatedMenuHeight && rect.top > estimatedMenuHeight);
    }

    return () => onOpenChange?.(false);
  }, [open, onOpenChange, options.length]);

  const display = hasCurrent && sizeName ? sizeName : "Choose";

  return (
    <div ref={rootRef} className="relative inline-flex w-fit max-w-full">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="listbox"
        onClick={() => setOpen((value) => !value)}
        className="relative inline-flex h-9 w-fit max-w-full items-center gap-1.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] pl-3 pr-8 text-left transition-shadow hover:shadow-sm"
      >
        <span className="shrink-0 text-[10px] font-medium uppercase leading-none tracking-[0.14em] text-[var(--color-muted)]">
          Size:
        </span>
        <span className="shrink-0 text-sm font-bold leading-none tracking-tight text-[var(--color-ink)]">
          {display}
        </span>
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
          className={cn(
            "pointer-events-none absolute right-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--color-muted)] transition-transform duration-200",
            open && "rotate-180"
          )}
        >
          <path d="m6 9 6 6 6-6" />
        </svg>
      </button>

      {open && (
        <ul
          role="listbox"
          className={cn(
            "absolute left-0 z-50 max-h-52 w-full overflow-y-auto rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1.5 shadow-[var(--shadow-soft)] [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
            openUpward ? "bottom-[calc(100%+6px)]" : "top-[calc(100%+6px)]"
          )}
        >
          {!hasCurrent && (
            <li className="px-4 py-2 text-xs uppercase tracking-[0.12em] text-[var(--color-muted)]">
              Choose size
            </li>
          )}
          {options.map((option) => {
            const selected = sizeName === option.sizeName;
            return (
              <li key={option.sizeId} role="option" aria-selected={selected}>
                <button
                  type="button"
                  className={cn(
                    "flex w-full items-center px-4 py-3 text-left text-base font-semibold tracking-wide text-[var(--color-ink)] transition-colors",
                    selected
                      ? "bg-[var(--color-surface-soft)] text-[var(--color-green)]"
                      : "hover:bg-[var(--color-surface-soft)]"
                  )}
                  onClick={() => {
                    setOpen(false);
                    void onSelectSize(option.sizeName);
                  }}
                >
                  {option.sizeName}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
