/**
 * GraphQL client for storefront (and Razorpay test).
 * Set REACT_APP_GRAPHQL_URL; for auth set REACT_APP_GRAPHQL_SESSION_ID or REACT_APP_GRAPHQL_TOKEN.
 */
const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

function getAuthHeaders() {
  const headers = { "Content-Type": "application/json" };
  const token =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_TOKEN;
  const sessionId =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_SESSION_ID;
  if (token)
    headers["Authorization"] = token.startsWith("Bearer ")
      ? token
      : `Bearer ${token}`;
  else if (sessionId) headers["X-Session-Id"] = sessionId;
  return headers;
}

export async function gql(query, variables = {}) {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({ query, variables }),
  });
  const text = await res.text();
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
