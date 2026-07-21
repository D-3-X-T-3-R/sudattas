import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const REVIEW_ROOTS = [
  "searchReview",
  "updateReview",
  "deleteReview",
  "adminUpdateReviewStatus",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: REVIEW_ROOTS });
}

