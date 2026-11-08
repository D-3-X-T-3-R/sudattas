import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const ROLE_ROOTS = [
  "createUserRole",
  "searchUserRole",
  "updateUserRole",
  "deleteUserRole",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: ROLE_ROOTS });
}
