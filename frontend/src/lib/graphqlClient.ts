/**
 * GraphQL client for storefront/customer interactions.
 * Auth order: logged-in session token > guest session from localStorage.
 */

import { getAccessToken } from "@/lib/authStore";
import {
  getGuestSessionId,
  ensureGuestSession,
  refreshGuestSession,
} from "@/lib/session";
import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";
import { publicGraphqlUrl } from "@/lib/env/public";

const GRAPHQL_URL = publicGraphqlUrl();

function getAuthHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const sessionToken = getAccessToken();
  const guestSessionId = getGuestSessionId();
  const token = sessionToken;
  if (token) {
    headers["Authorization"] = token.startsWith("Bearer ")
      ? token
      : `Bearer ${token}`;
  } else if (guestSessionId) {
    headers["X-Session-Id"] = guestSessionId;
  }
  return headers;
}

function hasAuth(): boolean {
  const token = getAccessToken();
  return !!(
    token ||
    getGuestSessionId()
  );
}

function usedGuestSession(): boolean {
  const token = getAccessToken();
  return (
    !token &&
    !!getGuestSessionId()
  );
}

export async function gql<T = unknown>(
  query: string,
  variables: Record<string, unknown> = {},
  retried = false
): Promise<T> {
  if (typeof window !== "undefined" && !hasAuth()) {
    await ensureGuestSession();
    if (!hasAuth()) {
      throw new Error(
        "Guest session unavailable. Is the backend running with Redis? (POST /session/guest may return 503.)"
      );
    }
  }

  const payload = { query, variables: variables ?? {} };
  let res: Response;
  try {
    res = await fetchWithResilience(
      GRAPHQL_URL,
      {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify(payload),
      },
      { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
    );
  } catch (error) {
    throw new Error(normalizeNetworkError(error));
  }
  const text = await res.text();

  if (res.status === 401 && typeof window !== "undefined" && usedGuestSession() && !retried) {
    await refreshGuestSession();
    return gql<T>(query, variables, true);
  }

  if (!res.ok) {
    try {
      const json = JSON.parse(text) as {
        message?: string;
        errors?: Array<{ message?: string }>;
      };
      throw new Error(
        json.message || json.errors?.[0]?.message || String(res.status)
      );
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text || String(res.status));
      throw e;
    }
  }

  const json = JSON.parse(text) as { data?: T; errors?: Array<{ message: string }> };
  if (json.errors?.length) {
    throw new Error(json.errors.map((e) => e.message).join("; "));
  }
  return json.data as T;
}
