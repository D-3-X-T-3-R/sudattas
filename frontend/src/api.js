/**
 * GraphQL client for storefront (and Razorpay test).
 * Auth order: Auth0 token (authStore) > REACT_APP_GRAPHQL_TOKEN > REACT_APP_GRAPHQL_SESSION_ID > guest session from localStorage.
 */
import { getAccessToken } from "./authStore";
import { getGuestSessionId, ensureGuestSession, clearGuestSession } from "./session";

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

function getAuthHeaders() {
  const headers = { "Content-Type": "application/json" };
  const auth0Token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_SESSION_ID;
  const guestSessionId = getGuestSessionId();
  const token = auth0Token || envToken;
  if (token)
    headers["Authorization"] = token.startsWith("Bearer ")
      ? token
      : `Bearer ${token}`;
  else if (envSessionId) headers["X-Session-Id"] = envSessionId;
  else if (guestSessionId) headers["X-Session-Id"] = guestSessionId;
  return headers;
}

/** Returns true if we already have some auth (token or session). */
function hasAuth() {
  const token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_SESSION_ID;
  return !!(token || envToken || envSessionId || getGuestSessionId());
}

/** Returns true if we're using guest session (no JWT/env token). */
function usedGuestSession() {
  const token = getAccessToken();
  const envToken =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_TOKEN;
  const envSessionId =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_SESSION_ID;
  return !token && !envToken && !envSessionId && !!getGuestSessionId();
}

export async function gql(query, variables = {}, retried = false) {
  // If we have no auth yet, ensure guest session so backend doesn't return 401
  if (!hasAuth()) {
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

  // 401 with guest session → session may be stale (e.g. Redis restarted); clear and retry once
  if (res.status === 401 && usedGuestSession() && !retried) {
    clearGuestSession();
    await ensureGuestSession();
    return gql(query, variables, true);
  }

  if (!res.ok) {
    try {
      const json = JSON.parse(text);
      throw new Error(json.message || json.errors?.[0]?.message || String(res.status));
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text || String(res.status));
      throw e;
    }
  }
  const json = JSON.parse(text);
  if (json.errors?.length)
    throw new Error(json.errors.map((e) => e.message).join("; "));
  return json.data;
}
