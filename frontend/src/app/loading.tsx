export default function GlobalLoading() {
  return (
    <main className="min-h-screen bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] px-4 py-10 text-[var(--foreground)] sm:px-6">
      <section className="mx-auto flex w-full max-w-2xl flex-col items-center rounded-[28px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.85),rgba(255,255,255,0.72))] p-8 text-center shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-10">
        <div className="h-10 w-10 animate-spin rounded-full border-2 border-[#0F3D2E]/20 border-t-[#0F3D2E]" />
        <h1 className="mt-5 font-display text-2xl text-[#0F3D2E] sm:text-3xl">
          Loading your Sudatta&apos;s experience
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          Please wait a moment while we prepare this page.
        </p>
      </section>
    </main>
  );
}
