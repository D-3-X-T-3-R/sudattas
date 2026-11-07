import { callGraphqlAsCustomer, requireAuthenticatedCustomerUserId } from "@/lib/server-session-auth";

const ACCOUNT_STATUS_QUERY = `query AccountStatus {
  authInfo {
    accountDeactivated
  }
}`;

/**
 * Live account-deactivation check, used by the /account-deactivated page to re-check status on
 * demand (mount + a manual "Refresh" button) instead of waiting on any client-side session cache.
 * Deliberately calls the backend via internal-service auth (X-Internal-Auth + the customer's
 * stable numeric ID), never the customer's own Google idToken — that token has its own separate
 * expiry/refresh cycle this check has no business depending on, and an earlier version of this
 * feature broke exactly that way (a stale/expired idToken made the check fail, which was then
 * misread as "confirmed not deactivated").
 */
export async function GET() {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) {
    // No session at all — nothing to report as deactivated.
    return Response.json({ accountDeactivated: false });
  }

  const result = await callGraphqlAsCustomer<{ authInfo?: { accountDeactivated?: boolean } }>(
    customerUserId,
    ACCOUNT_STATUS_QUERY
  );

  if (result.errors?.length || !result.data) {
    // Couldn't confirm either way — the caller should treat this as "still don't know" rather
    // than assuming reactivated, matching the deliberately-cautious posture backend-side.
    return Response.json({ accountDeactivated: null }, { status: 502 });
  }

  return Response.json({
    accountDeactivated: result.data.authInfo?.accountDeactivated === true,
  });
}
