/**
 * GraphQL client for storefront (and Razorpay test).
 * Auth order: Auth0 token (authStore) > NEXT_PUBLIC_GRAPHQL_TOKEN > NEXT_PUBLIC_GRAPHQL_SESSION_ID > guest session from localStorage.
 */

import { getAccessToken } from "@/lib/authStore";
import {
  getGuestSessionId,
  ensureGuestSession,
  clearGuestSession,
} from "@/lib/session";

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

function getAuthHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const auth0Token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_GRAPHQL_SESSION_ID;
  const guestSessionId = getGuestSessionId();
  const token = auth0Token || (typeof envToken === "string" ? envToken : null);
  if (token) {
    headers["Authorization"] = token.startsWith("Bearer ")
      ? token
      : `Bearer ${token}`;
  } else if (typeof envSessionId === "string" && envSessionId) {
    headers["X-Session-Id"] = envSessionId;
  } else if (guestSessionId) {
    headers["X-Session-Id"] = guestSessionId;
  }
  return headers;
}

function hasAuth(): boolean {
  const token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_GRAPHQL_SESSION_ID;
  return !!(
    token ||
    (typeof envToken === "string" && envToken) ||
    (typeof envSessionId === "string" && envSessionId) ||
    getGuestSessionId()
  );
}

function usedGuestSession(): boolean {
  const token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_GRAPHQL_SESSION_ID;
  return (
    !token &&
    !(typeof envToken === "string" && envToken) &&
    !(typeof envSessionId === "string" && envSessionId) &&
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
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify(payload),
  });
  const text = await res.text();

  if (res.status === 401 && typeof window !== "undefined" && usedGuestSession() && !retried) {
    clearGuestSession();
    await ensureGuestSession();
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
