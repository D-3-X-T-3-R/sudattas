import type { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";
import CredentialsProvider from "next-auth/providers/credentials";

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
  const gqlUrl =
    process.env.GRAPHQL_URL ||
    process.env.NEXT_PUBLIC_GRAPHQL_URL ||
    "http://localhost:8080/v2";
  return gqlUrl.replace(/\/v2\/?$/, "");
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
}): Promise<boolean> {
  const googleSub = extractGoogleSubFromJwt(input.idToken);
  if (!googleSub) return false;

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

  if (!res.ok) return false;
  const json = (await res.json().catch(() => ({}))) as {
    errors?: Array<{ message?: string }>;
  };
  const err = json.errors?.[0]?.message;
  if (!err) return true;
  return isDuplicateUserError(err);
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
      clientId: process.env.GOOGLE_CLIENT_ID ?? "",
      clientSecret: process.env.GOOGLE_CLIENT_SECRET ?? "",
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
  secret: process.env.AUTH_SECRET,
  callbacks: {
    async signIn({ user, account }) {
      if (account?.provider === "phone-otp") return true;
      const email = user?.email?.toLowerCase();
      if (!email) return false;

      // Enforce allowlist only for admin sign-in page flow.
      // Storefront login should remain open to regular customers.
      const isAdminFlow =
        typeof account?.callbackUrl === "string" &&
        account.callbackUrl.includes("/imtheboss");
      const allowedRaw = process.env.ADMIN_ALLOWED_EMAILS;
      if (isAdminFlow && allowedRaw?.trim()) {
        const allowed = allowedRaw
          .split(",")
          .map((e) => e.trim().toLowerCase())
          .filter(Boolean);
        if (allowed.length > 0 && !allowed.includes(email)) return false;
      }

      // Backend-authoritative provisioning: persist storefront user on successful Google auth.
      const idToken = account?.id_token;
      const username = user?.name?.trim();
      if (!idToken || !username) return false;
      return syncGoogleUserToBackend({
        idToken,
        email,
        username,
        fullName: user?.name ?? null,
      });
    },
    async jwt({ token, account }) {
      if (account?.access_token) {
        token.accessToken = account.access_token;
      }
      if (account?.id_token) {
        token.idToken = account.id_token;
      }
      // Persist so we have it on session refresh
      return token;
    },
    async session({ session, token }) {
      if (session.user) {
        const s = session as { accessToken?: string; idToken?: string };
        s.accessToken = token.accessToken as string | undefined;
        s.idToken = token.idToken as string | undefined;
      }
      return session;
    },
  },
  pages: {
    signIn: "/imtheboss/login",
  },
};
