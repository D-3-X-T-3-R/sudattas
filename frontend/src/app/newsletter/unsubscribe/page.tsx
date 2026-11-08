"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { unsubscribeFromNewsletter, NewsletterUnsubscribeError } from "@/lib/newsletter-api";

type Status = "working" | "done" | "error";

function UnsubscribeContent() {
  const searchParams = useSearchParams();
  const subscriberId = searchParams.get("id") ?? "";
  const token = searchParams.get("token") ?? "";

  const [status, setStatus] = useState<Status>("working");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!subscriberId || !token) {
      setStatus("error");
      setError("This unsubscribe link is missing information — please use the link from the email.");
      return;
    }
    let cancelled = false;
    unsubscribeFromNewsletter(subscriberId, token)
      .then(() => {
        if (!cancelled) setStatus("done");
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setStatus("error");
        setError(
          err instanceof NewsletterUnsubscribeError
            ? err.message
            : "This unsubscribe link is invalid or has expired."
        );
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="mx-auto max-w-md px-6 py-24 text-center">
      {status === "working" ? (
        <p className="text-[15px] text-[var(--color-muted)]">Unsubscribing…</p>
      ) : null}
      {status === "done" ? (
        <>
          <h1 className="font-[family-name:var(--font-display)] text-2xl text-[var(--color-ink)]">
            You&rsquo;re unsubscribed
          </h1>
          <p className="mt-3 text-[15px] leading-relaxed text-[var(--color-muted)]">
            You won&rsquo;t receive any more newsletter emails from us. Changed your mind? You can
            always sign up again from the site.
          </p>
        </>
      ) : null}
      {status === "error" ? (
        <>
          <h1 className="font-[family-name:var(--font-display)] text-2xl text-[var(--color-ink)]">
            Couldn&rsquo;t unsubscribe
          </h1>
          <p className="mt-3 text-[15px] leading-relaxed text-[var(--color-muted)]">{error}</p>
        </>
      ) : null}
      <Link href="/" className="mt-8 inline-block text-sm font-medium text-[var(--color-green)] underline">
        Back to Sudatta&rsquo;s
      </Link>
    </div>
  );
}

export default function NewsletterUnsubscribePage() {
  return (
    <Suspense
      fallback={
        <div className="mx-auto max-w-md px-6 py-24 text-center text-sm text-[var(--color-muted)]">
          Loading…
        </div>
      }
    >
      <UnsubscribeContent />
    </Suspense>
  );
}
