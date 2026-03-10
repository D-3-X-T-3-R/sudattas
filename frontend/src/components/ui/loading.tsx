"use client";

export function Spinner() {
  return (
    <div className="inline-flex items-center justify-center gap-2 text-xs text-[var(--color-muted)]">
      <span className="h-3 w-3 animate-spin rounded-full border border-[var(--color-line)] border-t-[var(--color-ink)]" />
      <span>Loading…</span>
    </div>
  );
}

