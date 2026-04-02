import "server-only";

import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { fetchWithResilience, normalizeNetworkError } from "@/lib/network-resilience";

export function graphQlUrl(): string {
  return (
    process.env.GRAPHQL_URL ||
    process.env.NEXT_PUBLIC_GRAPHQL_URL ||
    "http://localhost:8080/v2"
  );
}

export async function requireSessionToken(): Promise<string | null> {
  const session = await getServerSession(authOptions);
  const token = session?.idToken ?? session?.accessToken;
  return token ?? null;
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

export async function callGraphql<T = unknown>(
  token: string,
  query: string,
  variables: Record<string, unknown> = {},
  extraHeaders: Record<string, string> = {}
): Promise<{ data?: T; errors?: Array<{ message?: string }> }> {
  try {
    const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
    const res = await fetchWithResilience(
      graphQlUrl(),
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: token.startsWith("Bearer ") ? token : `Bearer ${token}`,
          ...forwardedHeaders,
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
