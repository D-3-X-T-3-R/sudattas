import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import { addressInputSchema } from "@/lib/validation-schemas";

type ShippingAddressRow = {
  shippingAddressId: string;
  userId?: string | null;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string | null;
  apartmentNoOrName?: string | null;
};

const LIST_QUERY = `query AccountAddressList {
  getShippingAddresses {
    shippingAddressId
    userId
    country
    stateRegion
    city
    postalCode
    road
    apartmentNoOrName
  }
}`;

const CREATE_MUTATION = `mutation CreateAccountAddress($input: NewShippingAddress!) {
  createShippingAddress(input: $input) {
    shippingAddressId
    userId
    country
    stateRegion
    city
    postalCode
    road
    apartmentNoOrName
  }
}`;

const UPDATE_MUTATION = `mutation UpdateAccountAddress($input: ShippingAddressMutation!) {
  updateShippingAddress(input: $input) {
    shippingAddressId
    userId
    country
    stateRegion
    city
    postalCode
    road
    apartmentNoOrName
  }
}`;

const DELETE_MUTATION = `mutation DeleteAccountAddress($shippingAddressId: String!) {
  deleteShippingAddress(shippingAddressId: $shippingAddressId) {
    shippingAddressId
  }
}`;

export async function GET() {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const result = await callGraphqlAsCustomer<{ getShippingAddresses?: ShippingAddressRow[] }>(
    customerUserId,
    LIST_QUERY
  );
  if (result.errors?.length) {
    return apiError(result.errors[0]?.message ?? "Failed to load addresses", 400, "GRAPHQL_ERROR");
  }
  return Response.json({
    ok: true,
    data: result.data?.getShippingAddresses ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function POST(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    input?: Record<string, unknown>;
  };
  if (!body.input) return apiError("Missing address input", 400, "BAD_REQUEST");
  const parsed = addressInputSchema.safeParse(body.input);
  if (!parsed.success) {
    return apiError(parsed.error.issues[0]?.message ?? "Invalid address input", 400, "VALIDATION_ERROR");
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();
  const result = await callGraphqlAsCustomer<{ createShippingAddress?: ShippingAddressRow[] }>(
    customerUserId,
    CREATE_MUTATION,
    { input: parsed.data },
    idempotencyKey ? { "Idempotency-Key": idempotencyKey } : {}
  );
  if (result.errors?.length) {
    return apiError(result.errors[0]?.message ?? "Failed to create address", 400, "GRAPHQL_ERROR");
  }
  return Response.json({
    ok: true,
    data: result.data?.createShippingAddress?.[0] ?? null,
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
  if (!body.input) return apiError("Missing address update input", 400, "BAD_REQUEST");
  const candidate = body.input as Record<string, unknown>;
  const shippingAddressId = String(candidate.shippingAddressId ?? "").trim();
  if (!shippingAddressId) return apiError("Missing shippingAddressId", 400, "BAD_REQUEST");
  const parsed = addressInputSchema.safeParse({
    country: candidate.country,
    stateRegion: candidate.stateRegion,
    city: candidate.city,
    postalCode: candidate.postalCode,
    road: candidate.road,
    apartmentNoOrName: candidate.apartmentNoOrName ?? null,
  });
  if (!parsed.success) {
    return apiError(parsed.error.issues[0]?.message ?? "Invalid address input", 400, "VALIDATION_ERROR");
  }
  const mutationInput = { shippingAddressId, ...parsed.data };

  const result = await callGraphqlAsCustomer<{ updateShippingAddress?: ShippingAddressRow[] }>(
    customerUserId,
    UPDATE_MUTATION,
    { input: mutationInput }
  );
  if (result.errors?.length) {
    return apiError(result.errors[0]?.message ?? "Failed to update address", 400, "GRAPHQL_ERROR");
  }
  return Response.json({
    ok: true,
    data: result.data?.updateShippingAddress?.[0] ?? null,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function DELETE(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    shippingAddressId?: string;
  };
  const shippingAddressId = body.shippingAddressId?.trim();
  if (!shippingAddressId) {
    return apiError("Missing shippingAddressId", 400, "BAD_REQUEST");
  }

  const result = await callGraphqlAsCustomer<{ deleteShippingAddress?: Array<{ shippingAddressId: string }> }>(
    customerUserId,
    DELETE_MUTATION,
    { shippingAddressId }
  );
  if (result.errors?.length) {
    return apiError(result.errors[0]?.message ?? "Failed to delete address", 400, "GRAPHQL_ERROR");
  }
  return Response.json({
    ok: true,
    data: true,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
