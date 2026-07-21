"use client";

export function Spinner() {
  return (
    <div className="inline-flex items-center justify-center gap-2 text-xs uppercase tracking-[0.16em] text-[var(--color-muted)]">
      <span className="h-3.5 w-3.5 animate-spin rounded-full border border-[var(--color-line)] border-t-[var(--color-green)]" />
      <span>Loading...</span>
    </div>
  );
}
