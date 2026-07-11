import Link from "next/link";
import { PageShell, SectionHeader } from "@/components/ui/page-shell";

const SUPPORT_CARDS = [
  {
    heading: "Fastest order support",
    body: "Open Profile > Orders for cancellation eligibility, refund status, courier updates, and support request options tied to your order.",
  },
  {
    heading: "Styling and bulk enquiries",
    body: (
      <>
        Email{" "}
        <a className="font-semibold text-[var(--color-green)] hover:text-[var(--color-green-2)]" href="mailto:sudattasdesignerboutique@gmail.com">
          sudattasdesignerboutique@gmail.com
        </a>{" "}
        with your event date, order reference if available, and the assistance you need.
      </>
    ),
  },
];

export default function ContactSupportPage() {
  return (
    <PageShell containerClassName="py-8 md:py-10">
      <SectionHeader
        label="Contact Support"
        title="Order help with a real support path."
        description="For delivery updates, cancellation questions, refund status, or styling assistance, start with the order detail page in your profile. That gives our team the exact order context immediately."
        className="max-w-4xl"
      />

      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {SUPPORT_CARDS.map((card) => (
          <div key={card.heading} className="rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-5 md:p-6">
            <h2 className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--color-accent-gold)]">
              {card.heading}
            </h2>
            <p className="mt-3 text-sm leading-7 text-[var(--color-muted)]">{card.body}</p>
          </div>
        ))}
      </div>

      <div className="mt-8 flex flex-col gap-3 border-t border-[var(--color-line)] pt-8 sm:flex-row">
        <Link
          href="/profile"
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition-colors hover:bg-[var(--color-green-2)]"
        >
          Go To Orders
        </Link>
        <Link
          href="/returns-exchanges"
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-gold)]"
        >
          Returns &amp; Exchanges
        </Link>
      </div>
    </PageShell>
  );
}
