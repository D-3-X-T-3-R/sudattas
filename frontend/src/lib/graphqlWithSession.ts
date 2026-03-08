/**
 * GraphQL client that uses only X-Session-Id (guest session).
 * Used by the storefront to load products without requiring admin credentials.
 */

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

export async function gqlWithSession<T = unknown>(
  sessionId: string,
  query: string,
  variables: Record<string, unknown> = {}
): Promise<T> {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Session-Id": sessionId,
    },
    body: JSON.stringify({ query, variables }),
  });
  const text = await res.text();

  if (res.status === 401) {
    throw new Error("Session invalid or expired");
  }
  if (res.status === 403) {
    throw new Error("Forbidden (e.g. CSRF or origin check)");
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
