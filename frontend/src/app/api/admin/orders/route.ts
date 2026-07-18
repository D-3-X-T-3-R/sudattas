import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const ORDER_ROOTS = [
  "searchOrder",
  "searchOrderStatus",
  "updateOrder",
  "createOrder",
  "updatePickupTarget",
  "resolveNeedsReview",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: ORDER_ROOTS });
}
