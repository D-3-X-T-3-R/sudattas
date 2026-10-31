import { gqlAdmin } from "./graphql-client";

export interface AdminReviewRow {
  reviewId: string;
  productId: string;
  userId: string;
  rating: number;
  comment: string;
  /** "pending" | "approved" | "rejected" */
  reviewStatus: string;
  isVerifiedPurchase: boolean;
  createdAt: string;
}

const REVIEW_FIELDS = `
  reviewId
  productId
  userId
  rating
  comment
  reviewStatus
  isVerifiedPurchase
  createdAt
`;

export async function fetchReviewsAdmin(statusFilter?: string): Promise<AdminReviewRow[]> {
  const data = await gqlAdmin<{ searchReview?: AdminReviewRow[] }>(
    `query AdminSearchReviews($input: SearchReview!) {
      searchReview(input: $input) { ${REVIEW_FIELDS} }
    }`,
    { input: statusFilter ? { statusFilter } : {} }
  );
  const rows = data?.searchReview ?? [];
  return [...rows].sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
}

export async function adminSetReviewStatus(
  reviewId: string,
  status: "approved" | "rejected"
): Promise<void> {
  await gqlAdmin(
    `mutation AdminSetReviewStatus($input: AdminUpdateReviewStatusInput!) {
      adminUpdateReviewStatus(input: $input)
    }`,
    { input: { reviewId, status } }
  );
}

export async function deleteReviewAdmin(reviewId: string): Promise<void> {
  await gqlAdmin(
    `mutation AdminDeleteReview($reviewId: String!) {
      deleteReview(reviewId: $reviewId) { reviewId }
    }`,
    { reviewId }
  );
}
