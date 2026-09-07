import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const COUPON_ROOTS = [
  "searchCouponAdmin",
  "createCouponAdmin",
  "updateCouponAdmin",
  "deleteCouponAdmin",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: COUPON_ROOTS });
}
