type CouponCodeFieldProps = {
  value: string;
  onChange: (value: string) => void;
  appliedCouponCode?: string | null;
  message?: string | null;
  applying?: boolean;
  onApply: () => void;
  onRemove: () => void;
};

/** Coupon code entry + apply/remove, used in the bag order summary. Clicking Apply runs an
 * immediate backend check (exists/active/date-window/usage-limit/min-order) before the code is
 * ever marked "applied" — `applying` reflects that in-flight check, `message` its rejection
 * reason. Cart-scope/per-customer eligibility is checked separately, later, once applied. */
export function CouponCodeField({
  value,
  onChange,
  appliedCouponCode,
  message,
  applying = false,
  onApply,
  onRemove,
}: CouponCodeFieldProps) {
  const isApplied = !!appliedCouponCode && !message && !applying;

  return (
    <div className="mt-4 border-t border-[var(--color-line)] pt-4">
      {isApplied ? (
        <div className="flex items-center justify-between gap-3 rounded-md border border-[var(--color-green)]/40 bg-[var(--color-green)]/5 px-3 py-2">
          <p className="text-sm font-medium text-[var(--color-green)]">
            &ldquo;{appliedCouponCode}&rdquo; applied
          </p>
          <button
            type="button"
            onClick={onRemove}
            className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--color-muted)] underline hover:text-[var(--color-ink)]"
          >
            Remove
          </button>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value.toUpperCase())}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                if (value.trim()) onApply();
              }
            }}
            placeholder="Coupon code"
            aria-label="Coupon code"
            className="h-10 min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm uppercase tracking-[0.04em] text-[var(--color-ink)] placeholder:normal-case placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
          <button
            type="button"
            onClick={onApply}
            disabled={!value.trim() || applying}
            className="h-10 shrink-0 rounded-md border border-[var(--color-green)] px-4 text-xs font-semibold uppercase tracking-[0.1em] text-[var(--color-green)] transition hover:bg-[var(--color-green)] hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
          >
            {applying ? "Applying…" : "Apply"}
          </button>
        </div>
      )}
      {message ? (
        <p className="mt-2 text-xs leading-relaxed text-rose-700" role="alert">
          {message}
        </p>
      ) : null}
    </div>
  );
}
