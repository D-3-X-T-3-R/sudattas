import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const ORDER_ROOTS = [
  "searchOrder",
  "searchOrderStatus",
  "updateOrder",
  "deleteOrder",
  "createOrder",
  "createOrderAdmin",
  "createOrderDetails",
  "placeOrderAdmin",
  "updatePickupTarget",
  "resolveNeedsReview",
  "orderStats",
  "getOrderEvents",
  "createOrderEvent",
  "getShippingAddresses",
  "createShippingAddress",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: ORDER_ROOTS });
}
