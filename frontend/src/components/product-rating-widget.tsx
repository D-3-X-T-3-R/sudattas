"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useSession } from "next-auth/react";
import { Star } from "lucide-react";
import { Rating } from "@/components/rating";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { ApiEnvelopeError, fetchApiEnvelope } from "@/lib/api-envelope";
import { cn } from "@/lib/utils";

const STAR_VALUES = [1, 2, 3, 4, 5] as const;

interface ProductRatingWidgetProps {
  productId: string;
  /** Server-computed average/count (ceil-rounded) at page-load time or after a `router.refresh()`. */
  initialAverage: number;
  initialCount: number;
}

/**
 * Star-only rating (1-5). There is no written review text at this time by design — the backend
 * still accepts a `comment` field, but this widget never collects one.
 *
 * The displayed average always mirrors `initialAverage`/`initialCount` as passed down from the
 * server component's `productRatingSummary` fetch — we deliberately don't try to recompute the
 * average client-side after a submit, because the backend only ever exposes the already
 * ceil-rounded whole number (e.g. both a raw 3.2 and 3.8 average come back as 4), not the
 * underlying sum, so there's no way to derive an exact incremental update from it. Instead, a
 * successful submit triggers `router.refresh()`, which re-runs the product page's server-side
 * fetch and flows the real, freshly-computed aggregate back down as new props.
 */
export function ProductRatingWidget({
  productId,
  initialAverage,
  initialCount,
}: ProductRatingWidgetProps) {
  const { status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const router = useRouter();
  const [average, setAverage] = useState(initialAverage);
  const [count, setCount] = useState(initialCount);
  const [myRating, setMyRating] = useState<number | null>(null);
  const [hovered, setHovered] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const [loadedMine, setLoadedMine] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Adopt the fresh aggregate once router.refresh() delivers new props after a submit.
  useEffect(() => {
    setAverage(initialAverage);
    setCount(initialCount);
  }, [initialAverage, initialCount]);

  useEffect(() => {
    if (status !== "authenticated") {
      setMyRating(null);
      setLoadedMine(false);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const data = await fetchApiEnvelope<{ rating: number | null }>(
          `/api/account/product-rating?productId=${encodeURIComponent(productId)}`
        );
        if (!cancelled) setMyRating(data.rating);
      } catch {
        // Best-effort: leave the star picker unselected if this fails to load.
      } finally {
        if (!cancelled) setLoadedMine(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [status, productId]);

  async function submitRating(value: number) {
    if (status !== "authenticated") {
      openLogin(`/product/${productId}`);
      return;
    }
    if (saving || value === myRating) return;

    const previousMine = myRating;
    setMyRating(value);
    setSaving(true);
    setError(null);

    try {
      await fetchApiEnvelope<{ rating: number }>("/api/account/product-rating", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ productId, rating: value }),
      });
      // Re-run the server component's productRatingSummary fetch so the displayed average/count
      // reflect the real backend aggregate rather than a guessed local approximation.
      router.refresh();
    } catch (e) {
      setMyRating(previousMine);
      // Surface the backend's actual message for a permanent rejection (e.g. "you can only
      // review products from a delivered order") instead of a generic "try again" — retrying
      // that specific failure can never succeed.
      setError(
        e instanceof ApiEnvelopeError && !e.retryable
          ? e.message
          : "Could not save your rating. Please try again."
      );
    } finally {
      setSaving(false);
    }
  }

  const displayValue = hovered ?? myRating ?? 0;

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center gap-2">
        <Rating value={average} />
        <span className="text-xs text-[var(--color-muted)]">
          {count > 0 ? `${count} rating${count === 1 ? "" : "s"}` : "No ratings yet"}
        </span>
      </div>

      <div
        className="flex flex-wrap items-center gap-x-2 gap-y-1"
        role="group"
        aria-label={myRating ? `Your rating: ${myRating} out of 5 stars` : "Rate this product"}
      >
        <span className="text-xs font-semibold uppercase tracking-[0.1em] text-[var(--color-muted)]">
          {myRating ? "Your rating:" : "Rate this product:"}
        </span>
        <div className="flex items-center gap-0.5" onMouseLeave={() => setHovered(null)}>
          {STAR_VALUES.map((value) => (
            <button
              key={value}
              type="button"
              disabled={saving || (status === "authenticated" && !loadedMine)}
              onClick={() => void submitRating(value)}
              onMouseEnter={() => setHovered(value)}
              aria-label={`Rate ${value} out of 5 stars`}
              aria-pressed={myRating === value}
              className="disabled:cursor-not-allowed disabled:opacity-50"
            >
              <Star
                className={cn(
                  "h-5 w-5 transition-colors",
                  value <= displayValue
                    ? "text-[var(--color-gold)]"
                    : "text-[#CFC7B8] hover:text-[var(--color-gold)]"
                )}
                fill={value <= displayValue ? "currentColor" : "none"}
              />
            </button>
          ))}
        </div>
        {error ? <span className="text-xs text-red-600">{error}</span> : null}
      </div>
    </div>
  );
}
