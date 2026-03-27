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
  const gqlUrl = process.env.NEXT_PUBLIC_GRAPHQL_URL || "http://localhost:8080/v2";
  const base = gqlUrl.replace(/\/v2\/?$/, "");
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

export const authOptions: NextAuthOptions = {
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
      const allowedRaw = process.env.ADMIN_ALLOWED_EMAILS;
      if (!allowedRaw?.trim()) return true;
      const allowed = allowedRaw
        .split(",")
        .map((e) => e.trim().toLowerCase())
        .filter(Boolean);
      if (allowed.length === 0) return true;
      const email = user?.email?.toLowerCase();
      if (!email) return false;
      return allowed.includes(email);
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
