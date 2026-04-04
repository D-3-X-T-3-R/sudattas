import { NextResponse } from "next/server";
import {
  fetchCategoriesWithSession,
  fetchOccasionsWithSession,
  fetchProductMoodsWithSession,
  fetchProductsListWithSession,
  fetchShopHighlightMoodsWithSession,
} from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { forwardedIpHeadersFromRequest } from "@/lib/forwarded-ip";

type MoodSummary = { moodId: string; moodName: string; thumbnailUrl?: string };
type CategorySummary = { categoryId: string; name: string; thumbnailUrl?: string };

function normalizeMoodRows(rows: unknown): MoodSummary[] {
  const list = Array.isArray(rows) ? rows : [];
  const out: MoodSummary[] = [];
  for (const raw of list) {
    if (!raw || typeof raw !== "object") continue;
    const m = raw as Record<string, unknown>;
    const moodId = String(m.moodId ?? m.mood_id ?? "").trim();
    const moodName = String(m.moodName ?? m.mood_name ?? "").trim();
    if (!moodId && !moodName) continue;
    out.push({ moodId, moodName });
  }
  return out;
}

const THUMB_CACHE_TTL_MS = 10 * 60 * 1000; // 10 min – thumbnails barely change

/** Simple module-level TTL cache so repeated page loads don't refetch thumbnails. */
type CacheEntry = { moodThumbs: Record<string, string>; categoryThumbs: Record<string, string>; expiresAt: number };
const thumbCache = new Map<string, CacheEntry>();
const thumbInflight = new Map<string, Promise<{ moodThumbs: Record<string, string>; categoryThumbs: Record<string, string> }>>();

// Route-level cache for the full filters response
type FilterPayload = { categories: CategorySummary[]; occasions: { occasionId: string; occasionName: string }[]; moods: MoodSummary[]; error: string | null };
type FilterCacheEntry = { ts: number; payload: FilterPayload };
const filterCache = new Map<string, FilterCacheEntry>();
const filterInflight = new Map<string, Promise<FilterPayload>>();
const FILTER_CACHE_TTL_MS = 2 * 60 * 1000; // 2 min

async function fetchThumbnails(
  sessionId: string,
  moods: { moodId: string }[],
  categories: { categoryId: string }[],
  extraHeaders: Record<string, string>
): Promise<{ moodThumbs: Record<string, string>; categoryThumbs: Record<string, string> }> {
  // Thumbnails are public catalog data — key on content shape, not on who's asking.
  const cacheKey = `${moods.map((m) => m.moodId).join(",")}:${categories.map((c) => c.categoryId).join(",")}`;
  const cached = thumbCache.get(cacheKey);
  if (cached && cached.expiresAt > Date.now()) return cached;

  // Inflight dedup — don't run parallel identical thumbnail fetches
  const existing = thumbInflight.get(cacheKey);
  if (existing) return existing;

  const loader = (async () => {
    const moodThumbs: Record<string, string> = {};
    const categoryThumbs: Record<string, string> = {};

    // One bulk fetch derives category thumbnails (products have categoryId).
    try {
      const allProducts = await fetchProductsListWithSession(
        sessionId,
        { limit: "200" },
        extraHeaders
      );
      for (const c of categories) {
        const match = allProducts.find((p) => p.categoryId === c.categoryId);
        const thumb = match?.images?.[0]?.thumbnailUrl;
        if (thumb) categoryThumbs[c.categoryId] = thumb;
      }
    } catch { /* skip category thumbnails on error */ }

    // Mood thumbnails require a per-mood filtered call — products don't carry moodIds.
    // These run sequentially to stay gentle on the backend, and the result is cached
    // for THUMB_CACHE_TTL_MS (10 min) so this block fires at most once per 10 minutes.
    for (const m of moods) {
      try {
        const products = await fetchProductsListWithSession(
          sessionId,
          { moodId: m.moodId, limit: "1" },
          extraHeaders
        );
        const thumb = products[0]?.images?.[0]?.thumbnailUrl;
        if (thumb) moodThumbs[m.moodId] = thumb;
      } catch { /* skip this mood's thumbnail on error */ }
    }

    const entry: CacheEntry = { moodThumbs, categoryThumbs, expiresAt: Date.now() + THUMB_CACHE_TTL_MS };
    thumbCache.set(cacheKey, entry);
    return { moodThumbs, categoryThumbs };
  })().finally(() => thumbInflight.delete(cacheKey));

  thumbInflight.set(cacheKey, loader);
  return loader;
}

const HIGHLIGHT = { recentProductLimit: 100, maxMoods: 12 } as const;

export async function GET(request: Request) {
  const forwardedHeaders = forwardedIpHeadersFromRequest(request);
  const headerSessionId = request.headers.get("x-session-id")?.trim() || null;
  let sessionId = headerSessionId;
  if (!sessionId) {
    sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  }
  if (!sessionId) {
    return NextResponse.json({
      categories: [],
      occasions: [],
      moods: [],
      error: "Guest session unavailable for storefront filters",
    });
  }
  // Catalog filters are the same for every user — use a single shared cache key.
  const cacheKey = "global";
  const now = Date.now();

  const cached = filterCache.get(cacheKey);
  if (cached && now - cached.ts < FILTER_CACHE_TTL_MS) {
    return NextResponse.json(cached.payload, { headers: { "Cache-Control": "public, s-maxage=120, stale-while-revalidate=60" } });
  }

  const pending = filterInflight.get(cacheKey);
  if (pending) {
    const payload = await pending;
    return NextResponse.json(payload, { headers: { "Cache-Control": "public, s-maxage=120, stale-while-revalidate=60" } });
  }

  const loader = (async (): Promise<FilterPayload> => {
    const recovered = await withRecoveredGuestSession(
      sessionId,
      forwardedHeaders,
      async (activeSessionId) => {
        const [categories, occasions, moodsRes] = await Promise.all([
          fetchCategoriesWithSession(activeSessionId, forwardedHeaders),
          fetchOccasionsWithSession(activeSessionId, forwardedHeaders),
          fetchShopHighlightMoodsWithSession(activeSessionId, HIGHLIGHT, forwardedHeaders),
        ]);

        let moods = normalizeMoodRows(moodsRes as unknown[]);
        if (moods.length === 0) {
          const fallbackRaw = await fetchProductMoodsWithSession(activeSessionId, forwardedHeaders);
          moods = normalizeMoodRows(fallbackRaw as unknown[]).sort(
            (a, b) => (parseInt(b.moodId, 10) || 0) - (parseInt(a.moodId, 10) || 0)
          );
          moods = moods.slice(0, HIGHLIGHT.maxMoods);
        }

        const rawCategories = categories.map((c) => ({ categoryId: c.categoryId, name: c.name }));
        const { moodThumbs, categoryThumbs } = await fetchThumbnails(
          activeSessionId,
          moods,
          rawCategories,
          forwardedHeaders
        );

        return {
          categories: rawCategories.map((c) => ({
            ...c,
            thumbnailUrl: categoryThumbs[c.categoryId],
          })),
          occasions: occasions.map((o) => ({
            occasionId: o.occasionId,
            occasionName: o.occasionName,
          })),
          moods: moods.map((m) => ({ ...m, thumbnailUrl: moodThumbs[m.moodId] })),
          error: null,
        } as FilterPayload;
      }
    );

    const payload = recovered.value;
    filterCache.set(cacheKey, { ts: Date.now(), payload });
    return payload;
  })()
    .catch((e): FilterPayload => {
      const message = e instanceof Error ? e.message : "Failed to load filters";
      console.error("API storefront-filters:", e);
      return { categories: [], occasions: [], moods: [], error: message };
    })
    .finally(() => filterInflight.delete(cacheKey));

  filterInflight.set(cacheKey, loader);
  const payload = await loader;
  if (payload.error == null) {
    return NextResponse.json(payload, { headers: { "Cache-Control": "public, s-maxage=120, stale-while-revalidate=60" } });
  }
  return NextResponse.json(payload);
}
