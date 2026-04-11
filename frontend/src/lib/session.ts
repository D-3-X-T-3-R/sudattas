/**
 * Guest session for bag (no login).
 * Persists in localStorage; call ensureGuestSession() early (e.g. App mount).
 */

import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";
import { publicGraphqlUrl } from "@/lib/env/public";

const STORAGE_KEY = "sudattas_guest_session";
let ensureGuestSessionInFlight: Promise<string | null> | null = null;

function getBaseUrl(): string {
  return publicGraphqlUrl().replace(/\/v2\/?$/, "");
}

export function getGuestSessionId(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(STORAGE_KEY);
}

export function setGuestSessionId(sessionId: string): void {
  if (typeof window !== "undefined" && sessionId) {
    window.localStorage.setItem(STORAGE_KEY, sessionId);
  }
}

export function clearGuestSession(): void {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(STORAGE_KEY);
  }
}

export async function refreshGuestSession(): Promise<string | null> {
  clearGuestSession();
  return ensureGuestSession();
}

/**
 * Ensure we have a guest session ID. If none in localStorage, call backend POST /session/guest and store it.
 * Returns the session ID (existing or new), or null if backend is unavailable or Redis is disabled.
 */
export async function ensureGuestSession(): Promise<string | null> {
  const existing = getGuestSessionId();
  if (existing) return existing;
  if (ensureGuestSessionInFlight) return ensureGuestSessionInFlight;

  ensureGuestSessionInFlight = (async () => {
    const base = getBaseUrl();
    try {
      const res = await fetchWithResilience(
        `${base}/session/guest`,
        { method: "POST" },
        { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
      );
      const text = await res.text();
      if (!res.ok) {
        try {
          const err = JSON.parse(text) as { error?: string };
          if (err?.error) console.warn("[session] Guest session failed:", err.error);
        } catch {
          // ignore
        }
        return null;
      }
      const data = JSON.parse(text) as { session_id?: string };
      const sessionId = data?.session_id;
      if (sessionId) {
        setGuestSessionId(sessionId);
        return sessionId;
      }
      console.warn("[session] POST succeeded but no session_id in response");
    } catch (e) {
      console.warn("[session] Guest session request failed:", normalizeNetworkError(e));
    }
    return null;
  })().finally(() => {
    ensureGuestSessionInFlight = null;
  });

  return ensureGuestSessionInFlight;
}
