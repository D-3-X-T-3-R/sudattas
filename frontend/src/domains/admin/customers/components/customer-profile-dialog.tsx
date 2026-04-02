"use client";

import Link from "next/link";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { CustomerListRow } from "@/lib/admin-queries";
import { ExternalLink, User } from "lucide-react";
import { formatCreateDate, formatCurrency } from "@/domains/admin/customers/utils";

type CustomerProfileDialogProps = {
  selectedCustomer: CustomerListRow | null;
  setSelectedCustomer: (value: CustomerListRow | null) => void;
  orderStats: Map<string, { count: number; totalPaise: number }>;
};

export function CustomerProfileDialog({
  selectedCustomer,
  setSelectedCustomer,
  orderStats,
}: CustomerProfileDialogProps) {
  if (!selectedCustomer) return null;

  return (
    <Dialog open={!!selectedCustomer} onOpenChange={(open) => !open && setSelectedCustomer(null)}>
      <DialogContent title="Customer profile" className="sm:max-w-md">
        <div className="space-y-4 text-sm">
          <div className="flex items-center gap-3 border-b border-[var(--color-line)] pb-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-[var(--color-line)]/40">
              <User className="h-6 w-6 text-[var(--color-muted)]" />
            </div>
            <div>
              <p className="font-medium text-[var(--color-ink)]">
                {selectedCustomer.fullName ?? selectedCustomer.username ?? "-"}
              </p>
              <p className="text-[var(--color-muted)]">{selectedCustomer.email}</p>
            </div>
          </div>
          <dl className="grid gap-2">
            <div>
              <dt className="text-xs text-[var(--color-muted)]">User ID</dt>
              <dd className="font-mono text-[var(--color-ink)]">{selectedCustomer.userId}</dd>
            </div>
            <div>
              <dt className="text-xs text-[var(--color-muted)]">Auth</dt>
              <dd className="text-[var(--color-ink)]">{selectedCustomer.authProvider}</dd>
            </div>
            <div>
              <dt className="text-xs text-[var(--color-muted)]">Created</dt>
              <dd className="text-[var(--color-ink)]">{formatCreateDate(selectedCustomer.createDate)}</dd>
            </div>
            {selectedCustomer.address && (
              <div>
                <dt className="text-xs text-[var(--color-muted)]">Address</dt>
                <dd className="text-[var(--color-ink)]">{selectedCustomer.address}</dd>
              </div>
            )}
            {selectedCustomer.phone && (
              <div>
                <dt className="text-xs text-[var(--color-muted)]">Phone</dt>
                <dd className="text-[var(--color-ink)]">{selectedCustomer.phone}</dd>
              </div>
            )}
          </dl>
          <div className="flex gap-4 rounded-lg bg-[var(--color-surface)] p-3">
            <div>
              <p className="text-xs text-[var(--color-muted)]">Orders</p>
              <p className="text-lg font-medium text-[var(--color-ink)]">
                {orderStats.get(selectedCustomer.userId)?.count ?? 0}
              </p>
            </div>
            <div>
              <p className="text-xs text-[var(--color-muted)]">Total spent</p>
              <p className="text-lg font-medium text-[var(--color-ink)]">
                {orderStats.get(selectedCustomer.userId)
                  ? formatCurrency(orderStats.get(selectedCustomer.userId)!.totalPaise)
                  : "-"}
              </p>
            </div>
          </div>
          <div className="flex gap-2 pt-2">
            <Button asChild size="sm" className="flex-1">
              <Link href={`/imtheboss/orders?userId=${encodeURIComponent(selectedCustomer.userId)}`}>
                <ExternalLink className="mr-1.5 h-4 w-4" />
                View orders
              </Link>
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={() => setSelectedCustomer(null)}>
              Close
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
