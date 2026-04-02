import { fetchWithResilience } from "@/lib/network-resilience";

let mintSingleFlight: Promise<string | null> | null = null;

function graphqlBaseUrl(): string {
  const raw =
    process.env.GRAPHQL_URL ||
    process.env.NEXT_PUBLIC_GRAPHQL_URL ||
    "http://localhost:8080/v2";
  return raw.replace(/\/v2\/?$/, "");
}

export function isStaleGuestSessionError(err: unknown): boolean {
  if (!(err instanceof Error)) return false;
  const m = err.message.toLowerCase();
  return m.includes("session invalid") || m.includes("expired") || m.includes("unauthorized");
}

export async function withRecoveredGuestSession<T>(
  sessionId: string | null,
  extraHeaders: Record<string, string>,
  operation: (activeSessionId: string) => Promise<T>
): Promise<{ value: T; sessionIdUsed: string; refreshedSessionId: string | null }> {
  if (!sessionId) {
    throw new Error("Guest session unavailable");
  }
  try {
    return {
      value: await operation(sessionId),
      sessionIdUsed: sessionId,
      refreshedSessionId: null,
    };
  } catch (error) {
    if (!isStaleGuestSessionError(error)) throw error;
    const fresh = await mintGuestSessionIdSingleFlight(extraHeaders);
    if (!fresh) throw error;
    return {
      value: await operation(fresh),
      sessionIdUsed: fresh,
      refreshedSessionId: fresh,
    };
  }
}

export async function mintGuestSessionIdSingleFlight(
  extraHeaders: Record<string, string> = {}
): Promise<string | null> {
  if (!mintSingleFlight) {
    mintSingleFlight = (async () => {
      try {
        const res = await fetchWithResilience(
          `${graphqlBaseUrl()}/session/guest`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json", ...extraHeaders },
            cache: "no-store",
          },
          { max429Retries: 1, maxNetworkRetries: 1, baseBackoffMs: 400 }
        );
        if (!res.ok) return null;
        const data = (await res.json()) as { session_id?: string };
        return data.session_id ?? null;
      } catch {
        return null;
      }
    })().finally(() => {
      mintSingleFlight = null;
    });
  }
  return mintSingleFlight;
}
