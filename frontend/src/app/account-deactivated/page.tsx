"use client";

import { useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { signOut } from "next-auth/react";
import { PageShell, SectionHeader } from "@/components/ui/page-shell";

const SUPPORT_EMAIL = "sudattasdesignerboutique@gmail.com";

type AccountStatusResponse = { accountDeactivated: boolean | null };

export default function AccountDeactivatedPage() {
  const router = useRouter();
  const [checking, setChecking] = useState(false);

  const refreshStatus = useCallback(async () => {
    setChecking(true);
    try {
      const res = await fetch("/api/account/status", { cache: "no-store" });
      const body = (await res.json().catch(() => ({}))) as AccountStatusResponse;
      // Only leave this page on a confirmed "not deactivated" — `null` (couldn't check) or a
      // request failure both mean "still don't know," never "assume reactivated."
      if (body.accountDeactivated === false) {
        router.replace("/");
      }
    } catch {
      // Network hiccup — stay put, the manual button lets them retry.
    } finally {
      setChecking(false);
    }
  }, [router]);

  useEffect(() => {
    // The page could be stale the moment it loads — e.g. an admin reactivated the account
    // between the redirect firing and this page rendering — so check once immediately rather
    // than only offering the manual button.
    void refreshStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <PageShell containerClassName="py-8 md:py-10">
      <SectionHeader
        label="Account Status"
        title="This account has been deactivated."
        description="You've been signed out of everything except this page. If you believe this is a mistake, contact support and we'll help sort it out."
        className="max-w-4xl"
      />

      <div className="mt-8 rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-5 md:p-6">
        <h2 className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--color-accent-gold)]">
          Contact support
        </h2>
        <p className="mt-3 text-sm leading-7 text-[var(--color-muted)]">
          Email{" "}
          <a
            className="font-semibold text-[var(--color-green)] hover:text-[var(--color-green-2)]"
            href={`mailto:${SUPPORT_EMAIL}`}
          >
            {SUPPORT_EMAIL}
          </a>{" "}
          and mention the email address on this account so we can look into it.
        </p>
      </div>

      <div className="mt-8 flex flex-col gap-3 border-t border-[var(--color-line)] pt-8 sm:flex-row">
        <button
          type="button"
          onClick={() => void refreshStatus()}
          disabled={checking}
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-gold)] disabled:opacity-60"
        >
          {checking ? "Checking..." : "Already reactivated? Refresh"}
        </button>
        <button
          type="button"
          onClick={() => void signOut({ callbackUrl: "/" })}
          className="inline-flex h-11 items-center justify-center rounded-[var(--radius-md)] border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white transition-colors hover:bg-[var(--color-green-2)]"
        >
          Sign Out
        </button>
      </div>
    </PageShell>
  );
}
