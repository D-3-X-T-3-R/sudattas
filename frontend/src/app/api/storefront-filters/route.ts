import { NextResponse } from "next/server";
import {
  fetchCategories,
  fetchOccasions,
  fetchShopHighlightMoods,
  searchProductMoods,
} from "@/lib/admin-queries";
import {
  fetchCategoriesWithSession,
  fetchOccasionsWithSession,
  fetchProductMoodsWithSession,
  fetchProductsListWithSession,
  fetchShopHighlightMoodsWithSession,
} from "@/lib/storefront-queries";

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

/** Simple module-level TTL cache so repeated page loads don't refetch thumbnails. */
type CacheEntry = { moodThumbs: Record<string, string>; categoryThumbs: Record<string, string>; expiresAt: number };
const thumbCache = new Map<string, CacheEntry>();
const CACHE_TTL_MS = 5 * 60 * 1000;

async function fetchThumbnails(
  sessionId: string,
  moods: { moodId: string }[],
  categories: { categoryId: string }[]
): Promise<{ moodThumbs: Record<string, string>; categoryThumbs: Record<string, string> }> {
  const cacheKey = `${sessionId}:${moods.map((m) => m.moodId).join(",")}:${categories.map((c) => c.categoryId).join(",")}`;
  const cached = thumbCache.get(cacheKey);
  if (cached && cached.expiresAt > Date.now()) return cached;

  const moodThumbs: Record<string, string> = {};
  const categoryThumbs: Record<string, string> = {};

  // Derive category thumbnails from a single bulk product fetch.
  try {
    const allProducts = await fetchProductsListWithSession(sessionId, { limit: "200" });
    for (const c of categories) {
      const match = allProducts.find((p) => p.categoryId === c.categoryId);
      const thumb = match?.images?.[0]?.thumbnailUrl;
      if (thumb) categoryThumbs[c.categoryId] = thumb;
    }
  } catch { /* skip */ }

  // Mood thumbnails require per-mood calls — fetch sequentially to stay under rate limits.
  for (const m of moods) {
    try {
      const products = await fetchProductsListWithSession(sessionId, { moodId: m.moodId, limit: "1" });
      const thumb = products[0]?.images?.[0]?.thumbnailUrl;
      if (thumb) moodThumbs[m.moodId] = thumb;
    } catch { /* skip */ }
  }

  const entry: CacheEntry = { moodThumbs, categoryThumbs, expiresAt: Date.now() + CACHE_TTL_MS };
  thumbCache.set(cacheKey, entry);
  return entry;
}

const HIGHLIGHT = { recentProductLimit: 100, maxMoods: 12 } as const;

export async function GET(request: Request) {
  const sessionId = request.headers.get("x-session-id")?.trim() || null;

  try {
    const [categories, occasions, moodsRes] = sessionId
      ? await Promise.all([
          fetchCategoriesWithSession(sessionId),
          fetchOccasionsWithSession(sessionId),
          fetchShopHighlightMoodsWithSession(sessionId, HIGHLIGHT),
        ])
      : await Promise.all([
          fetchCategories(),
          fetchOccasions(),
          fetchShopHighlightMoods(HIGHLIGHT),
        ]);

    let moods = normalizeMoodRows(moodsRes as unknown[]);
    if (moods.length === 0) {
      const fallbackRaw = sessionId
        ? await fetchProductMoodsWithSession(sessionId)
        : await searchProductMoods({});
      moods = normalizeMoodRows(fallbackRaw as unknown[]).sort(
        (a, b) => (parseInt(b.moodId, 10) || 0) - (parseInt(a.moodId, 10) || 0)
      );
      moods = moods.slice(0, HIGHLIGHT.maxMoods);
    }

    const rawCategories = categories.map((c) => ({ categoryId: c.categoryId, name: c.name }));

    const { moodThumbs, categoryThumbs } = sessionId
      ? await fetchThumbnails(sessionId, moods, rawCategories)
      : { moodThumbs: {}, categoryThumbs: {} };

    const moodsWithThumbs: MoodSummary[] = moods.map((m) => ({
      ...m,
      thumbnailUrl: moodThumbs[m.moodId],
    }));
    const categoriesWithThumbs: CategorySummary[] = rawCategories.map((c) => ({
      ...c,
      thumbnailUrl: categoryThumbs[c.categoryId],
    }));

    return NextResponse.json(
      {
        categories: categoriesWithThumbs,
        occasions: occasions.map((o) => ({ occasionId: o.occasionId, occasionName: o.occasionName })),
        moods: moodsWithThumbs,
        error: null,
      },
      { headers: { "Cache-Control": "private, max-age=300" } }
    );
  } catch (e) {
    const message = e instanceof Error ? e.message : "Failed to load filters";
    return NextResponse.json(
      { categories: [], occasions: [], moods: [], error: message },
      { status: 200 }
    );
  }
}
