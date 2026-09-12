import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const RETURNS_ROOTS = [
  "searchReturnRequests",
  "adminMarkReturnReceived",
  "adminUpdateReturnStatus",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: RETURNS_ROOTS });
}
