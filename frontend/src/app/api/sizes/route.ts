import { NextResponse } from "next/server";
import { fetchSizesWithSession } from "@/lib/storefront-queries";

export async function GET(request: Request) {
  const sessionId = request.headers.get("x-session-id")?.trim() || null;
  if (!sessionId) {
    return NextResponse.json({ sizes: [] });
  }
  try {
    const sizes = await fetchSizesWithSession(sessionId);
    return NextResponse.json({ sizes });
  } catch {
    return NextResponse.json({ sizes: [] });
  }
}
