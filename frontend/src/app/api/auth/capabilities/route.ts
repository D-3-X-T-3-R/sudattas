import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";
import { isAdminEmail } from "@/lib/admin-auth-server";

export async function GET() {
  const session = await getServerSession(authOptions);
  const email = session?.user?.email ?? null;
  const authenticated = Boolean(session);
  const admin = isAdminEmail(email);

  return Response.json({
    ok: true,
    data: {
      authenticated,
      admin,
      mode: admin ? "admin" : authenticated ? "customer" : "guest",
      capabilities: {
        canCheckout: authenticated,
        canViewAccount: authenticated,
        canUseAdmin: admin,
      },
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

