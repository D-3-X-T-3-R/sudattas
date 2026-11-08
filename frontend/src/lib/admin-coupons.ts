import { gqlAdmin } from "./graphql-client";

export interface AdminCouponRow {
  couponId: string;
  code: string;
  /** "percentage" | "fixed_amount" */
  discountType: string;
  discountValue: number;
  minOrderValuePaise: number | null;
  usageLimit: number | null;
  usageCount: number | null;
  maxUsesPerCustomer: number | null;
  /** "active" | "inactive" */
  status: string;
  startsAt: string;
  endsAt: string | null;
}

const COUPON_FIELDS = `
  couponId
  code
  discountType
  discountValue
  minOrderValuePaise
  usageLimit
  usageCount
  maxUsesPerCustomer
  status
  startsAt
  endsAt
`;

export async function fetchCouponsAdmin(): Promise<AdminCouponRow[]> {
  const data = await gqlAdmin<{ searchCouponAdmin?: AdminCouponRow[] }>(
    `query AdminSearchCoupons($input: SearchCouponAdminInput!) {
      searchCouponAdmin(input: $input) { ${COUPON_FIELDS} }
    }`,
    { input: {} }
  );
  const rows = data?.searchCouponAdmin ?? [];
  return [...rows].sort((a, b) => a.code.localeCompare(b.code));
}

export async function createCouponAdmin(params: {
  code: string;
  discountType: "percentage" | "fixed_amount";
  discountValue: number;
  minOrderValuePaise?: number | null;
  usageLimit?: number | null;
  maxUsesPerCustomer?: number | null;
  startsAt: string;
  endsAt?: string | null;
}): Promise<void> {
  await gqlAdmin(
    `mutation AdminCreateCoupon($input: CreateCouponInput!) { createCouponAdmin(input: $input) }`,
    { input: params }
  );
}

export async function updateCouponAdmin(params: {
  couponId: string;
  status?: "active" | "inactive";
  usageLimit?: number | null;
  endsAt?: string | null;
}): Promise<void> {
  await gqlAdmin(
    `mutation AdminUpdateCoupon($input: UpdateCouponInput!) { updateCouponAdmin(input: $input) }`,
    { input: params }
  );
}

export async function deleteCouponAdmin(couponId: string): Promise<void> {
  await gqlAdmin(
    `mutation AdminDeleteCoupon($input: DeleteCouponAdminInput!) {
      deleteCouponAdmin(input: $input) { couponId }
    }`,
    { input: { couponId } }
  );
}
