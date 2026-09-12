// Reviews/ratings are disabled in the frontend for now — see the comment at the top of
// ../product-rating/route.ts. This test is commented out along with the route it exercises;
// uncomment both together to re-enable (and remove the placeholder test below).

import { it } from "vitest";

it.skip("reviews/ratings are disabled — see product-rating/route.ts for how to re-enable", () => {});

// import { beforeEach, describe, expect, it, vi } from "vitest";
//
// const mocks = vi.hoisted(() => ({
//   requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
//   callGraphqlAsCustomer: vi.fn<
//     (
//       userId: string,
//       query: string,
//       variables?: Record<string, unknown>
//     ) => Promise<{ data?: unknown; errors?: Array<{ message?: string }> }>
//   >(),
// }));
//
// vi.mock("@/lib/server-session-auth", () => ({
//   apiError: (message: string, status: number, errorCode: string) =>
//     Response.json(
//       {
//         ok: false,
//         data: null,
//         errorCode,
//         message,
//         fieldErrors: null,
//         retryable: status >= 500,
//       },
//       { status }
//     ),
//   graphqlErrorToApiStatus: (
//     errors: Array<{ message?: string }> | undefined,
//     fallbackMessage: string
//   ) => ({ status: 400, message: errors?.[0]?.message ?? fallbackMessage }),
//   requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
//   callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
// }));
//
// import { GET, POST } from "@/app/api/account/product-rating/route";
//
// function getRequest(productId: string) {
//   return new Request(`http://localhost/api/account/product-rating?productId=${productId}`);
// }
//
// function postRequest(body: Record<string, unknown>) {
//   return new Request("http://localhost/api/account/product-rating", {
//     method: "POST",
//     body: JSON.stringify(body),
//   });
// }
//
// describe("account product-rating route (star rating only, no review text)", () => {
//   beforeEach(() => {
//     mocks.requireAuthenticatedCustomerUserId.mockReset();
//     mocks.callGraphqlAsCustomer.mockReset();
//   });
//
//   it("GET is unauthorized when there is no authenticated customer", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);
//
//     const res = await GET(getRequest("101"));
//     expect(res.status).toBe(401);
//     expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
//   });
//
//   it("GET returns null when the customer has not rated this product", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//     mocks.callGraphqlAsCustomer.mockResolvedValue({ data: { searchReview: [] } });
//
//     const res = await GET(getRequest("101"));
//     const json = (await res.json()) as { data: { rating: number | null } };
//     expect(res.status).toBe(200);
//     expect(json.data.rating).toBeNull();
//   });
//
//   it("GET returns the customer's existing rating", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//     mocks.callGraphqlAsCustomer.mockResolvedValue({
//       data: { searchReview: [{ reviewId: "9", rating: 4 }] },
//     });
//
//     const res = await GET(getRequest("101"));
//     const json = (await res.json()) as { data: { rating: number | null } };
//     expect(json.data.rating).toBe(4);
//   });
//
//   it("POST rejects a rating outside 1-5", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//
//     const res = await POST(postRequest({ productId: "101", rating: 6 }));
//     const json = (await res.json()) as { errorCode: string };
//     expect(res.status).toBe(400);
//     expect(json.errorCode).toBe("VALIDATION_ERROR");
//     expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
//   });
//
//   it("POST rejects a non-integer rating", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//
//     const res = await POST(postRequest({ productId: "101", rating: 3.5 }));
//     expect(res.status).toBe(400);
//   });
//
//   it("POST creates a new review when the customer has not rated this product yet", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//     mocks.callGraphqlAsCustomer.mockImplementation(async (_uid, query) => {
//       if (query.includes("MyProductRating")) return { data: { searchReview: [] } };
//       return { data: { createReview: [{ reviewId: "9", rating: 5 }] } };
//     });
//
//     const res = await POST(postRequest({ productId: "101", rating: 5 }));
//     const json = (await res.json()) as { ok: boolean; data: { rating: number } };
//     expect(res.status).toBe(200);
//     expect(json.ok).toBe(true);
//     expect(json.data.rating).toBe(5);
//     expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(
//       2,
//       "42",
//       expect.stringContaining("CreateProductRating"),
//       { input: { productId: "101", userId: "42", rating: 5, comment: "" } }
//     );
//   });
//
//   it("POST updates the existing review instead of creating a duplicate", async () => {
//     mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("42");
//     mocks.callGraphqlAsCustomer.mockImplementation(async (_uid, query) => {
//       if (query.includes("MyProductRating")) {
//         return { data: { searchReview: [{ reviewId: "9", rating: 2 }] } };
//       }
//       return { data: { updateReview: [{ reviewId: "9", rating: 5 }] } };
//     });
//
//     const res = await POST(postRequest({ productId: "101", rating: 5 }));
//     const json = (await res.json()) as { data: { rating: number } };
//     expect(res.status).toBe(200);
//     expect(json.data.rating).toBe(5);
//     expect(mocks.callGraphqlAsCustomer).toHaveBeenNthCalledWith(
//       2,
//       "42",
//       expect.stringContaining("UpdateProductRating"),
//       { input: { reviewId: "9", rating: 5 } }
//     );
//   });
// });
