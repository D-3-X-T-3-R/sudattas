/**
 * GraphQL client that uses only X-Session-Id (guest session).
 * Used by the storefront to load products without requiring admin credentials.
 *
 * When Next.js API routes call this from the server, the GraphQL service may enforce
 * CSRF for session auth (ALLOWED_ORIGINS). Set STOREFRONT_ORIGIN or NEXT_PUBLIC_SITE_URL
 * to your real site origin (e.g. http://localhost:3000) so Origin can be sent.
 */

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

const MAX_RETRIES_429 = 2;
const RETRY_BASE_MS = 150;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function jitteredBackoffMs(attempt: number): number {
  const exp = RETRY_BASE_MS * 2 ** Math.max(0, attempt - 1);
  const jitter = Math.floor(Math.random() * 120);
  return exp + jitter;
}

function sessionFetchHeaders(sessionId: string): Record<string, string> {
  const h: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Session-Id": sessionId,
  };
  const raw =
    (typeof process !== "undefined" && process.env.STOREFRONT_ORIGIN) ||
    (typeof process !== "undefined" && process.env.NEXT_PUBLIC_SITE_URL) ||
    (typeof process !== "undefined" &&
      process.env.VERCEL_URL &&
      `https://${process.env.VERCEL_URL}`);
  if (!raw) return h;
  try {
    const url = raw.startsWith("http://") || raw.startsWith("https://") ? raw : `https://${raw}`;
    h.Origin = new URL(url).origin;
  } catch {
    /* ignore bad env */
  }
  return h;
}

export async function gqlWithSession<T = unknown>(
  sessionId: string,
  query: string,
  variables: Record<string, unknown> = {},
  retryCount = 0
): Promise<T> {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: sessionFetchHeaders(sessionId),
    body: JSON.stringify({ query, variables }),
  });
  const text = await res.text();

  if (res.status === 429 && retryCount < MAX_RETRIES_429) {
    await sleep(jitteredBackoffMs(retryCount + 1));
    return gqlWithSession<T>(sessionId, query, variables, retryCount + 1);
  }

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
