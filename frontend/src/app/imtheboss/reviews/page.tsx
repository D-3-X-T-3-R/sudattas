"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { MessageSquareText, Star, Trash2 } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/admin/status-badge";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import {
  adminSetReviewStatus,
  deleteReviewAdmin,
  fetchReviewsAdmin,
  type AdminReviewRow,
} from "@/lib/admin-reviews";

const STATUS_TABS = [
  { value: "", label: "All" },
  { value: "pending", label: "Pending" },
  { value: "approved", label: "Approved" },
  { value: "rejected", label: "Rejected" },
] as const;

function formatReviewDate(raw: string): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

export default function AdminReviewsPage() {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState("");
  const [deleteConfirm, setDeleteConfirm] = useState<AdminReviewRow | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const reviewsQuery = useQuery({
    queryKey: ["admin", "reviews", statusFilter],
    queryFn: () => fetchReviewsAdmin(statusFilter || undefined),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "reviews"] });

  const statusMutation = useMutation({
    mutationFn: ({ reviewId, status }: { reviewId: string; status: "approved" | "rejected" }) =>
      adminSetReviewStatus(reviewId, status),
    onSuccess: () => invalidate(),
  });

  const deleteMutation = useMutation({
    mutationFn: (reviewId: string) => deleteReviewAdmin(reviewId),
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete review."),
  });

  const reviews = reviewsQuery.data ?? [];

  return (
    <AdminPageShell
      label="Reviews"
      title="Review moderation"
      description="Approve or reject customer reviews. The storefront only collects a star rating today (no comment box), so Comment is usually empty, but the field is shown in case one is ever set."
    >
      <AdminTableCard
        title="Reviews"
        icon={<MessageSquareText className="h-4 w-4 text-[var(--color-green)]" />}
      >
        <div className="mb-4 flex flex-wrap gap-2">
          {STATUS_TABS.map((tab) => (
            <Button
              key={tab.value}
              type="button"
              size="sm"
              variant={statusFilter === tab.value ? "default" : "outline"}
              onClick={() => setStatusFilter(tab.value)}
            >
              {tab.label}
            </Button>
          ))}
        </div>

        {reviewsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading reviews…</p>
        ) : null}
        {reviewsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load reviews.</p>
        ) : null}
        {!reviewsQuery.isLoading && !reviewsQuery.isError && reviews.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No reviews here.</p>
        ) : null}

        {!reviewsQuery.isLoading && !reviewsQuery.isError && reviews.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[720px] border-collapse text-[15px]">
              <caption className="sr-only">Product reviews awaiting or under moderation</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Product</th>
                  <th className="pb-2 pr-4 font-medium">Customer</th>
                  <th className="pb-2 pr-4 font-medium">Rating</th>
                  <th className="pb-2 pr-4 font-medium">Comment</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 pr-4 font-medium">Date</th>
                  <th className="pb-2 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {reviews.map((r) => (
                  <tr key={r.reviewId} className="border-b border-[var(--color-line)] last:border-0">
                    <td className="py-3 pr-4 text-[var(--color-ink)]">
                      #{r.productId}
                      {r.isVerifiedPurchase ? (
                        <span className="ml-2 rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-800">
                          Verified purchase
                        </span>
                      ) : null}
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">#{r.userId}</td>
                    <td className="py-3 pr-4">
                      <span className="inline-flex items-center gap-1 text-[var(--color-ink)]">
                        <Star className="h-4 w-4 fill-[var(--color-gold)] text-[var(--color-gold)]" />
                        {r.rating}
                      </span>
                    </td>
                    <td className="max-w-[16rem] py-3 pr-4 text-[var(--color-muted)]">
                      {r.comment ? r.comment : <span className="italic">No comment</span>}
                    </td>
                    <td className="py-3 pr-4">
                      <StatusBadge label={r.reviewStatus} />
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{formatReviewDate(r.createdAt)}</td>
                    <td className="py-3">
                      <div className="flex flex-wrap gap-1.5">
                        {r.reviewStatus !== "approved" && (
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            disabled={statusMutation.isPending}
                            onClick={() => statusMutation.mutate({ reviewId: r.reviewId, status: "approved" })}
                          >
                            Approve
                          </Button>
                        )}
                        {r.reviewStatus !== "rejected" && (
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            disabled={statusMutation.isPending}
                            onClick={() => statusMutation.mutate({ reviewId: r.reviewId, status: "rejected" })}
                          >
                            Reject
                          </Button>
                        )}
                        <button
                          type="button"
                          onClick={() => {
                            setDeleteError("");
                            setDeleteConfirm(r);
                          }}
                          aria-label={`Delete review ${r.reviewId}`}
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
        entity={deleteConfirm ? { name: `review #${deleteConfirm.reviewId}` } : null}
        label="review"
        isPending={deleteMutation.isPending}
        error={deleteError}
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.reviewId);
        }}
      />
    </AdminPageShell>
  );
}
