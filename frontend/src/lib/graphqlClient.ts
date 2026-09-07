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
import type { GraphqlErrorLike } from "@/lib/graphql-error-types";

const GRAPHQL_URL = publicGraphqlUrl();

type GraphqlErrorPayload = {
  message?: string;
  errors?: Array<{ message?: string; extensions?: { code?: string; grpc_code?: number } }>;
};

/**
 * The backend serializes `extensions: {code, grpc_code}` on every GraphQL error (see
 * backend/graphql/src/resolvers/error.rs IntoFieldError) — `code` mirrors gRPC status names
 * (e.g. "FailedPrecondition", "NotFound") and should be preferred over substring-matching
 * `message`. Callers that need to distinguish error classes (e.g. permission vs. validation)
 * should catch this instead of a plain `Error`.
 *
 * A near-identical class also exists in graphqlWithSession.ts rather than being shared here,
 * because this module (browser-side) pulls in authStore/session, which the server-side
 * graphqlWithSession.ts must not import — both implement GraphqlErrorLike instead.
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

function randomRequestId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `gql-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
}

function operationNameFromQuery(query: string): string | null {
  return query.match(/\b(?:query|mutation)\s+([A-Za-z0-9_]+)/)?.[1] ?? null;
}

function getAuthHeaders(requestId: string, operationName: string | null): Record<string, string> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Request-Id": requestId,
    "X-Client-Action": operationName ? `GraphQL ${operationName}` : "GraphQL anonymous",
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

function extractErrorMessage(text: string): string {
  try {
    const json = JSON.parse(text) as GraphqlErrorPayload;
    return (json.message || json.errors?.[0]?.message || "").trim();
  } catch {
    return text.trim();
  }
}

function looksLikeSessionAuthFailure(status: number, text: string): boolean {
  if (status === 401) return true;
  const message = extractErrorMessage(text).toLowerCase();
  return (
    message.includes("unauthorized") ||
    message.includes("session invalid") ||
    message.includes("session not found") ||
    message.includes("session not found or expired") ||
    message.includes("expired") ||
    message.includes("no valid authentication credentials")
  );
}

function withCurrentGuestSessionVariables(value: unknown, guestSessionId: string): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => withCurrentGuestSessionVariables(item, guestSessionId));
  }
  if (!value || typeof value !== "object") {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
      key,
      key === "sessionId"
        ? guestSessionId
        : withCurrentGuestSessionVariables(entry, guestSessionId),
    ])
  );
}

function variablesForRequest(variables: Record<string, unknown>): Record<string, unknown> {
  const guestSessionId =
    typeof window !== "undefined" && !getAccessToken() ? getGuestSessionId() : null;
  if (!guestSessionId) return variables;
  return withCurrentGuestSessionVariables(variables, guestSessionId) as Record<string, unknown>;
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

  const operationName = operationNameFromQuery(query);
  const requestId = randomRequestId();
  if (typeof window !== "undefined" && process.env.NODE_ENV !== "production") {
    console.info("[graphqlClient] sending request", {
      operationName,
      requestId,
      retried,
      authMode: getAccessToken() ? "bearer" : getGuestSessionId() ? "guest-session" : "none",
    });
  }
  const payload = {
    query,
    variables: variablesForRequest(variables ?? {}),
    operationName,
  };
  let res: Response;
  try {
    res = await fetchWithResilience(
      GRAPHQL_URL,
      {
        method: "POST",
        headers: getAuthHeaders(requestId, operationName),
        body: JSON.stringify(payload),
      },
      { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
    );
  } catch (error) {
    throw new Error(normalizeNetworkError(error));
  }
  const text = await res.text();

  if (typeof window !== "undefined" && usedGuestSession() && !retried) {
    // Recover stale guest sessions once before surfacing an error.
    if (looksLikeSessionAuthFailure(res.status, text)) {
      await refreshGuestSession();
      return gql<T>(query, variables, true);
    }
  }

  if (!res.ok) {
    try {
      const json = JSON.parse(text) as GraphqlErrorPayload;
      const firstError = json.errors?.[0];
      throw new GraphqlRequestError(
        json.message || firstError?.message || String(res.status),
        firstError?.extensions?.code,
        firstError?.extensions?.grpc_code
      );
    } catch (e) {
      if (e instanceof SyntaxError) throw new Error(text || String(res.status));
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
