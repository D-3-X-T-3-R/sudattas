import "server-only";

import { getServerSession } from "next-auth";
import type { Session } from "next-auth";
import { authOptions } from "@/lib/auth";
import { callGraphql } from "@/lib/server-session-auth";

const ADMIN_PROBE_QUERY = `query AdminProbeCanSearchUsers {
  searchUser(input: { limit: "1", offset: "0" }) {
    userId
  }
}`;

type AdminProbeResponse = {
  searchUser?: Array<{ userId?: string }>;
};

type CacheRow = {
  value: boolean;
  expiresAt: number;
};

const ADMIN_CACHE_TTL_MS = 60_000;
const adminProbeCache = new Map<string, CacheRow>();

function cacheKeyFromSession(session: Session): string | null {
  const token =
    (session as Session & { idToken?: string; accessToken?: string }).idToken ??
    (session as Session & { idToken?: string; accessToken?: string }).accessToken;
  if (!token) return null;
  const email = session.user?.email?.trim().toLowerCase() ?? "";
  return `${email}:${token.slice(-16)}`;
}

async function probeAdminAccess(session: Session): Promise<boolean> {
  const token =
    (session as Session & { idToken?: string; accessToken?: string }).idToken ??
    (session as Session & { idToken?: string; accessToken?: string }).accessToken;
  if (!token) return false;

  const key = cacheKeyFromSession(session);
  if (key) {
    const cached = adminProbeCache.get(key);
    if (cached && cached.expiresAt > Date.now()) {
      return cached.value;
    }
  }

  const res = await callGraphql<AdminProbeResponse>(token, ADMIN_PROBE_QUERY, {});
  const isAdmin = !res.errors?.length;

  if (key) {
    adminProbeCache.set(key, {
      value: isAdmin,
      expiresAt: Date.now() + ADMIN_CACHE_TTL_MS,
    });
  }

  return isAdmin;
}

export async function getAuthenticatedSession(): Promise<Session | null> {
  try {
    const session = await getServerSession(authOptions);
    return session ?? null;
  } catch {
    return null;
  }
}

export async function getAdminSession(): Promise<Session | null> {
  const session = await getAuthenticatedSession();
  if (!session?.user?.email) return null;
  if (!(await probeAdminAccess(session))) return null;
  return session;
}
