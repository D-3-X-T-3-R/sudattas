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

      <div className="mt-8 space-y-4">
        {sections.map((section) => (
          <section key={section.heading} className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] sm:p-6">
            <h2 className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
              {section.heading}
            </h2>
            <div className="mt-3 space-y-3 text-sm leading-7 text-[var(--color-muted)]">
              {section.body.map((paragraph) => (
                <p key={paragraph}>{paragraph}</p>
              ))}
            </div>
          </section>
        ))}
      </div>

      <div className="mt-8 flex flex-col gap-3 sm:flex-row">
        <Link
          href={supportHref}
          className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
        >
          {supportLabel}
        </Link>
        <Link
          href="/profile"
          className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-line)] bg-white px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]"
        >
          View Orders
        </Link>
      </div>
    </PageShell>
  );
}
