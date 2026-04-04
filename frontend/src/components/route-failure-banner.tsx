"use client";

import type { RouteFailureUi } from "@/lib/route-state";

type RouteFailureBannerProps = {
  failure: RouteFailureUi;
  onRetry?: () => void;
  onSignIn?: () => void;
  className?: string;
};

function variantClasses(kind: RouteFailureUi["kind"]): string {
  if (kind === "retryable") return "border-amber-200 bg-amber-50 text-amber-900";
  if (kind === "unauthorized" || kind === "stale") {
    return "border-red-200 bg-red-50 text-red-900";
  }
  return "border-red-200 bg-red-50 text-red-900";
}

export function RouteFailureBanner({
  failure,
  onRetry,
  onSignIn,
  className,
}: RouteFailureBannerProps) {
  const showRetry = failure.kind === "retryable" && Boolean(onRetry);
  const showSignIn =
    (failure.kind === "unauthorized" || failure.kind === "stale") &&
    Boolean(onSignIn);

  return (
    <div
      role="alert"
      className={`rounded-xl border px-4 py-3 text-sm ${variantClasses(failure.kind)} ${className ?? ""}`}
    >
      <p className="font-semibold">{failure.title}</p>
      <p className="mt-1">{failure.message}</p>
      {(showRetry || showSignIn) && (
        <div className="mt-3 flex flex-wrap gap-2">
          {showRetry && (
            <button
              type="button"
              onClick={onRetry}
              className="rounded-full border border-current px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] opacity-90 transition hover:opacity-100"
            >
              Retry
            </button>
          )}
          {showSignIn && (
            <button
              type="button"
              onClick={onSignIn}
              className="rounded-full border border-current px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] opacity-90 transition hover:opacity-100"
            >
              Sign in again
            </button>
          )}
        </div>
      )}
    </div>
  );
}
