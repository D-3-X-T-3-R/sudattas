import { INR } from "@/lib/constants";

export function AnnouncementBar() {
  return (
    <div
      className="relative border-b border-[var(--color-line)] text-[11px] text-[var(--color-muted)]"
    >
      <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-2.5 pl-6 sm:pl-4">
        <span className="flex items-center gap-2">
          <span className="absolute left-0 top-0 bottom-0 w-1 bg-[var(--color-accent-gold)]" />
          Complimentary shipping above {INR.format(4999)} • 7-day returns
        </span>
        <span className="hidden sm:block">
          Support: WhatsApp-style chat (replace)
        </span>
      </div>
    </div>
  );
}
