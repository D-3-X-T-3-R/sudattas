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

async function syncGoogleUserToBackend(input: {
  idToken: string;
  email: string;
  username: string;
  fullName?: string | null;
}): Promise<string | null> {
  const googleSub = extractGoogleSubFromJwt(input.idToken);
  if (!googleSub) return null;

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
      Authorization: input.idToken.startsWith("Bearer ")
        ? input.idToken
        : `Bearer ${input.idToken}`,
    },
    body: JSON.stringify({
      query: mutation,
      variables: {
        input: {
          username: input.username,
          email: input.email,
          authProvider: "google",
          googleSub,
          fullName: input.fullName ?? null,
        },
      },
    }),
    cache: "no-store",
  });

  if (!res.ok) return null;
  const json = (await res.json().catch(() => ({}))) as {
    data?: {
      createUser?: Array<{ userId?: string | number | null }>;
    };
    errors?: Array<{ message?: string }>;
  };
  const err = json.errors?.[0]?.message;
  if (!err) {
    const first = json.data?.createUser?.[0]?.userId;
    return first === null || first === undefined ? null : String(first);
  }
  if (!isDuplicateUserError(err)) return null;

  // Backend now resolves duplicate provisioning to canonical user, but keep
  // the fallback branch for resilience when older deployments are still live.
  const first = json.data?.createUser?.[0]?.userId;
  return first === null || first === undefined ? null : String(first);
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
        return {
          id: phoneE164,
          name: phoneE164,
          email: null,
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

      // Enforce allowlist only for admin sign-in page flow.
      // Storefront login should remain open to regular customers.
      const allowedRaw = serverEnv.ADMIN_ALLOWED_EMAILS;
      if (isAdminFlow && allowedRaw?.trim()) {
        const allowed = allowedRaw
          .split(",")
          .map((e) => e.trim().toLowerCase())
          .filter(Boolean);
        if (allowed.length > 0 && !allowed.includes(email)) {
          await trackAuthEvent({
            action: "AUTH_SIGN_IN_GOOGLE",
            outcome: "failure",
            userMode,
            errorClass: "unauthorized",
            errorCode: "ACCESS_DENIED",
            message: "Admin allowlist denied email.",
          });
          return false;
        }
      }

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
      const synced = Boolean(customerUserId);
      if (synced) {
        (user as { customerUserId?: string }).customerUserId = customerUserId ?? undefined;
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
      // Persist so we have it on session refresh
      return token;
    },
    async session({ session, token }) {
      if (session.user) {
        const s = session as { accessToken?: string; idToken?: string; customerUserId?: string };
        s.accessToken = token.accessToken as string | undefined;
        s.idToken = token.idToken as string | undefined;
        s.customerUserId = (token as { customerUserId?: string }).customerUserId;
      }
      return session;
    },
  },
  pages: {
    signIn: "/imtheboss/login",
  },
};
