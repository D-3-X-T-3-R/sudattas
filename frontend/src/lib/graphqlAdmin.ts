/**
 * GraphQL client for admin panel. Uses NEXT_PUBLIC_ADMIN_API_KEY (or token/session) for auth.
 */

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

function getAuthHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const token =
    typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_TOKEN;
  const sessionId =
    typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_GRAPHQL_SESSION_ID;
  const adminKey =
    typeof process !== "undefined" &&
    process.env?.NEXT_PUBLIC_ADMIN_API_KEY;
  if (typeof token === "string" && token) {
    headers["Authorization"] = token.startsWith("Bearer ")
      ? token
      : `Bearer ${token}`;
  } else if (typeof sessionId === "string" && sessionId) {
    headers["X-Session-Id"] = sessionId;
  } else if (typeof adminKey === "string" && adminKey) {
    headers["X-Admin-Key"] = adminKey;
  }
  return headers;
}

export async function gqlAdmin<T = unknown>(
  query: string,
  variables: Record<string, unknown> = {}
): Promise<T> {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({ query, variables }),
  });
  const text = await res.text();

  if (res.status === 401) {
    throw new Error(
      "Unauthorized. Set NEXT_PUBLIC_ADMIN_API_KEY in .env to match ADMIN_API_KEY in the backend (or use NEXT_PUBLIC_GRAPHQL_TOKEN / NEXT_PUBLIC_GRAPHQL_SESSION_ID)."
    );
  }
  if (res.status === 403) {
    throw new Error("Forbidden. Request was rejected (e.g. CSRF or origin check).");
  }
  if (!res.ok) {
    try {
      const json = JSON.parse(text) as { message?: string; errors?: Array<{ message?: string }> };
      throw new Error(
        json.message || json.errors?.[0]?.message || `HTTP ${res.status}`
      );
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text || `HTTP ${res.status}`);
      throw e;
    }
  }

  const json = JSON.parse(text) as { data?: T; errors?: Array<{ message: string }> };
  if (json.errors?.length) {
    throw new Error(json.errors.map((e) => e.message).join("; "));
  }
  return json.data as T;
}
