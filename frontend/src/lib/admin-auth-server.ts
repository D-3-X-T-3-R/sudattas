import "server-only";

import { getServerSession } from "next-auth";
import type { Session } from "next-auth";
import { authOptions } from "@/lib/auth";
import { serverEnv } from "@/lib/env/server";

function adminAllowlist(): string[] {
  const raw = serverEnv.ADMIN_ALLOWED_EMAILS ?? "";
  return raw
    .split(",")
    .map((email) => email.trim().toLowerCase())
    .filter(Boolean);
}

export function isAdminEmail(email: string | null | undefined): boolean {
  if (!email) return false;
  const allowlist = adminAllowlist();
  if (allowlist.length === 0) return false;
  return allowlist.includes(email.trim().toLowerCase());
}

export async function getAdminSession(): Promise<Session | null> {
  let session: Session | null;
  try {
    session = await getServerSession(authOptions);
  } catch {
    return null;
  }
  if (!session?.user?.email) return null;
  if (!isAdminEmail(session.user.email)) return null;
  return session;
}
