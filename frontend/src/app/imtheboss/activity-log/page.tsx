"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Activity, ScrollText, Trash2 } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import {
  deleteEventLog,
  deleteUserActivity,
  fetchEventLogs,
  fetchUserActivities,
  type EventLogRow,
  type UserActivityRow,
} from "@/lib/admin-activity-log";

function formatLogTime(raw: string): string {
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleString("en-IN", {
    day: "2-digit",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function UserActivityTable({ rows, onDelete, isDeleting }: {
  rows: UserActivityRow[];
  onDelete: (id: string) => void;
  isDeleting: boolean;
}) {
  if (rows.length === 0) {
    return <p className="py-6 text-center text-sm text-[var(--color-muted)]">No activity recorded yet.</p>;
  }
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[600px] border-collapse text-[15px]">
        <caption className="sr-only">User activity log</caption>
        <thead>
          <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
            <th className="pb-2 pr-4 font-medium">When</th>
            <th className="pb-2 pr-4 font-medium">User</th>
            <th className="pb-2 pr-4 font-medium">Type</th>
            <th className="pb-2 pr-4 font-medium">Details</th>
            <th className="pb-2 font-medium sr-only">Actions</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.activityId} className="border-b border-[var(--color-line)] last:border-0">
              <td className="py-2.5 pr-4 text-[var(--color-muted)]">{formatLogTime(row.activityTime)}</td>
              <td className="py-2.5 pr-4 text-[var(--color-ink)]">#{row.userId}</td>
              <td className="py-2.5 pr-4 text-[var(--color-ink)] capitalize">{row.activityType}</td>
              <td className="py-2.5 pr-4 text-[var(--color-muted)]">{row.activityDetails}</td>
              <td className="py-2.5">
                <button
                  type="button"
                  onClick={() => onDelete(row.activityId)}
                  disabled={isDeleting}
                  aria-label="Delete activity entry"
                  className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function EventLogTable({ rows, onDelete, isDeleting }: {
  rows: EventLogRow[];
  onDelete: (id: string) => void;
  isDeleting: boolean;
}) {
  if (rows.length === 0) {
    return <p className="py-6 text-center text-sm text-[var(--color-muted)]">No system events logged yet.</p>;
  }
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[600px] border-collapse text-[15px]">
        <caption className="sr-only">System event log</caption>
        <thead>
          <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
            <th className="pb-2 pr-4 font-medium">When</th>
            <th className="pb-2 pr-4 font-medium">User</th>
            <th className="pb-2 pr-4 font-medium">Event</th>
            <th className="pb-2 pr-4 font-medium">Description</th>
            <th className="pb-2 font-medium sr-only">Actions</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.logId} className="border-b border-[var(--color-line)] last:border-0">
              <td className="py-2.5 pr-4 text-[var(--color-muted)]">{formatLogTime(row.eventTime)}</td>
              <td className="py-2.5 pr-4 text-[var(--color-ink)]">#{row.userId}</td>
              <td className="py-2.5 pr-4 text-[var(--color-ink)] capitalize">{row.eventType}</td>
              <td className="py-2.5 pr-4 text-[var(--color-muted)]">{row.eventDescription}</td>
              <td className="py-2.5">
                <button
                  type="button"
                  onClick={() => onDelete(row.logId)}
                  disabled={isDeleting}
                  aria-label="Delete event log entry"
                  className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function AdminActivityLogPage() {
  const queryClient = useQueryClient();
  const activitiesQuery = useQuery({ queryKey: ["admin", "user-activities"], queryFn: fetchUserActivities });
  const eventLogsQuery = useQuery({ queryKey: ["admin", "event-logs"], queryFn: fetchEventLogs });

  const deleteActivityMutation = useMutation({
    mutationFn: (activityId: string) => deleteUserActivity(activityId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["admin", "user-activities"] }),
  });

  const deleteEventLogMutation = useMutation({
    mutationFn: (logId: string) => deleteEventLog(logId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["admin", "event-logs"] }),
  });

  return (
    <AdminPageShell
      label="Audit"
      title="Activity & event log"
      description="A read-mostly audit trail. Nothing in checkout, fulfilment, or admin actions writes here automatically yet — entries only appear if something calls the logging APIs directly."
    >
      <div className="space-y-6">
        <AdminTableCard title="User activity" icon={<Activity className="h-4 w-4 text-[var(--color-green)]" />}>
          {activitiesQuery.isLoading ? (
            <p className="py-6 text-center text-sm text-[var(--color-muted)]">Loading…</p>
          ) : null}
          {activitiesQuery.isError ? (
            <p className="py-6 text-center text-sm text-rose-700">Could not load user activity.</p>
          ) : null}
          {!activitiesQuery.isLoading && !activitiesQuery.isError ? (
            <UserActivityTable
              rows={activitiesQuery.data ?? []}
              onDelete={(id) => deleteActivityMutation.mutate(id)}
              isDeleting={deleteActivityMutation.isPending}
            />
          ) : null}
        </AdminTableCard>

        <AdminTableCard title="System events" icon={<ScrollText className="h-4 w-4 text-[var(--color-green)]" />}>
          {eventLogsQuery.isLoading ? (
            <p className="py-6 text-center text-sm text-[var(--color-muted)]">Loading…</p>
          ) : null}
          {eventLogsQuery.isError ? (
            <p className="py-6 text-center text-sm text-rose-700">Could not load event log.</p>
          ) : null}
          {!eventLogsQuery.isLoading && !eventLogsQuery.isError ? (
            <EventLogTable
              rows={eventLogsQuery.data ?? []}
              onDelete={(id) => deleteEventLogMutation.mutate(id)}
              isDeleting={deleteEventLogMutation.isPending}
            />
          ) : null}
        </AdminTableCard>
      </div>
    </AdminPageShell>
  );
}
