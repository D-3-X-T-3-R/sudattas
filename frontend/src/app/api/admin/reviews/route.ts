import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const REVIEW_ROOTS = [
  "searchReview",
  "searchProductReview",
  "updateReview",
  "deleteReview",
  "approveReview",
  "rejectReview",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: REVIEW_ROOTS });
}

