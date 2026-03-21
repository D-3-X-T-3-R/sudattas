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

async function attachMoodThumbnails(
  moods: MoodSummary[],
  sessionId: string | null
): Promise<MoodSummary[]> {
  const thumbs = await Promise.all(
    moods.map(async (m) => {
      try {
        const products = sessionId
          ? await fetchProductsListWithSession(sessionId, { moodId: m.moodId, limit: "1" })
          : [];
        const thumb = products[0]?.images?.[0]?.thumbnailUrl ?? undefined;
        return { ...m, thumbnailUrl: thumb };
      } catch {
        return m;
      }
    })
  );
  return thumbs;
}

const HIGHLIGHT = { recentProductLimit: 100, maxMoods: 12 } as const;

/** Categories, occasions, and moods from newest products (distinct mood order from product walk). */
export async function GET(request: Request) {
  const sessionId = request.headers.get("x-session-id")?.trim() || null;
  console.log("[storefront-filters] GET called | sessionId =", sessionId);

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

    console.log("[storefront-filters] moodsRes raw:", JSON.stringify(moodsRes));
    let moods = normalizeMoodRows(moodsRes as unknown[]);
    console.log("[storefront-filters] after normalize:", moods.length, "moods");
    // Highlight needs mood mappings on recent products; if none, show newest mood labels from DB
    if (moods.length === 0) {
      console.log("[storefront-filters] highlight empty — trying fallback searchProductMoods");
      const fallbackRaw = sessionId
        ? await fetchProductMoodsWithSession(sessionId)
        : await searchProductMoods({});
      console.log("[storefront-filters] fallbackRaw:", JSON.stringify(fallbackRaw));
      moods = normalizeMoodRows(fallbackRaw as unknown[]).sort(
        (a, b) => (parseInt(b.moodId, 10) || 0) - (parseInt(a.moodId, 10) || 0)
      );
      moods = moods.slice(0, HIGHLIGHT.maxMoods);
    }

    console.log("[storefront-filters] returning", moods.length, "moods");
    moods = await attachMoodThumbnails(moods, sessionId);
    return NextResponse.json({
      categories: categories.map((c) => ({ categoryId: c.categoryId, name: c.name })),
      occasions: occasions.map((o) => ({ occasionId: o.occasionId, occasionName: o.occasionName })),
      moods,
      error: null,
    });
  } catch (e) {
    const message = e instanceof Error ? e.message : "Failed to load filters";
    console.error("[storefront-filters] CAUGHT ERROR:", message, e);
    return NextResponse.json(
      { categories: [], occasions: [], moods: [], error: message },
      { status: 200 }
    );
  }
}
