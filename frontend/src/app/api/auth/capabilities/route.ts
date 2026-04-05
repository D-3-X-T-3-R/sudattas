import { getAdminSession, getAuthenticatedSession } from "@/lib/admin-auth-server";

export async function GET() {
  const session = await getAuthenticatedSession();
  const authenticated = Boolean(session);
  const admin = Boolean(authenticated && (await getAdminSession()));

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
