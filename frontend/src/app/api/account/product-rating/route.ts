// Reviews/ratings are disabled in the frontend for now — the backend (delivered-purchase gate,
// productRatingSummary aggregate) and this route's implementation are kept intact below, just
// commented out, so this route currently has no exported handlers and 404s. To re-enable: (1)
// uncomment this whole file (and remove the `export {}` below), (2) uncomment its test at
// src/app/api/account/__tests__/product-rating-route.test.ts, (3) uncomment the widget usage in
// product-detail-view.tsx and the summary fetch in app/product/[id]/page.tsx.

// Keeps this file a module (not an ambient script) for Next's App Router route-type validator,
// which otherwise errors with "is not a module" when a route.ts has zero exports.
export {};

// import {
//   apiError,
//   callGraphqlAsCustomer,
//   graphqlErrorToApiStatus,
//   requireAuthenticatedCustomerUserId,
// } from "@/lib/server-session-auth";
//
// /**
//  * Star-only product ratings (1-5), no written review text at this time. This route always
//  * upserts: `Reviews` has a UNIQUE(UserID, ProductID) constraint on the backend, so a second
//  * rating from the same customer for the same product must go through `updateReview` (keyed by
//  * the existing `reviewId`) rather than a second `createReview`, which would fail as a duplicate.
//  */
//
// const MIN_RATING = 1;
// const MAX_RATING = 5;
//
// type MyReviewRow = { reviewId: string; rating: number };
//
// const MY_REVIEW_QUERY = `query MyProductRating($input: SearchReview!) {
//   searchReview(input: $input) { reviewId rating }
// }`;
//
// const CREATE_REVIEW_MUTATION = `mutation CreateProductRating($input: NewReview!) {
//   createReview(input: $input) { reviewId rating }
// }`;
//
// const UPDATE_REVIEW_MUTATION = `mutation UpdateProductRating($input: ReviewMutation!) {
//   updateReview(input: $input) { reviewId rating }
// }`;
//
// function normalizeProductId(raw: unknown): string {
//   return String(raw ?? "").trim();
// }
//
// function parseRating(raw: unknown): number | null {
//   const n = typeof raw === "number" ? raw : Number(raw);
//   if (!Number.isInteger(n) || n < MIN_RATING || n > MAX_RATING) return null;
//   return n;
// }
//
// async function findMyReview(
//   userId: string,
//   productId: string
// ): Promise<{ data?: { searchReview?: MyReviewRow[] }; errors?: unknown[] }> {
//   return callGraphqlAsCustomer<{ searchReview?: MyReviewRow[] }>(userId, MY_REVIEW_QUERY, {
//     input: { productId, userId, limit: "1" },
//   });
// }
//
// export async function GET(request: Request) {
//   const userId = await requireAuthenticatedCustomerUserId();
//   if (!userId) {
//     return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
//   }
//
//   const productId = normalizeProductId(new URL(request.url).searchParams.get("productId"));
//   if (!productId) {
//     return apiError("productId is required", 400, "VALIDATION_ERROR");
//   }
//
//   const result = await findMyReview(userId, productId);
//   if (result.errors?.length) {
//     const { status, message } = graphqlErrorToApiStatus(
//       result.errors as Parameters<typeof graphqlErrorToApiStatus>[0],
//       "Failed to load your rating"
//     );
//     return apiError(message, status, "GRAPHQL_ERROR");
//   }
//
//   const existing = result.data?.searchReview?.[0] ?? null;
//   return Response.json({
//     ok: true,
//     data: { rating: existing?.rating ?? null },
//     errorCode: null,
//     message: null,
//     fieldErrors: null,
//     retryable: false,
//   });
// }
//
// export async function POST(request: Request) {
//   const userId = await requireAuthenticatedCustomerUserId();
//   if (!userId) {
//     return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
//   }
//
//   const body = (await request.json().catch(() => ({}))) as {
//     productId?: unknown;
//     rating?: unknown;
//   };
//   const productId = normalizeProductId(body.productId);
//   if (!productId) {
//     return apiError("productId is required", 400, "VALIDATION_ERROR");
//   }
//   const rating = parseRating(body.rating);
//   if (rating === null) {
//     return apiError(
//       `rating must be a whole number between ${MIN_RATING} and ${MAX_RATING}`,
//       400,
//       "VALIDATION_ERROR"
//     );
//   }
//
//   const existingResult = await findMyReview(userId, productId);
//   if (existingResult.errors?.length) {
//     const { status, message } = graphqlErrorToApiStatus(
//       existingResult.errors as Parameters<typeof graphqlErrorToApiStatus>[0],
//       "Failed to load your rating"
//     );
//     return apiError(message, status, "GRAPHQL_ERROR");
//   }
//   const existing = existingResult.data?.searchReview?.[0] ?? null;
//
//   const saved = existing
//     ? await callGraphqlAsCustomer<{ updateReview?: MyReviewRow[] }>(
//         userId,
//         UPDATE_REVIEW_MUTATION,
//         { input: { reviewId: existing.reviewId, rating } }
//       )
//     : await callGraphqlAsCustomer<{ createReview?: MyReviewRow[] }>(
//         userId,
//         CREATE_REVIEW_MUTATION,
//         // comment is a required field on the backend even though we don't collect one yet.
//         { input: { productId, userId, rating, comment: "" } }
//       );
//
//   if (saved.errors?.length) {
//     const { status, message } = graphqlErrorToApiStatus(
//       saved.errors as Parameters<typeof graphqlErrorToApiStatus>[0],
//       "Failed to save your rating"
//     );
//     return apiError(message, status, "GRAPHQL_ERROR");
//   }
//
//   return Response.json({
//     ok: true,
//     data: { rating },
//     errorCode: null,
//     message: null,
//     fieldErrors: null,
//     retryable: false,
//   });
// }
