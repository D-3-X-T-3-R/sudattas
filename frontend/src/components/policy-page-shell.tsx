import Link from "next/link";

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
    <main className="min-h-screen bg-[linear-gradient(180deg,#F8F3EC_0%,#F4EBDD_50%,#F7F3EB_100%)] px-4 py-10 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto max-w-4xl rounded-[32px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.9),rgba(255,255,255,0.76))] p-6 shadow-[0_30px_90px_rgba(15,61,46,0.08)] backdrop-blur-xl sm:p-10">
        <p className="inline-flex rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.24em] text-[#A37D34]">
          {eyebrow}
        </p>
        <h1 className="mt-5 font-display text-3xl leading-tight text-[#0F3D2E] sm:text-5xl">
          {title}
        </h1>
        <p className="mt-4 max-w-3xl text-sm leading-7 text-[#615A50] sm:text-base">
          {intro}
        </p>
        <div className="mt-10 space-y-8">
          {sections.map((section) => (
            <section key={section.heading} className="rounded-[24px] border border-[#0F3D2E]/8 bg-white/65 p-5 sm:p-6">
              <h2 className="text-sm font-semibold uppercase tracking-[0.18em] text-[#0F3D2E]">
                {section.heading}
              </h2>
              <div className="mt-4 space-y-3 text-sm leading-7 text-[#615A50]">
                {section.body.map((paragraph) => (
                  <p key={paragraph}>{paragraph}</p>
                ))}
              </div>
            </section>
          ))}
        </div>
        <div className="mt-10 flex flex-col gap-3 sm:flex-row">
          <Link
            href={supportHref}
            className="inline-flex h-12 items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#F6F3EA] transition hover:bg-[#0C3126]"
          >
            {supportLabel}
          </Link>
          <Link
            href="/profile"
            className="inline-flex h-12 items-center justify-center rounded-full border border-[#0F3D2E]/20 bg-white px-6 text-xs font-semibold uppercase tracking-[0.18em] text-[#0F3D2E] transition hover:border-[#0F3D2E]/40"
          >
            View Orders
          </Link>
        </div>
      </section>
    </main>
  );
}
