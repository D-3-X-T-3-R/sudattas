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
import type { GraphqlErrorLike } from "@/lib/graphql-error-types";

const GRAPHQL_URL = publicGraphqlUrl();

/**
 * The backend serializes `extensions: {code, grpc_code}` on every GraphQL error (see
 * backend/graphql/src/resolvers/error.rs IntoFieldError) — `code` mirrors gRPC status names
 * and should be preferred over substring-matching `message`. Defined locally (rather than
 * imported from graphqlClient.ts) since this module runs server-side and graphqlClient.ts
 * pulls in browser-only modules (authStore/session) at import time. Both classes implement
 * GraphqlErrorLike so an `instanceof` check isn't required to read `code`/`grpcCode` generically.
 */
export class GraphqlRequestError extends Error implements GraphqlErrorLike {
  code?: string;
  grpcCode?: number;

  constructor(message: string, code?: string, grpcCode?: number) {
    super(message);
    this.name = "GraphqlRequestError";
    this.code = code;
    this.grpcCode = grpcCode;
  }
}

let warnedMissingOrigin = false;

function sessionFetchHeaders(sessionId: string): Record<string, string> {
  const h: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Session-Id": sessionId,
  };
  const origin = configuredStorefrontOrigin();
  if (!origin) {
    // If the backend's ALLOWED_ORIGINS CSRF check is configured but STOREFRONT_ORIGIN /
    // NEXT_PUBLIC_SITE_URL isn't, every session-authenticated call through this client will
    // 403 with no Origin header to explain why. Warn once (not per-request) so a missing
    // config doesn't fail silently.
    if (!warnedMissingOrigin) {
      warnedMissingOrigin = true;
      console.warn(
        "[graphqlWithSession] STOREFRONT_ORIGIN/NEXT_PUBLIC_SITE_URL not configured — " +
          "requests will omit Origin, which will 403 if the backend's ALLOWED_ORIGINS CSRF check is enabled."
      );
    }
    return h;
  }
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
      const json = JSON.parse(text) as {
        message?: string;
        errors?: Array<{ message?: string; extensions?: { code?: string; grpc_code?: number } }>;
      };
      const firstError = json.errors?.[0];
      throw new GraphqlRequestError(
        json.message || firstError?.message || `HTTP ${res.status}`,
        firstError?.extensions?.code,
        firstError?.extensions?.grpc_code
      );
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text || `HTTP ${res.status}`);
      throw e;
    }
  }

  const json = JSON.parse(text) as {
    data?: T;
    errors?: Array<{ message: string; extensions?: { code?: string; grpc_code?: number } }>;
  };
  if (json.errors?.length) {
    const firstError = json.errors[0];
    throw new GraphqlRequestError(
      json.errors.map((e) => e.message).join("; "),
      firstError.extensions?.code,
      firstError.extensions?.grpc_code
    );
  }
  return json.data as T;
}

