import { gqlAdmin } from "./graphql-client";

export interface ShippingMethodRow {
  methodId: string;
  methodName: string;
  /** Paise (integer minor units) — convert with money.ts helpers before showing/editing. */
  costPaise: string;
  estimatedDeliveryTime: string;
}

const METHOD_FIELDS = `methodId methodName costPaise estimatedDeliveryTime`;

export async function fetchShippingMethods(): Promise<ShippingMethodRow[]> {
  const data = await gqlAdmin<{ searchShippingMethod?: ShippingMethodRow[] }>(
    `query AdminShippingMethods($input: SearchShippingMethod!) {
      searchShippingMethod(input: $input) { ${METHOD_FIELDS} }
    }`,
    { input: {} }
  );
  return data?.searchShippingMethod ?? [];
}

export interface NewShippingMethodInput {
  methodName: string;
  costPaise: string;
  estimatedDeliveryTime: string;
}

export async function createShippingMethod(
  input: NewShippingMethodInput
): Promise<ShippingMethodRow | null> {
  const data = await gqlAdmin<{ createShippingMethod?: ShippingMethodRow[] }>(
    `mutation CreateShippingMethod($input: NewShippingMethod!) {
      createShippingMethod(input: $input) { ${METHOD_FIELDS} }
    }`,
    { input }
  );
  return data?.createShippingMethod?.[0] ?? null;
}

export interface UpdateShippingMethodInput {
  methodId: string;
  methodName?: string;
  costPaise?: string;
  estimatedDeliveryTime?: string;
}

export async function updateShippingMethod(
  input: UpdateShippingMethodInput
): Promise<ShippingMethodRow | null> {
  const data = await gqlAdmin<{ updateShippingMethod?: ShippingMethodRow[] }>(
    `mutation UpdateShippingMethod($input: ShippingMethodMutation!) {
      updateShippingMethod(input: $input) { ${METHOD_FIELDS} }
    }`,
    { input }
  );
  return data?.updateShippingMethod?.[0] ?? null;
}

export async function deleteShippingMethod(methodId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteShippingMethod($methodId: String!) {
      deleteShippingMethod(methodId: $methodId) { methodId }
    }`,
    { methodId }
  );
}
