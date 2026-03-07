import type { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";

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
  ],
  secret: process.env.AUTH_SECRET,
  callbacks: {
    async signIn({ user }) {
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
