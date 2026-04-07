import type { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";
import CredentialsProvider from "next-auth/providers/credentials";
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

function getGraphqlBaseUrl(): string {
  return graphqlBaseUrl();
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
        },
      },
    }),
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
  ],
  secret: serverEnv.AUTH_SECRET,
  callbacks: {
    async signIn({ user, account }) {
      const isAdminFlow =
        typeof account?.callbackUrl === "string" &&
        account.callbackUrl.includes("/imtheboss");
      const userMode: "public" | "admin" = isAdminFlow ? "admin" : "public";

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
