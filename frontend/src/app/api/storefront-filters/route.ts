import { NextResponse } from "next/server";
import {
  fetchCategoriesWithSession,
  fetchOccasionsWithSession,
} from "@/lib/storefront-queries";

export async function GET(request: Request) {
  const sessionId = request.headers.get("x-session-id")?.trim() || null;
  if (!sessionId) {
    return NextResponse.json(
      { categories: [], occasions: [], error: "Missing X-Session-Id" },
      { status: 200 }
    );
  }
  try {
    const [categories, occasions] = await Promise.all([
      fetchCategoriesWithSession(sessionId),
      fetchOccasionsWithSession(sessionId),
    ]);
    return NextResponse.json({
      categories: categories.map((c) => ({ categoryId: c.categoryId, name: c.name })),
      occasions: occasions.map((o) => ({ occasionId: o.occasionId, occasionName: o.occasionName })),
      error: null,
    });
  } catch (e) {
    const message = e instanceof Error ? e.message : "Failed to load filters";
    return NextResponse.json(
      { categories: [], occasions: [], error: message },
      { status: 200 }
    );
  }
}
