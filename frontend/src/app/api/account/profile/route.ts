import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type UserPiiExport = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
};

const PROFILE_QUERY = `query AccountProfile {
  exportMyPii {
    userId
    email
    fullName
    address
    phone
    createDate
  }
}`;

export async function GET() {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const result = await callGraphqlAsCustomer<{ exportMyPii?: UserPiiExport }>(
    customerUserId,
    PROFILE_QUERY
  );
  if (result.errors?.length) {
    return apiError(
      result.errors[0]?.message ?? "Failed to load profile",
      400,
      "GRAPHQL_ERROR"
    );
  }

  return Response.json({
    ok: true,
    data: result.data?.exportMyPii ?? null,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
