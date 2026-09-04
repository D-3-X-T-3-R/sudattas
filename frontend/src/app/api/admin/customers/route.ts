import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const CUSTOMER_ROOTS = [
  "searchUser",
  "updateUser",
  "createUser",
  "deleteUser",
  "setUserStatus",
  "adminExportUserPii",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: CUSTOMER_ROOTS });
}

