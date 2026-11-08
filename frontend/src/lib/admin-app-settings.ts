import { gqlAdmin } from "./graphql-client";

export interface AbandonedCartSettingsRow {
  enabled: boolean;
  delayHours: string;
  pollIntervalSec: string;
}

const SETTINGS_FIELDS = `enabled delayHours pollIntervalSec`;

export async function fetchAbandonedCartSettings(): Promise<AbandonedCartSettingsRow | null> {
  const data = await gqlAdmin<{ abandonedCartSettings?: AbandonedCartSettingsRow }>(
    `query AdminAbandonedCartSettings { abandonedCartSettings { ${SETTINGS_FIELDS} } }`
  );
  return data?.abandonedCartSettings ?? null;
}

export async function updateAbandonedCartSettings(params: {
  enabled?: boolean;
  delayHours?: string;
  pollIntervalSec?: string;
}): Promise<AbandonedCartSettingsRow | null> {
  const data = await gqlAdmin<{ updateAbandonedCartSettings?: AbandonedCartSettingsRow }>(
    `mutation AdminUpdateAbandonedCartSettings($input: UpdateAbandonedCartSettingsInput!) {
      updateAbandonedCartSettings(input: $input) { ${SETTINGS_FIELDS} }
    }`,
    { input: params }
  );
  return data?.updateAbandonedCartSettings ?? null;
}
