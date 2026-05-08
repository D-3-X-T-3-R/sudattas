import { PageShell } from "@/components/ui/page-shell";

export default function GlobalLoading() {
  return (
    <PageShell containerClassName="flex min-h-[72vh] items-center justify-center py-10">
      <section className="w-full max-w-xl rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-8 text-center shadow-[var(--shadow-subtle)]">
        <div className="mx-auto h-10 w-10 animate-spin rounded-full border-2 border-[var(--color-line)] border-t-[var(--color-green)]" />
        <h1 className="mt-5 font-display text-3xl text-[var(--color-ink)]">
          Loading your Sudatta&apos;s experience
        </h1>
        <p className="mt-3 text-sm text-[var(--color-muted)]">
          Please wait a moment while we prepare this page.
        </p>
      </section>
    </PageShell>
  );
}
