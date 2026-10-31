"use client";

import { Button } from "@/components/ui/button";
import { parseTrackingEvents, type AdminShipmentRow } from "@/lib/admin-shipments";
import { formatOrderDate } from "@/domains/admin/orders/utils";

interface OrderShipmentRowProps {
  shipment: AdminShipmentRow;
  onEdit: () => void;
}

export function OrderShipmentRow({ shipment: s, onEdit }: OrderShipmentRowProps) {
  const events = parseTrackingEvents(s.trackingEventsJson);
  return (
    <div className="rounded-lg border border-[var(--color-line)] p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-[15px] font-medium text-[var(--color-ink)]">
          {s.customerTrackingStatus || s.status}
        </p>
        <Button type="button" variant="outline" size="sm" onClick={onEdit}>
          Edit
        </Button>
      </div>
      <dl className="mt-2 grid gap-1.5 text-sm sm:grid-cols-2">
        <div>
          <dt className="text-[var(--color-muted)]">AWB code</dt>
          <dd className="text-[var(--color-ink)]">{s.awbCode || "—"}</dd>
        </div>
        <div>
          <dt className="text-[var(--color-muted)]">Carrier</dt>
          <dd className="text-[var(--color-ink)]">{s.carrier || "—"}</dd>
        </div>
        <div>
          <dt className="text-[var(--color-muted)]">Created</dt>
          <dd className="text-[var(--color-ink)]">{formatOrderDate(s.createdAt)}</dd>
        </div>
        <div>
          <dt className="text-[var(--color-muted)]">Delivered</dt>
          <dd className="text-[var(--color-ink)]">
            {s.deliveredAt ? formatOrderDate(s.deliveredAt) : "—"}
          </dd>
        </div>
      </dl>
      {events.length > 0 && (
        <ol className="mt-3 space-y-1.5 border-t border-[var(--color-line)] pt-2.5 text-sm">
          {events.map((e, i) => (
            <li key={i} className="text-[var(--color-ink)]">
              <span className="font-medium">{e.label || "Update"}</span>
              {e.at ? <span className="text-[var(--color-muted)]"> — {formatOrderDate(e.at)}</span> : null}
              {e.location ? <span className="text-[var(--color-muted)]"> ({e.location})</span> : null}
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}
