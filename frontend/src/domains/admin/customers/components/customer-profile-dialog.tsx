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
        <div className="space-y-4 text-[15px]">
          <div className="flex items-center gap-3 border-b border-[var(--color-line)] pb-4">
            <div className="flex h-14 w-14 items-center justify-center rounded-full bg-[var(--color-line)]/40">
              <User className="h-7 w-7 text-[var(--color-muted)]" />
            </div>
            <div>
              <p className="text-lg font-semibold text-[var(--color-ink)]">
                {selectedCustomer.fullName ?? selectedCustomer.username ?? "-"}
              </p>
              <p className="text-[var(--color-muted)]">{selectedCustomer.email}</p>
            </div>
          </div>
          <div className="flex gap-4 rounded-xl bg-[var(--color-surface)] p-4">
            <div>
              <p className="text-sm text-[var(--color-muted)]">Orders</p>
              <p className="text-xl font-semibold text-[var(--color-ink)]">
                {orderStats.get(selectedCustomer.userId)?.count ?? 0}
              </p>
            </div>
            <div>
              <p className="text-sm text-[var(--color-muted)]">Total spent</p>
              <p className="text-xl font-semibold text-[var(--color-ink)]">
                {orderStats.get(selectedCustomer.userId)
                  ? formatCurrency(orderStats.get(selectedCustomer.userId)!.totalPaise)
                  : "-"}
              </p>
            </div>
          </div>
          <dl className="grid gap-2.5">
            <div>
              <dt className="text-sm text-[var(--color-muted)]">Signed in with</dt>
              <dd className="capitalize text-[var(--color-ink)]">{selectedCustomer.authProvider}</dd>
            </div>
            <div>
              <dt className="text-sm text-[var(--color-muted)]">Customer since</dt>
              <dd className="text-[var(--color-ink)]">{formatCreateDate(selectedCustomer.createDate)}</dd>
            </div>
            {selectedCustomer.address && (
              <div>
                <dt className="text-sm text-[var(--color-muted)]">Address</dt>
                <dd className="text-[var(--color-ink)]">{selectedCustomer.address}</dd>
              </div>
            )}
            {selectedCustomer.phone && (
              <div>
                <dt className="text-sm text-[var(--color-muted)]">Phone</dt>
                <dd className="text-[var(--color-ink)]">{selectedCustomer.phone}</dd>
              </div>
            )}
            <div>
              <dt className="text-sm text-[var(--color-muted)]">Customer ID</dt>
              <dd className="text-[var(--color-muted)]">{selectedCustomer.userId}</dd>
            </div>
          </dl>
          <div className="flex gap-2 pt-2">
            <Button asChild className="flex-1">
              <Link href={`/imtheboss/orders?userId=${encodeURIComponent(selectedCustomer.userId)}`}>
                <ExternalLink className="mr-1.5 h-4 w-4" />
                View orders
              </Link>
            </Button>
            <Button type="button" variant="outline" onClick={() => setSelectedCustomer(null)}>
              Close
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
