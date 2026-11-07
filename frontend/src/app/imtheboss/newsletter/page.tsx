"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Mail, Trash2, Download, Send } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import {
  deleteNewsletterSubscriberAdmin,
  fetchNewsletterCampaigns,
  fetchNewsletterSubscribers,
  sendNewsletterCampaign,
  setNewsletterSubscriberUnsubscribed,
  type NewsletterCampaignRow,
  type NewsletterSubscriberRow,
} from "@/lib/admin-newsletter";

function formatSubDate(raw: string): string {
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

function formatSentAt(raw: string): string {
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

function ComposeCampaignCard({ activeCount }: { activeCount: number }) {
  const queryClient = useQueryClient();
  const [subject, setSubject] = useState("");
  const [bodyText, setBodyText] = useState("");
  const [ctaLabel, setCtaLabel] = useState("");
  const [ctaUrl, setCtaUrl] = useState("");
  const [confirmOpen, setConfirmOpen] = useState(false);

  const sendMutation = useMutation({
    mutationFn: () => sendNewsletterCampaign({ subject, bodyText, ctaLabel, ctaUrl }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "newsletter-campaigns"] });
      setConfirmOpen(false);
      setSubject("");
      setBodyText("");
      setCtaLabel("");
      setCtaUrl("");
    },
  });

  const canSend = subject.trim().length > 0 && bodyText.trim().length > 0 && activeCount > 0;

  return (
    <AdminTableCard title="Compose campaign" icon={<Send className="h-4 w-4 text-[var(--color-green)]" />}>
      <p className="mb-4 text-sm text-[var(--color-muted)]">
        Sends immediately to all {activeCount} currently-subscribed email{activeCount === 1 ? "" : "s"} —
        there's no draft or scheduled state. Recipients get it wrapped in the site's branded
        template automatically, with a working unsubscribe link.
      </p>
      <div className="space-y-4">
        <label className="block text-sm text-[var(--color-muted)]">
          Subject
          <input
            type="text"
            value={subject}
            onChange={(e) => setSubject(e.target.value)}
            placeholder="This week's new arrivals"
            className="mt-1 block h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
        </label>
        <label className="block text-sm text-[var(--color-muted)]">
          Message
          <textarea
            value={bodyText}
            onChange={(e) => setBodyText(e.target.value)}
            rows={6}
            placeholder={"Hi there,\n\nWe just added a new collection of hand-woven sarees...\n\nLeave a blank line between paragraphs."}
            className="mt-1 block w-full rounded-lg border border-[var(--color-line)] bg-white px-3 py-2.5 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
        </label>
        <div className="grid gap-4 sm:grid-cols-2">
          <label className="block text-sm text-[var(--color-muted)]">
            Button text (optional)
            <input
              type="text"
              value={ctaLabel}
              onChange={(e) => setCtaLabel(e.target.value)}
              placeholder="Shop now"
              className="mt-1 block h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            />
          </label>
          <label className="block text-sm text-[var(--color-muted)]">
            Button link (optional)
            <input
              type="url"
              value={ctaUrl}
              onChange={(e) => setCtaUrl(e.target.value)}
              placeholder="https://sudattas.com/collections/new"
              className="mt-1 block h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            />
          </label>
        </div>
      </div>

      <div className="mt-5 flex items-center gap-3">
        <Button type="button" disabled={!canSend} onClick={() => setConfirmOpen(true)}>
          Send to {activeCount} subscriber{activeCount === 1 ? "" : "s"}
        </Button>
        {activeCount === 0 ? (
          <span className="text-sm text-[var(--color-muted)]">No active subscribers yet.</span>
        ) : null}
      </div>
      {sendMutation.isError ? (
        <p className="mt-2 text-sm text-rose-700" role="alert">
          {sendMutation.error instanceof Error
            ? sendMutation.error.message
            : "Could not send campaign."}
        </p>
      ) : null}

      <Dialog open={confirmOpen} onOpenChange={(open) => !open && !sendMutation.isPending && setConfirmOpen(false)}>
        <DialogContent className="sm:max-w-md">
          <p className="text-[15px] leading-relaxed text-[var(--color-ink)]">
            Send <strong>&ldquo;{subject}&rdquo;</strong> to all {activeCount} subscribed email
            {activeCount === 1 ? "" : "s"} now? This sends for real — there's no preview or undo
            once it starts.
          </p>
          <div className="mt-5 flex justify-end gap-2">
            <Button variant="outline" onClick={() => setConfirmOpen(false)} disabled={sendMutation.isPending}>
              Cancel
            </Button>
            <Button onClick={() => sendMutation.mutate()} disabled={sendMutation.isPending}>
              {sendMutation.isPending ? "Sending…" : "Send now"}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </AdminTableCard>
  );
}

function CampaignHistoryCard() {
  const campaignsQuery = useQuery({
    queryKey: ["admin", "newsletter-campaigns"],
    queryFn: fetchNewsletterCampaigns,
  });
  const campaigns = campaignsQuery.data ?? [];

  return (
    <AdminTableCard title="Campaign history">
      {campaignsQuery.isLoading ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading campaigns…</p>
      ) : null}
      {campaignsQuery.isError ? (
        <p className="py-8 text-center text-sm text-rose-700">Could not load campaign history.</p>
      ) : null}
      {!campaignsQuery.isLoading && !campaignsQuery.isError && campaigns.length === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">
          Nothing sent yet — compose one above.
        </p>
      ) : null}
      {!campaignsQuery.isLoading && !campaignsQuery.isError && campaigns.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] border-collapse text-[15px]">
            <caption className="sr-only">Newsletter campaign history</caption>
            <thead>
              <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                <th className="pb-2 pr-4 font-medium">Subject</th>
                <th className="pb-2 pr-4 font-medium">Sent</th>
                <th className="pb-2 pr-4 font-medium">Recipients</th>
                <th className="pb-2 font-medium">Result</th>
              </tr>
            </thead>
            <tbody>
              {campaigns.map((c: NewsletterCampaignRow) => (
                <tr key={c.campaignId} className="border-b border-[var(--color-line)] last:border-0">
                  <td className="py-3 pr-4 text-[var(--color-ink)]">{c.subject}</td>
                  <td className="py-3 pr-4 text-[var(--color-muted)]">{formatSentAt(c.sentAt)}</td>
                  <td className="py-3 pr-4 text-[var(--color-muted)]">{c.recipientCount}</td>
                  <td className="py-3">
                    <span className="rounded-full border border-emerald-200 bg-emerald-50 px-2.5 py-1 text-xs font-medium text-emerald-700">
                      {c.successCount} sent
                    </span>
                    {Number(c.failureCount) > 0 ? (
                      <span className="ml-1.5 rounded-full border border-red-200 bg-red-50 px-2.5 py-1 text-xs font-medium text-red-700">
                        {c.failureCount} failed
                      </span>
                    ) : null}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}
    </AdminTableCard>
  );
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
      title="Newsletter"
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
      <div className="space-y-6">
        <ComposeCampaignCard activeCount={activeCount} />
        <CampaignHistoryCard />

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
      </div>

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
