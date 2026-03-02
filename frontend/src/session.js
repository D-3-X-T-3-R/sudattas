/**
 * Guest session for cart (no login).
 * Persists in localStorage; call ensureGuestSession() early (e.g. App mount).
 */

const STORAGE_KEY = "sudattas_guest_session";

const getBaseUrl = () => {
  const url =
    (typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_URL) ||
    "http://localhost:8080/v2";
  return url.replace(/\/v2\/?$/, "");
};

export function getGuestSessionId() {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(STORAGE_KEY);
}

export function setGuestSessionId(sessionId) {
  if (typeof window !== "undefined" && sessionId) {
    window.localStorage.setItem(STORAGE_KEY, sessionId);
  }
}

export function clearGuestSession() {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(STORAGE_KEY);
  }
}

/**
 * Ensure we have a guest session ID. If none in localStorage, call backend POST /session/guest and store it.
 * Returns the session ID (existing or new), or null if backend is unavailable or Redis is disabled.
 */
export async function ensureGuestSession() {
  const existing = getGuestSessionId();
  if (existing) return existing;

  const base = getBaseUrl();
  try {
    const res = await fetch(`${base}/session/guest`, { method: "POST" });
    const text = await res.text();
    if (!res.ok) {
      // Backend returns JSON { error } on 503
      try {
        const err = JSON.parse(text);
        if (err?.error) console.warn("Guest session failed:", err.error);
      } catch (_) {}
      return null;
    }
    const data = JSON.parse(text);
    const sessionId = data?.session_id;
    if (sessionId) {
      setGuestSessionId(sessionId);
      return sessionId;
    }
  } catch (e) {
    console.warn("Guest session request failed:", e?.message || e);
  }
  return null;
}
