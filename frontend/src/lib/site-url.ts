import { publicEnv } from "@/lib/env/public";

export function siteUrl(): string {
  const raw = publicEnv.NEXT_PUBLIC_SITE_URL?.trim();
  if (!raw) return "https://www.sudattas.com";
  return raw.endsWith("/") ? raw.slice(0, -1) : raw;
}
