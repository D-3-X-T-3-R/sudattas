import { cn } from "@/lib/utils";

type StatusTone = "green" | "gold" | "blue" | "muted" | "red";

const TONE_CLASSES: Record<StatusTone, string> = {
  green: "border-emerald-200 bg-emerald-50 text-emerald-600",
  gold: "border-amber-200 bg-amber-50 text-amber-800",
  blue: "border-sky-200 bg-sky-50 text-sky-800",
  muted: "border-[var(--color-line)] bg-[var(--color-surface-soft)] text-[var(--color-muted)]",
  red: "border-rose-200 bg-rose-50 text-rose-800",
};

const STATUS_TONES: Record<string, StatusTone> = {
  pending: "gold",
  confirmed: "blue",
  processing: "blue",
  "processing order": "blue",
  needs_review: "gold",
  "needs review": "gold",
  shipped: "blue",
  "in transit": "blue",
  delivered: "green",
  cancelled: "red",
  refunded: "muted",
  draft: "muted",
  active: "green",
  archived: "muted",
};

function toneForStatus(statusName: string): StatusTone {
  const key = statusName.trim().toLowerCase();
  return STATUS_TONES[key] ?? "muted";
}

/** Color-coded pill for order/product statuses — lets a status be read at a glance instead of parsed as text. */
export function StatusBadge({ label, className }: { label: string; className?: string }) {
  const tone = toneForStatus(label);
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-sm font-medium capitalize",
        TONE_CLASSES[tone],
        className
      )}
    >
      <span className="h-1.5 w-1.5 rounded-full bg-current" aria-hidden="true" />
      {label}
    </span>
  );
}
