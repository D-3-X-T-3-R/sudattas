import "server-only";

import { NextResponse } from "next/server";
import { getAdminSession } from "@/lib/admin-auth-server";
import { graphQlUrl } from "@/lib/server-session-auth";
import { forwardedIpHeadersFromRequest } from "@/lib/forwarded-ip";
import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";

type GraphqlBody = {
  query?: string;
  variables?: Record<string, unknown>;
  operationName?: string;
};

type ForwardOptions = {
  allowedRoots?: string[];
};

type ApiEnvelope<T> = {
  ok: boolean;
  data: T | null;
  errorCode: string | null;
  message: string | null;
  fieldErrors: Record<string, string> | null;
  retryable: boolean;
};

function envelopeOk<T>(data: T): ApiEnvelope<T> {
  return {
    ok: true,
    data,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  };
}

function envelopeErr(
  status: number,
  errorCode: string,
  message: string,
  fieldErrors: Record<string, string> | null = null
): ApiEnvelope<null> {
  return {
    ok: false,
    data: null,
    errorCode,
    message,
    fieldErrors,
    retryable: status >= 500,
  };
}

/**
 * Maps the backend's GraphQL `extensions.code` (see backend/graphql/src/resolvers/error.rs,
 * gRPC-status-style names) to an HTTP status + errorCode, so a PermissionDenied rejection is
 * distinguishable from a plain validation failure instead of every GraphQL-layer error
 * collapsing to a flat 400/"GRAPHQL_ERROR".
 */
function graphqlCodeToStatus(code: string | undefined): { status: number; errorCode: string } {
  switch (code) {
    case "Unauthenticated":
      return { status: 401, errorCode: "UNAUTHORIZED" };
    case "PermissionDenied":
      return { status: 403, errorCode: "FORBIDDEN" };
    case "NotFound":
      return { status: 404, errorCode: "NOT_FOUND" };
    case "AlreadyExists":
      return { status: 409, errorCode: "CONFLICT" };
    case "FailedPrecondition":
      return { status: 409, errorCode: "FAILED_PRECONDITION" };
    case "Aborted":
      // Transient conflict (e.g. transaction abort) — same status family as FailedPrecondition.
      return { status: 409, errorCode: "CONFLICT" };
    case "OutOfRange":
      return { status: 400, errorCode: "VALIDATION_ERROR" };
    case "Unimplemented":
      return { status: 501, errorCode: "NOT_IMPLEMENTED" };
    case "ResourceExhausted":
      return { status: 429, errorCode: "RATE_LIMITED" };
    case "InvalidArgument":
      return { status: 400, errorCode: "VALIDATION_ERROR" };
    case "Unavailable":
      // Backend-down / circuit-breaker-tripped — distinct from a generic upstream error so
      // callers can treat it as retryable rather than a bad request.
      return { status: 503, errorCode: "UPSTREAM_UNAVAILABLE" };
    case "DeadlineExceeded":
      return { status: 504, errorCode: "UPSTREAM_TIMEOUT" };
    case "Cancelled":
      return { status: 503, errorCode: "UPSTREAM_UNAVAILABLE" };
    case "Internal":
    case "Unknown":
    case "DataLoss":
      return { status: 502, errorCode: "UPSTREAM_ERROR" };
    default:
      return { status: 400, errorCode: "GRAPHQL_ERROR" };
  }
}

function errorStatusToCode(status: number): string {
  if (status === 400) return "BAD_REQUEST";
  if (status === 401) return "UNAUTHORIZED";
  if (status === 403) return "FORBIDDEN";
  if (status === 404) return "NOT_FOUND";
  if (status === 409) return "CONFLICT";
  if (status === 429) return "RATE_LIMITED";
  if (status >= 500) return "UPSTREAM_ERROR";
  return "REQUEST_FAILED";
}

function firstOperationField(query: string): string | null {
  const normalized = query.replace(/\s+/g, " ").trim();
  const match = normalized.match(
    /^(?:query|mutation)(?:\s+[A-Za-z_]\w*)?(?:\s*\([^)]*\))?\s*\{\s*([A-Za-z_]\w*)/
  );
  return match ? match[1] : null;
}

// Checks that the query's first top-level selection is an allowed root, rather than that some
// allowed root name appears anywhere in the query text. This is a minimal, regex-based tightening
// — it does not defend against a single document with an allowed field followed by additional,
// unrelated top-level fields/mutations (that needs real GraphQL parsing). It closes the "root name
// appears in an unrelated place in the string" gap the previous substring check had.
function hasAllowedRoot(query: string, allowedRoots: string[]): boolean {
  const field = firstOperationField(query);
  return field !== null && allowedRoots.includes(field);
}

export async function forwardAdminGraphql(
  request: Request,
  options: ForwardOptions = {}
) {
  const session = await getAdminSession();
  if (!session) {
    return NextResponse.json(
      envelopeErr(401, "UNAUTHORIZED", "Unauthorized admin request"),
      { status: 401 }
    );
  }

  let body: GraphqlBody;
  try {
    body = (await request.json()) as GraphqlBody;
  } catch {
    return NextResponse.json(
      envelopeErr(400, "BAD_REQUEST", "Invalid JSON body"),
      { status: 400 }
    );
  }

  const query = body.query?.trim();
  if (!query) {
    return NextResponse.json(
      envelopeErr(400, "BAD_REQUEST", "Missing GraphQL query"),
      { status: 400 }
    );
  }

  if (
    options.allowedRoots &&
    options.allowedRoots.length > 0 &&
    !hasAllowedRoot(query, options.allowedRoots)
  ) {
    return NextResponse.json(
      envelopeErr(400, "BAD_REQUEST", "Query root not allowed on this admin route"),
      { status: 400 }
    );
  }

  const token = session.idToken ?? session.accessToken;
  if (!token) {
    return NextResponse.json(
      envelopeErr(401, "UNAUTHORIZED", "Admin session token unavailable"),
      { status: 401 }
    );
  }

  const baseHeaders = {
    "Content-Type": "application/json",
    Authorization: token.startsWith("Bearer ") ? token : `Bearer ${token}`,
    ...forwardedIpHeadersFromRequest(request),
    ...(request.headers.get("x-request-id")
      ? { "X-Request-Id": request.headers.get("x-request-id") as string }
      : {}),
    ...(request.headers.get("idempotency-key")
      ? { "Idempotency-Key": request.headers.get("idempotency-key") as string }
      : {}),
    ...(request.headers.get("x-client-action")
      ? { "X-Client-Action": request.headers.get("x-client-action") as string }
      : {}),
    ...(request.headers.get("x-guest-session-id")
      ? { "X-Guest-Session-Id": request.headers.get("x-guest-session-id") as string }
      : {}),
  };
  const payload = JSON.stringify({
    query,
    variables: body.variables ?? {},
    operationName: body.operationName,
  });
  let response: Response;
  try {
    response = await fetchWithResilience(
      graphQlUrl(),
      {
        method: "POST",
        headers: baseHeaders,
        body: payload,
        cache: "no-store",
      },
      { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
    );
  } catch (error) {
    return NextResponse.json(
      envelopeErr(502, "UPSTREAM_ERROR", normalizeNetworkError(error, "Failed to reach GraphQL")),
      { status: 502 }
    );
  }

  const text = await response.text();
  let parsed: {
    data?: unknown;
    errors?: Array<{ message?: string; extensions?: { code?: string; grpc_code?: number } }>;
  } | null = null;
  try {
    parsed = JSON.parse(text) as {
      data?: unknown;
      errors?: Array<{ message?: string; extensions?: { code?: string; grpc_code?: number } }>;
    };
  } catch {
    parsed = null;
  }

  // GraphQL layer can return HTTP 200 with `errors`; normalize these to envelope errors,
  // reading the backend's structured extensions.code rather than flattening every failure
  // to 400/"GRAPHQL_ERROR" (that previously made PermissionDenied indistinguishable from a
  // plain validation error to any caller classifying on status/errorCode).
  if (parsed?.errors?.length) {
    const firstErrorEntry = parsed.errors[0];
    const first = firstErrorEntry?.message?.trim() || "GraphQL operation failed";
    const mapped = graphqlCodeToStatus(firstErrorEntry?.extensions?.code);
    const status = response.status === 200 ? mapped.status : response.status;
    return NextResponse.json(envelopeErr(status, mapped.errorCode, first), { status });
  }

  if (!response.ok) {
    return NextResponse.json(
      envelopeErr(
        response.status,
        errorStatusToCode(response.status),
        text || `HTTP ${response.status}`
      ),
      { status: response.status }
    );
  }

  return NextResponse.json(envelopeOk(parsed?.data ?? null), { status: 200 });
}
