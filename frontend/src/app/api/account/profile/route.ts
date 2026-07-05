import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import { profileUpdateSchema } from "@/lib/validation-schemas";

type UserPiiExport = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
  firstName?: string | null;
  lastName?: string | null;
  gender?: string | null;
  dateOfBirth?: string | null;
};

const PROFILE_QUERY = `query AccountProfile {
  exportMyPii {
    userId
    email
    fullName
    address
    phone
    createDate
    firstName
    lastName
    gender
    dateOfBirth
  }
}`;

const UPDATE_PROFILE_MUTATION = `mutation UpdateAccountProfile($input: UpdateUserInput!) {
  updateUser(input: $input) {
    userId
    email
    fullName
    address
    phone
    createDate
    firstName
    lastName
    gender
    dateOfBirth
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

export async function PATCH(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    input?: Record<string, unknown>;
  };
  if (!body.input) return apiError("Missing profile input", 400, "BAD_REQUEST");

  const parsed = profileUpdateSchema.safeParse(body.input);
  if (!parsed.success) {
    return apiError(parsed.error.issues[0]?.message ?? "Invalid profile input", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{ updateUser?: UserPiiExport[] }>(
    customerUserId,
    UPDATE_PROFILE_MUTATION,
    {
      input: {
        userId: customerUserId,
        firstName: parsed.data.firstName,
        lastName: parsed.data.lastName || null,
        gender: parsed.data.gender ?? null,
        dateOfBirth: parsed.data.dateOfBirth ?? null,
        phone: parsed.data.phoneNumber,
      },
    }
  );
  if (result.errors?.length) {
    return apiError(result.errors[0]?.message ?? "Failed to update profile", 400, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.updateUser?.[0] ?? null,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
