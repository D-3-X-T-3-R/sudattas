import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const ACTIVITY_LOG_ROOTS = [
  "searchUserActivity",
  "searchEventLog",
  "createUserActivity",
  "createEventLog",
  "deleteUserActivity",
  "deleteEventLog",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: ACTIVITY_LOG_ROOTS });
}
