"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { History } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { createAdminOrderNote, type AdminOrderEvent } from "@/lib/admin-order-detail";
import { formatOrderDate } from "@/domains/admin/orders/utils";

function actorLabel(actorType: string): string {
  const t = actorType.trim().toLowerCase();
  if (t === "admin") return "Admin";
  if (t === "customer") return "Customer";
  if (t === "system") return "System";
  return actorType || "Unknown";
}

function eventSummary(event: AdminOrderEvent): string {
  if (event.fromStatus && event.toStatus && event.fromStatus !== event.toStatus) {
    return `${event.fromStatus} → ${event.toStatus}`;
  }
  return event.eventType.replace(/_/g, " ");
}

type OrderTimelineCardProps = {
  orderId: string;
  events: AdminOrderEvent[];
};

export function OrderTimelineCard({ orderId, events }: OrderTimelineCardProps) {
  const queryClient = useQueryClient();
  const [noteDraft, setNoteDraft] = useState("");

  const addNoteMutation = useMutation({
    mutationFn: (message: string) => createAdminOrderNote(orderId, message),
    onSuccess: () => {
      setNoteDraft("");
      queryClient.invalidateQueries({ queryKey: ["admin", "order", orderId] });
    },
  });

  return (
    <Card className="bg-[var(--admin-surface-muted)]">
      <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
        <History className="h-4 w-4 text-[var(--color-green)]" />
        Timeline
      </CardTitle>
      <CardContent className="mt-4">
        {events.length === 0 ? (
          <p className="py-4 text-center text-sm text-[var(--color-muted)]">No events recorded yet.</p>
        ) : (
          <ol className="space-y-4 border-l border-[var(--color-line)] pl-4">
            {events.map((event) => (
              <li key={event.eventId} className="relative">
                <span className="absolute -left-[1.35rem] top-1 h-2 w-2 rounded-full bg-[var(--color-green)]" />
                <p className="text-[15px] font-medium text-[var(--color-ink)]">{eventSummary(event)}</p>
                <p className="mt-0.5 text-xs text-[var(--color-muted)]">
                  {formatOrderDate(event.createdAt)} &middot; {actorLabel(event.actorType)}
                </p>
                {event.message ? (
                  <p className="mt-1 text-sm text-[var(--color-ink)]">{event.message}</p>
                ) : null}
              </li>
            ))}
          </ol>
        )}

        <div className="mt-5 border-t border-[var(--color-line)] pt-4">
          <label htmlFor="admin-order-note" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
            Add a note
          </label>
          <textarea
            id="admin-order-note"
            value={noteDraft}
            onChange={(e) => setNoteDraft(e.target.value)}
            placeholder="e.g. Called customer to confirm delivery slot"
            rows={2}
            className="w-full rounded-lg border border-[var(--color-line)] bg-white px-3 py-2 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          />
          <Button
            type="button"
            size="sm"
            className="mt-2"
            disabled={addNoteMutation.isPending || !noteDraft.trim()}
            onClick={() => addNoteMutation.mutate(noteDraft.trim())}
          >
            {addNoteMutation.isPending ? "Adding…" : "Add note"}
          </Button>
          {addNoteMutation.isError ? (
            <p className="mt-2 text-sm text-rose-700" role="alert">
              {addNoteMutation.error instanceof Error
                ? addNoteMutation.error.message
                : "Could not add note."}
            </p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}
