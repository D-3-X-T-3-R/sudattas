/**
 * GraphQL client that uses only X-Session-Id (guest session).
 * Used by the storefront to load products without requiring admin credentials.
 *
 * When Next.js API routes call this from the server, the GraphQL service may enforce
 * CSRF for session auth (ALLOWED_ORIGINS). Set STOREFRONT_ORIGIN or NEXT_PUBLIC_SITE_URL
 * to your real site origin (e.g. http://localhost:3000) so Origin can be sent.
 */

import { fetchWithResilience } from "@/lib/network-resilience";
import { configuredStorefrontOrigin, publicGraphqlUrl } from "@/lib/env/public";

const GRAPHQL_URL = publicGraphqlUrl();

function sessionFetchHeaders(sessionId: string): Record<string, string> {
  const h: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Session-Id": sessionId,
  };
  const origin = configuredStorefrontOrigin();
  if (!origin) return h;
  h.Origin = origin;
  return h;
}

export async function gqlWithSession<T = unknown>(
  sessionId: string,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<T> {
  const res = await fetchWithResilience(
    GRAPHQL_URL,
    {
      method: "POST",
      headers: {
        ...sessionFetchHeaders(sessionId),
        ...extraHeaders,
      },
      body: JSON.stringify({ query, variables }),
    },
    { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
  );
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

