import Link from "next/link";
import { PageShell, SectionHeader } from "@/components/ui/page-shell";

type PolicyPageShellProps = {
  eyebrow: string;
  title: string;
  intro: string;
  sections: Array<{ heading: string; body: string[] }>;
  supportHref?: string;
  supportLabel?: string;
};

export function PolicyPageShell({
  eyebrow,
  title,
  intro,
  sections,
  supportHref = "/contact-support",
  supportLabel = "Contact Support",
}: PolicyPageShellProps) {
  return (
    <PageShell containerClassName="py-8 md:py-10">
      <SectionHeader label={eyebrow} title={title} description={intro} className="max-w-4xl" />

      <div className="mt-8 divide-y divide-[var(--color-line)]">
        {sections.map((section) => (
          <section key={section.heading} className="py-6 md:py-8">
            <h2 className="font-display text-xl text-[var(--color-ink)] md:text-2xl">
              {section.heading}
            </h2>
            <div className="mt-3 max-w-3xl space-y-3 text-sm leading-7 text-[var(--color-muted)]">
              {section.body.map((paragraph) => (
                <p key={paragraph}>{paragraph}</p>
              ))}
            </div>
          </section>
        ))}
      </div>

      <div className="flex flex-col gap-3 border-t border-[var(--color-line)] pt-8 sm:flex-row">
        <Link
          href={supportHref}
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition-colors hover:bg-[var(--color-green-2)]"
        >
          {supportLabel}
        </Link>
        <Link
          href="/profile"
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-gold)]"
        >
          View Orders
        </Link>
      </div>
    </PageShell>
  );
}
