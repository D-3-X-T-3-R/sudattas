import type { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";
// import CredentialsProvider from "next-auth/providers/credentials"; // phone-otp, disabled for now
import { graphqlBaseUrl, serverEnv } from "@/lib/env/server";
import { appendTelemetryEvent } from "@/lib/telemetry-store";

async function trackAuthEvent(input: {
  action: string;
  outcome: "success" | "failure";
  userMode: "public" | "admin";
  errorClass?: "none" | "unauthorized" | "validation" | "retryable" | "network" | "fatal" | "boundary";
  errorCode?: string | null;
  message?: string | null;
}) {
  try {
    await appendTelemetryEvent({
      occurredAt: new Date().toISOString(),
      route: "/api/auth/[...nextauth]",
      pageRoute: null,
      userMode: input.userMode,
      action: input.action,
      outcome: input.outcome,
      errorClass: input.errorClass ?? (input.outcome === "success" ? "none" : "fatal"),
      errorCode: input.errorCode ?? null,
      message: input.message ?? null,
      status: null,
      requestId: null,
      online: null,
      userAgent: null,
      effectiveType: null,
      downlink: null,
      rtt: null,
    });
  } catch {
    // Telemetry must never block auth.
  }
}

/* ── Phone/OTP sign-in — disabled for now, kept for a possible future re-enable ──
function normalizePhone(raw: string): string | null {
  const digits = raw.replace(/\D/g, "");
  if (digits.length === 10) return `+91${digits}`;
  if (digits.length === 12 && digits.startsWith("91")) return `+${digits}`;
  if (digits.length >= 10 && raw.trim().startsWith("+")) return `+${digits}`;
  return null;
}

async function verifyTwilioOtp(phoneE164: string, otp: string): Promise<boolean> {
  const base = getGraphqlBaseUrl();
  const res = await fetch(`${base}/auth/phone-otp/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ phone: phoneE164, otp }),
    cache: "no-store",
  });
  if (!res.ok) return false;
  const data = (await res.json()) as { approved?: boolean };
  return data.approved === true;
}

function digitsOnly(value: string): string {
  return value.replace(/\D/g, "");
}
──────────────────────────────────────────────────────────────────────────── */

function getGraphqlBaseUrl(): string {
  return graphqlBaseUrl();
}

async function refreshGoogleIdToken(refreshToken: string): Promise<{
  idToken: string;
  accessToken: string;
  idTokenExpiresAt: number;
} | null> {
  try {
    const res = await fetch("https://oauth2.googleapis.com/token", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        client_id: serverEnv.GOOGLE_CLIENT_ID ?? "",
        client_secret: serverEnv.GOOGLE_CLIENT_SECRET ?? "",
        grant_type: "refresh_token",
        refresh_token: refreshToken,
      }),
      cache: "no-store",
    });
    if (!res.ok) return null;
    const data = (await res.json()) as {
      id_token?: string;
      access_token?: string;
      expires_in?: number;
    };
    if (!data.id_token || !data.access_token || !data.expires_in) return null;
    return {
      idToken: data.id_token,
      accessToken: data.access_token,
      idTokenExpiresAt: Date.now() + data.expires_in * 1000,
    };
  } catch {
    return null;
  }
}

function extractGoogleSubFromJwt(idToken: string): string | null {
  try {
    const parts = idToken.split(".");
    if (parts.length < 2) return null;
    const payload = JSON.parse(
      Buffer.from(parts[1], "base64").toString("utf8")
    ) as { sub?: string };
    return payload.sub ?? null;
  } catch {
    return null;
  }
}

function isDuplicateUserError(message: string): boolean {
  const m = message.toLowerCase();
  return (
    m.includes("duplicate") ||
    m.includes("already exists") ||
    m.includes("unique")
  );
}

async function createStorefrontUserInternal(input: {
  username: string;
  email: string;
  fullName?: string | null;
  authProvider: "google";
  googleSub: string;
}): Promise<string | null> {
  const internalSecret = serverEnv.INTERNAL_API_SECRET?.trim();
  if (!internalSecret) return null;

  const mutation = `
    mutation CreateStorefrontUser($input: NewUser!) {
      createUser(input: $input) {
        userId
      }
    }
  `;

  const res = await fetch(`${getGraphqlBaseUrl()}/v2`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Internal-Auth": internalSecret,
    },
    body: JSON.stringify({
      query: mutation,
      variables: {
        input: {
          username: input.username,
          email: input.email,
          authProvider: input.authProvider,
          googleSub: input.googleSub,
          fullName: input.fullName ?? null,
        },
      },
    }),
    cache: "no-store",
  });

  if (!res.ok) return null;
  const json = (await res.json().catch(() => ({}))) as {
    data?: { createUser?: Array<{ userId?: string | number | null }> };
    errors?: Array<{ message?: string }>;
  };
  const err = json.errors?.[0]?.message;
  if (err && !isDuplicateUserError(err)) return null;
  const first = json.data?.createUser?.[0]?.userId;
  return first === null || first === undefined ? null : String(first);
}

async function syncGoogleUserToBackend(input: {
  idToken: string;
  email: string;
  username: string;
  fullName?: string | null;
}): Promise<string | null> {
  const googleSub = extractGoogleSubFromJwt(input.idToken);
  if (!googleSub) return null;

  return createStorefrontUserInternal({
    username: input.username,
    email: input.email,
    fullName: input.fullName ?? null,
    authProvider: "google",
    googleSub,
  });
}

// How often to re-probe admin status for an already-established session, instead of trusting the
// sign-in-time isAdmin claim for the full session lifetime. Matches the backend's own
// ADMIN_ROLE_CACHE_TTL_SEC default (admin_roles.rs) so the page-navigation gate (middleware.ts)
// doesn't stay open much longer than the backend's own admin-role cache would.
//
// Account deactivation is *not* tracked this way (see history: an earlier version cached
// `accountDeactivated` on the JWT the same way, re-probed on this same interval using the
// customer's own Google idToken). That was wrong on two counts: (1) middleware's `getToken()`
// only ever reads whatever is already baked into the session cookie — it never runs this
// callback itself, so the cached flag could only become fresh again after a separate client-side
// session round-trip happened to fire, meaning a customer could sail right through a fresh
// deactivation for the whole interval; and (2) reactivation had the same problem in reverse.
// Account status is instead checked live on every request that needs it (middleware.ts,
// api/account/status/route.ts) via internal-service auth keyed on the stable `customerUserId`
// already in the token, which sidesteps Google-idToken freshness entirely.
const ADMIN_RECHECK_INTERVAL_MS = 5 * 60 * 1000;

async function probeAdminAccessByToken(token: string): Promise<boolean> {
  const query = `query AdminProbeCanSearchUsers {
    searchUser(input: { limit: "1", offset: "0" }) {
      userId
    }
  }`;
  try {
    const res = await fetch(`${getGraphqlBaseUrl()}/v2`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: token.startsWith("Bearer ") ? token : `Bearer ${token}`,
      },
      body: JSON.stringify({ query, variables: {} }),
      cache: "no-store",
    });
    const json = (await res.json().catch(() => ({}))) as {
      errors?: Array<{ message?: string }>;
    };
    return res.ok && !(json.errors?.length);
  } catch {
    return false;
  }
}

/* ── Phone/OTP sign-in — disabled for now, kept for a possible future re-enable ──
async function syncPhoneOtpUserToBackend(phoneE164: string): Promise<string | null> {
  const digits = digitsOnly(phoneE164);
  if (!digits) return null;
  return createStorefrontUserInternal({
    username: `otp_${digits}`,
    email: `otp_${digits}@phone.local`,
    fullName: phoneE164,
    authProvider: "google",
    googleSub: `otp:${digits}`,
  });
}
──────────────────────────────────────────────────────────────────────────── */

export const authOptions: NextAuthOptions = {
  session: {
    strategy: "jwt",
    maxAge: 90 * 24 * 60 * 60, // 90 days
  },
  jwt: {
    maxAge: 90 * 24 * 60 * 60, // 90 days
  },
  providers: [
    GoogleProvider({
      clientId: serverEnv.GOOGLE_CLIENT_ID ?? "",
      clientSecret: serverEnv.GOOGLE_CLIENT_SECRET ?? "",
      authorization: {
        params: {
          prompt: "consent",
          access_type: "offline",
        },
      },
    }),
    /* ── Phone/OTP sign-in — disabled for now, kept for a possible future re-enable ──
    CredentialsProvider({
      id: "phone-otp",
      name: "Phone OTP",
      credentials: {
        phone: { label: "Phone", type: "text" },
        otp: { label: "OTP", type: "text" },
      },
      async authorize(credentials) {
        const phoneE164 = normalizePhone(credentials?.phone ?? "");
        const otp = (credentials?.otp ?? "").trim();
        if (!phoneE164 || !/^\d{4,8}$/.test(otp)) return null;
        const approved = await verifyTwilioOtp(phoneE164, otp);
        if (!approved) return null;
        const customerUserId = await syncPhoneOtpUserToBackend(phoneE164);
        if (!customerUserId) return null;
        return {
          id: phoneE164,
          name: phoneE164,
          email: null,
          customerUserId,
        };
      },
    }),
    ──────────────────────────────────────────────────────────────────────────── */
  ],
  secret: serverEnv.AUTH_SECRET ?? serverEnv.NEXTAUTH_SECRET,
  callbacks: {
    async signIn({ user, account }) {
      const isAdminFlow =
        typeof account?.callbackUrl === "string" &&
        account.callbackUrl.includes("/imtheboss");
      const userMode: "public" | "admin" = isAdminFlow ? "admin" : "public";

      /* ── Phone/OTP sign-in — disabled for now, kept for a possible future re-enable ──
      if (account?.provider === "phone-otp") {
        const customerUserId = (user as { customerUserId?: string } | null)?.customerUserId;
        if (!customerUserId) {
          await trackAuthEvent({
            action: "AUTH_SIGN_IN_PHONE_OTP",
            outcome: "failure",
            userMode,
            errorClass: "fatal",
            errorCode: "BACKEND_SYNC_FAILED",
            message: "OTP sign-in did not resolve canonical customer identity.",
          });
          return false;
        }
        await trackAuthEvent({
          action: "AUTH_SIGN_IN_PHONE_OTP",
          outcome: "success",
          userMode,
        });
        return true;
      }
      ──────────────────────────────────────────────────────────────────────────── */
      const email = user?.email?.toLowerCase();
      if (!email) {
        await trackAuthEvent({
          action: "AUTH_SIGN_IN_GOOGLE",
          outcome: "failure",
          userMode,
          errorClass: "validation",
          errorCode: "MISSING_EMAIL",
          message: "Google sign-in missing email.",
        });
        return false;
      }

      // Admin authorization is backend-role-driven; avoid frontend allowlist hard-deny.
      // Non-admin users are rejected by backend admin resolvers.

      // Backend-authoritative provisioning: persist storefront user on successful Google auth.
      const idToken = account?.id_token;
      const username = user?.name?.trim();
      if (!idToken || !username) {
        await trackAuthEvent({
          action: "AUTH_SIGN_IN_GOOGLE",
          outcome: "failure",
          userMode,
          errorClass: "validation",
          errorCode: "MISSING_TOKEN_OR_USERNAME",
          message: "Google sign-in missing id token or username.",
        });
        return false;
      }
      const customerUserId = await syncGoogleUserToBackend({
        idToken,
        email,
        username,
        fullName: user?.name ?? null,
      });
      const isAdmin = await probeAdminAccessByToken(idToken);
      const synced = Boolean(customerUserId);
      if (synced) {
        (user as { customerUserId?: string; isAdmin?: boolean }).customerUserId =
          customerUserId ?? undefined;
        (user as { customerUserId?: string; isAdmin?: boolean }).isAdmin = isAdmin;
      }
      if (isAdminFlow && !isAdmin) {
        await trackAuthEvent({
          action: "AUTH_SIGN_IN_GOOGLE",
          outcome: "failure",
          userMode,
          errorClass: "unauthorized",
          errorCode: "ADMIN_ACCESS_DENIED",
          message: "User authenticated but is not authorized for admin panel.",
        });
        return false;
      }
      await trackAuthEvent({
        action: "AUTH_SIGN_IN_GOOGLE",
        outcome: synced ? "success" : "failure",
        userMode,
        errorClass: synced ? "none" : "fatal",
        errorCode: synced ? null : "BACKEND_SYNC_FAILED",
        message: synced ? "Google sign-in synced." : "Failed to sync user with backend.",
      });
      return synced;
    },
    async jwt({ token, account, user }) {
      if (account?.access_token) {
        token.accessToken = account.access_token;
      }
      if (account?.id_token) {
        token.idToken = account.id_token;
      }
      if (account?.refresh_token) {
        (token as { refreshToken?: string }).refreshToken = account.refresh_token;
      }
      if (account?.expires_at) {
        (token as { idTokenExpiresAt?: number }).idTokenExpiresAt = account.expires_at * 1000;
      }
      // The Google id_token backend calls authenticate with has its own short lifetime
      // (Google-issued, independent of this NextAuth session's 90-day maxAge). Refresh it
      // proactively before it expires — and before the isAdmin probe below, which otherwise
      // presents an already-expired idToken to the backend and reads the resulting auth failure
      // as "confirmed not admin" rather than "couldn't check".
      {
        const expiresAt = (token as { idTokenExpiresAt?: number }).idTokenExpiresAt;
        const refreshToken = (token as { refreshToken?: string }).refreshToken;
        if (typeof expiresAt === "number" && Date.now() > expiresAt - 60_000 && refreshToken) {
          const refreshed = await refreshGoogleIdToken(refreshToken);
          if (refreshed) {
            token.idToken = refreshed.idToken;
            token.accessToken = refreshed.accessToken;
            (token as { idTokenExpiresAt?: number }).idTokenExpiresAt = refreshed.idTokenExpiresAt;
          } else {
            // Refresh failed (revoked/expired refresh token) — drop the stale idToken so
            // downstream backend calls fail fast with 401 instead of silently using dead creds.
            // Also drop accessToken: server-session-auth.ts and admin-graphql-server.ts both
            // fall back to it when idToken is unset (`session?.idToken ?? session?.accessToken`),
            // which would otherwise silently forward the equally-stale accessToken instead.
            (token as { idToken?: string }).idToken = undefined;
            token.accessToken = undefined;
          }
        }
      }
      const maybeCustomerUserId = (token as { customerUserId?: string }).customerUserId;
      if (!maybeCustomerUserId) {
        const userCustomerId = (user as { customerUserId?: string } | undefined)?.customerUserId;
        if (userCustomerId) {
          (token as { customerUserId?: string }).customerUserId = userCustomerId;
        }
      }
      if (typeof (token as { isAdmin?: boolean }).isAdmin !== "boolean") {
        (token as { isAdmin?: boolean }).isAdmin = false;
      }
      const userIsAdmin = (user as { isAdmin?: boolean } | undefined)?.isAdmin;
      if (typeof userIsAdmin === "boolean") {
        (token as { isAdmin?: boolean }).isAdmin = userIsAdmin;
        (token as { isAdminCheckedAt?: number }).isAdminCheckedAt = Date.now();
      } else if (
        (token as { isAdmin?: boolean }).isAdmin === true &&
        typeof token.idToken === "string"
      ) {
        // Re-probe periodically instead of trusting the sign-in-time isAdmin claim for the full
        // session lifetime, so a revoked admin's page-navigation gate (middleware.ts) closes
        // within roughly the same window the backend itself uses for admin role resolution.
        const checkedAt = (token as { isAdminCheckedAt?: number }).isAdminCheckedAt ?? 0;
        if (Date.now() - checkedAt > ADMIN_RECHECK_INTERVAL_MS) {
          // probeAdminAccessByToken already catches internally and resolves `false` on any
          // network/HTTP failure (never rejects) — so a transient backend hiccup during this
          // recheck window demotes the admin (fail-closed), not the fail-open behavior a wrapping
          // .catch() would visually suggest. This is intentionally the safer default; if fail-open
          // is actually wanted, probeAdminAccessByToken needs to distinguish "confirmed not admin"
          // from "couldn't check" (e.g. return `boolean | null`) rather than collapsing both to `false`.
          const stillAdmin = await probeAdminAccessByToken(token.idToken);
          (token as { isAdmin?: boolean }).isAdmin = stillAdmin;
          (token as { isAdminCheckedAt?: number }).isAdminCheckedAt = Date.now();
        }
      }
      // Persist so we have it on session refresh
      return token;
    },
    async session({ session, token }) {
      if (session.user) {
        const s = session as {
          accessToken?: string;
          idToken?: string;
          customerUserId?: string;
          isAdmin?: boolean;
        };
        s.accessToken = token.accessToken as string | undefined;
        s.idToken = token.idToken as string | undefined;
        s.customerUserId = (token as { customerUserId?: string }).customerUserId;
        s.isAdmin = (token as { isAdmin?: boolean }).isAdmin === true;
      }
      return session;
    },
  },
  pages: {
    signIn: "/imtheboss/login",
  },
};
