import "server-only";

import { getServerSession } from "next-auth";
import { headers as nextHeaders } from "next/headers";
import { authOptions } from "@/lib/auth";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";
import { serverGraphqlUrl } from "@/lib/env/server";

export function graphQlUrl(): string {
  return serverGraphqlUrl();
}

export async function requireSessionToken(): Promise<string | null> {
  const session = await getServerSession(authOptions);
  const token = session?.idToken ?? session?.accessToken;
  return token ?? null;
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
  const session = (await getServerSession(authOptions)) as { customerUserId?: string } | null;
  const fromSession = parseNumericId(session?.customerUserId);
  if (fromSession) return fromSession;

  const token = await requireSessionToken();
  if (!token) return null;
  return parseNumericId(decodeJwtSub(token));
}

export async function callGraphql<T = unknown>(
  token: string,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<{ message?: string }> }> {
  try {
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
    const res = await fetchWithResilience(
      graphQlUrl(),
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: token.startsWith("Bearer ") ? token : `Bearer ${token}`,
          ...forwardedHeaders,
          ...(inboundHeaders.requestId ? { "X-Request-Id": inboundHeaders.requestId } : {}),
          ...(inboundHeaders.clientAction
            ? { "X-Client-Action": inboundHeaders.clientAction }
            : {}),
          ...(inboundHeaders.guestSessionId
            ? { "X-Guest-Session-Id": inboundHeaders.guestSessionId }
            : {}),
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
    return JSON.parse(text) as { data?: T; errors?: Array<{ message?: string }> };
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

