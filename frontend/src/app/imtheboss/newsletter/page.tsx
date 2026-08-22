"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Mail, Trash2, Download } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import {
  deleteNewsletterSubscriberAdmin,
  fetchNewsletterSubscribers,
  setNewsletterSubscriberUnsubscribed,
  type NewsletterSubscriberRow,
} from "@/lib/admin-newsletter";

function formatSubDate(raw: string): string {
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

function downloadSubscribersCsv(rows: NewsletterSubscriberRow[]): void {
  const headers = ["Subscriber ID", "Email", "Subscribed on", "Status"];
  const escaped = (v: string) => `"${v.replace(/"/g, '""')}"`;
  const lines = [
    headers.join(","),
    ...rows.map((r) =>
      [
        r.subscriberId,
        escaped(r.email),
        escaped(formatSubDate(r.subscriptionDate)),
        r.unsubscribedAt ? "Unsubscribed" : "Subscribed",
      ].join(",")
    ),
  ];
  const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `newsletter-subscribers-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export default function AdminNewsletterPage() {
  const queryClient = useQueryClient();
  const subsQuery = useQuery({
    queryKey: ["admin", "newsletter-subscribers"],
    queryFn: fetchNewsletterSubscribers,
  });

  const [deleteConfirm, setDeleteConfirm] = useState<NewsletterSubscriberRow | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "newsletter-subscribers"] });

  const toggleMutation = useMutation({
    mutationFn: ({ row, unsubscribed }: { row: NewsletterSubscriberRow; unsubscribed: boolean }) =>
      setNewsletterSubscriberUnsubscribed(row.subscriberId, row.email, unsubscribed),
    onSuccess: () => invalidate(),
  });

  const deleteMutation = useMutation({
    mutationFn: (subscriberId: string) => deleteNewsletterSubscriberAdmin(subscriberId),
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete subscriber."),
  });

  const subscribers = subsQuery.data ?? [];
  const activeCount = subscribers.filter((s) => !s.unsubscribedAt).length;

  return (
    <AdminPageShell
      label="Newsletter"
      title="Newsletter subscribers"
      description={`${activeCount} of ${subscribers.length} subscriber${subscribers.length === 1 ? "" : "s"} currently subscribed.`}
      action={
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={subscribers.length === 0}
          onClick={() => downloadSubscribersCsv(subscribers)}
        >
          <Download className="mr-1.5 h-4 w-4" />
          Export CSV
        </Button>
      }
    >
      <AdminTableCard title="Subscribers" icon={<Mail className="h-4 w-4 text-[var(--color-green)]" />}>
        {subsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading subscribers…</p>
        ) : null}
        {subsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load subscribers.</p>
        ) : null}
        {!subsQuery.isLoading && !subsQuery.isError && subscribers.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No subscribers yet.</p>
        ) : null}

        {!subsQuery.isLoading && !subsQuery.isError && subscribers.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[560px] border-collapse text-[15px]">
              <caption className="sr-only">Newsletter subscribers</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Email</th>
                  <th className="pb-2 pr-4 font-medium">Subscribed on</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {subscribers.map((s) => (
                  <tr key={s.subscriberId} className="border-b border-[var(--color-line)] last:border-0">
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{s.email}</td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{formatSubDate(s.subscriptionDate)}</td>
                    <td className="py-3 pr-4">
                      {s.unsubscribedAt ? (
                        <span className="rounded-full border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-2.5 py-1 text-xs font-medium text-[var(--color-muted)]">
                          Unsubscribed
                        </span>
                      ) : (
                        <span className="rounded-full border border-emerald-200 bg-emerald-50 px-2.5 py-1 text-xs font-medium text-emerald-700">
                          Subscribed
                        </span>
                      )}
                    </td>
                    <td className="py-3">
                      <div className="flex flex-wrap gap-1.5">
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={toggleMutation.isPending}
                          onClick={() =>
                            toggleMutation.mutate({ row: s, unsubscribed: !s.unsubscribedAt })
                          }
                        >
                          {s.unsubscribedAt ? "Resubscribe" : "Unsubscribe"}
                        </Button>
                        <button
                          type="button"
                          onClick={() => {
                            setDeleteError("");
                            setDeleteConfirm(s);
                          }}
                          aria-label={`Delete subscriber ${s.email}`}
                          className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </AdminTableCard>

      <DeleteEntityDialog
        entity={deleteConfirm ? { name: deleteConfirm.email } : null}
        label="subscriber"
        isPending={deleteMutation.isPending}
        error={deleteError}
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.subscriberId);
        }}
      />
    </AdminPageShell>
  );
}
