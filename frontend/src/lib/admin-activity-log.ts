import { gqlAdmin } from "./graphql-client";

export interface UserActivityRow {
  activityId: string;
  userId: string;
  activityType: string;
  activityTime: string;
  activityDetails: string;
}

export interface EventLogRow {
  logId: string;
  eventType: string;
  eventDescription: string;
  userId: string;
  eventTime: string;
}

const ACTIVITY_FIELDS = `activityId userId activityType activityTime activityDetails`;
const EVENT_LOG_FIELDS = `logId eventType eventDescription userId eventTime`;

export async function fetchUserActivities(): Promise<UserActivityRow[]> {
  const data = await gqlAdmin<{ searchUserActivity?: UserActivityRow[] }>(
    `mutation UserActivities { searchUserActivity(input: { activityId: "0" }) { ${ACTIVITY_FIELDS} } }`
  );
  const rows = data?.searchUserActivity ?? [];
  return [...rows].sort((a, b) => new Date(b.activityTime).getTime() - new Date(a.activityTime).getTime());
}

export async function deleteUserActivity(activityId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteUserActivity($input: DeleteUserActivityInput!) {
      deleteUserActivity(input: $input) { activityId }
    }`,
    { input: { activityId } }
  );
}

export async function fetchEventLogs(): Promise<EventLogRow[]> {
  const data = await gqlAdmin<{ searchEventLog?: EventLogRow[] }>(
    `mutation EventLogs { searchEventLog(input: { logId: "0" }) { ${EVENT_LOG_FIELDS} } }`
  );
  const rows = data?.searchEventLog ?? [];
  return [...rows].sort((a, b) => new Date(b.eventTime).getTime() - new Date(a.eventTime).getTime());
}

export async function deleteEventLog(logId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteEventLog($input: DeleteEventLogInput!) { deleteEventLog(input: $input) { logId } }`,
    { input: { logId } }
  );
}
