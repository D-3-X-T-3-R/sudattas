import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const TRANSACTION_ROOTS = [
  "createTransaction",
  "searchTransaction",
  "updateTransaction",
  "deleteTransaction",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: TRANSACTION_ROOTS });
}
