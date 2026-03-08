import NextAuth from "next-auth";
import { authOptions } from "@/lib/auth";

const handler = NextAuth(authOptions);

async function wrappedGet(
  req: Request,
  context: { params: Promise<{ nextauth: string[] }> }
) {
  try {
    return await handler(req, context);
  } catch (err: unknown) {
    // Session cookie was encrypted with a different AUTH_SECRET (e.g. after env change).
    // Return empty session so the client treats the user as signed out instead of throwing.
    const isDecryptError =
      err instanceof Error &&
      (err.name === "JWEDecryptionFailed" || err.message?.includes("decryption operation failed"));
    if (isDecryptError) {
      return new Response(
        JSON.stringify({ session: null }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    }
    throw err;
  }
}

export { wrappedGet as GET, handler as POST };
