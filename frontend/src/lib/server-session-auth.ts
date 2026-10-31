import "server-only";

import { getServerSession } from "next-auth";
import { headers as nextHeaders } from "next/headers";
import { authOptions } from "@/lib/auth";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";
import { serverEnv, serverGraphqlUrl } from "@/lib/env/server";

/**
 * The backend serializes `extensions: {code, grpc_code}` on every GraphQL error (see
 * backend/graphql/src/resolvers/error.rs IntoFieldError) — this was already present on the wire
 * but untyped here, so callers couldn't read it without an unsafe cast. `code` mirrors gRPC status
 * names (e.g. "FailedPrecondition", "NotFound") and should be preferred over substring-matching
 * the human-readable `message`.
 */
export type GraphqlErrorEntry = {
  message?: string;
  extensions?: { code?: string; grpc_code?: number };
};

export function graphQlUrl(): string {
  return serverGraphqlUrl();
}

export async function requireSessionToken(): Promise<string | null> {
  try {
    const session = await getServerSession(authOptions);
    const token = session?.idToken ?? session?.accessToken;
    return token ?? null;
  } catch {
    return null;
  }
}

function parseNumericId(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  return /^\d+$/.test(trimmed) ? trimmed : null;
}

export function decodeJwtSub(token: string): string | null {
  try {
    const parts = token.split(".");
    if (parts.length < 2) return null;
    const payload = JSON.parse(Buffer.from(parts[1], "base64").toString("utf8")) as {
      sub?: string;
    };
    return payload.sub ?? null;
  } catch {
    return null;
  }
}

export async function requireAuthenticatedCustomerUserId(): Promise<string | null> {
  try {
    const session = (await getServerSession(authOptions)) as { customerUserId?: string } | null;
    const fromSession = parseNumericId(session?.customerUserId);
    if (fromSession) return fromSession;
  } catch {
    return null;
  }

  const token = await requireSessionToken();
  if (!token) return null;
  return parseNumericId(decodeJwtSub(token));
}

async function inboundForwardingHeaders(): Promise<Record<string, string>> {
  const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
  const inboundHeaders = await (async () => {
    try {
      const h = await nextHeaders();
      return {
        requestId: h.get("x-request-id")?.trim() || null,
        clientAction: h.get("x-client-action")?.trim() || null,
        guestSessionId: h.get("x-guest-session-id")?.trim() || null,
      };
    } catch {
      return { requestId: null, clientAction: null, guestSessionId: null };
    }
  })();

  return {
    ...forwardedHeaders,
    ...(inboundHeaders.requestId ? { "X-Request-Id": inboundHeaders.requestId } : {}),
    ...(inboundHeaders.clientAction ? { "X-Client-Action": inboundHeaders.clientAction } : {}),
    ...(inboundHeaders.guestSessionId
      ? { "X-Guest-Session-Id": inboundHeaders.guestSessionId }
      : {}),
  };
}

async function callGraphqlRaw<T = unknown>(
  authHeaders: Record<string, string>,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<GraphqlErrorEntry> }> {
  try {
    const forwardingHeaders = await inboundForwardingHeaders();
    const res = await fetchWithResilience(
      graphQlUrl(),
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...authHeaders,
          ...forwardingHeaders,
          ...extraHeaders,
        },
        body: JSON.stringify({ query, variables }),
        cache: "no-store",
      },
      { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
    );
    const text = await res.text();
    if (!res.ok) {
      return { errors: [{ message: text || `HTTP ${res.status}` }] };
    }
    return JSON.parse(text) as { data?: T; errors?: Array<GraphqlErrorEntry> };
  } catch (error) {
    return {
      errors: [
        {
          message: normalizeNetworkError(error, "Invalid GraphQL response"),
        },
      ],
    };
  }
}

export async function callGraphql<T = unknown>(
  token: string,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<GraphqlErrorEntry> }> {
  return callGraphqlRaw<T>(
    {
      Authorization: token.startsWith("Bearer ") ? token : `Bearer ${token}`,
    },
    query,
    variables,
    extraHeaders
  );
}

export async function callGraphqlAsCustomer<T = unknown>(
  customerUserId: string,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<GraphqlErrorEntry> }> {
  const internalSecret = serverEnv.INTERNAL_API_SECRET?.trim();
  if (!internalSecret) {
    return {
      errors: [{ message: "INTERNAL_API_SECRET is not configured" }],
    };
  }

  return callGraphqlRaw<T>(
    {
      "X-Internal-Auth": internalSecret,
      "X-Customer-User-Id": customerUserId,
    },
    query,
    variables,
    extraHeaders
  );
}

export async function callGraphqlAsInternalService<T = unknown>(
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<GraphqlErrorEntry> }> {
  const internalSecret = serverEnv.INTERNAL_API_SECRET?.trim();
  if (!internalSecret) {
    return {
      errors: [{ message: "INTERNAL_API_SECRET is not configured" }],
    };
  }

  return callGraphqlRaw<T>(
    {
      "X-Internal-Auth": internalSecret,
    },
    query,
    variables,
    extraHeaders
  );
}

/**
 * Maps a GraphQL error's extensions.code (falling back to substring-matching message, same
 * heuristic already used in account/orders/[orderId]/cancel and .../returns) to an HTTP
 * status, so callers get a distinguishable status instead of always 400. Reused across the
 * checkout and account routes so PermissionDenied/Unauthenticated/backend-unavailable failures
 * surface as distinguishable statuses rather than being indistinguishable from a plain
 * validation error.
 *
 * Unauthenticated maps to 401 (not 403, alongside PermissionDenied) so that
 * `fetchApiEnvelope`'s session-refresh-and-retry logic — gated on exactly
 * `response.status === 401` — actually fires on a stale/expired token, the most common
 * real-world trigger for this code.
 */
export function graphqlErrorToApiStatus(
  errors: GraphqlErrorEntry[] | undefined,
  fallbackMessage: string
): { status: number; message: string } {
  const firstError = errors?.[0];
  const message = firstError?.message?.trim() || fallbackMessage;
  const code = firstError?.extensions?.code;
  const lower = message.toLowerCase();
  const status =
    code === "Unauthenticated"
      ? 401
      : code === "PermissionDenied"
        ? 403
        : code === "NotFound" || lower.includes("not found")
          ? 404
          : code === "FailedPrecondition" || lower.includes("failed_precondition")
            ? 409
            : code === "Aborted"
              ? 409
              : code === "OutOfRange"
                ? 400
                : code === "Unimplemented"
                  ? 501
                  : code === "Unavailable"
                    ? 503
                    : code === "DeadlineExceeded"
                      ? 504
                      : code === "Cancelled"
                        ? 503
                        : lower.includes("illegal") || lower.includes("invalid")
                          ? 400
                          : 400;
  return { status, message };
}

export function apiError(message: string, status: number, errorCode: string) {
  return Response.json(
    {
      ok: false,
      data: null,
      errorCode,
      message,
      fieldErrors: null,
      retryable: status >= 500,
    },
    { status }
  );
}
