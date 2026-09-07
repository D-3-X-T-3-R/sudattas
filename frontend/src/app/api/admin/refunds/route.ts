import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const REFUNDS_ROOTS = [
  "getRefunds",
  "createRefund",
  "searchRefundAttempts",
  "resolveRefundAttemptNeedsReview",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: REFUNDS_ROOTS });
}
