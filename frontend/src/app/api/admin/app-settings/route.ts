import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const APP_SETTINGS_ROOTS = ["abandonedCartSettings", "updateAbandonedCartSettings"];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: APP_SETTINGS_ROOTS });
}
