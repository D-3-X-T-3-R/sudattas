import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const SHIPPING_METHOD_ROOTS = [
  "searchShippingMethod",
  "createShippingMethod",
  "updateShippingMethod",
  "deleteShippingMethod",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: SHIPPING_METHOD_ROOTS });
}
