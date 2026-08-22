"use client";

import { useState } from "react";
import Link from "next/link";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { updateCustomerAdmin, type CustomerListRow } from "@/lib/admin-queries";
import { ExternalLink, Pencil, User } from "lucide-react";
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
        {/* Keyed on userId so switching customers (or reopening after Save) resets the local
         * edit-mode/draft state via remount, instead of syncing it back from props in an effect. */}
        <CustomerProfileDialogBody
          key={selectedCustomer.userId}
          customer={selectedCustomer}
          orderStats={orderStats}
          onUpdated={setSelectedCustomer}
          onClose={() => setSelectedCustomer(null)}
        />
      </DialogContent>
    </Dialog>
  );
}

function CustomerProfileDialogBody({
  customer,
  orderStats,
  onUpdated,
  onClose,
}: {
  customer: CustomerListRow;
  orderStats: Map<string, { count: number; totalPaise: number }>;
  onUpdated: (updated: CustomerListRow) => void;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState(false);
  const [fullName, setFullName] = useState(customer.fullName ?? "");
  const [address, setAddress] = useState(customer.address ?? "");
  const [phone, setPhone] = useState(customer.phone ?? "");
  const [error, setError] = useState("");

  const updateMutation = useMutation({
    mutationFn: () => updateCustomerAdmin({ userId: customer.userId, fullName, address, phone }),
    onSuccess: (updated) => {
      queryClient.invalidateQueries({ queryKey: ["admin", "customers"] });
      if (updated) onUpdated(updated);
      setEditing(false);
      setError("");
    },
    onError: (err: Error) => setError(err.message || "Failed to save changes."),
  });

  return (
    <div className="space-y-4 text-[15px]">
      <div className="flex items-center gap-3 border-b border-[var(--color-line)] pb-4">
        <div className="flex h-14 w-14 items-center justify-center rounded-full bg-[var(--color-line)]/40">
          <User className="h-7 w-7 text-[var(--color-muted)]" />
        </div>
        <div>
          <p className="text-lg font-semibold text-[var(--color-ink)]">
            {customer.fullName ?? customer.username ?? "-"}
          </p>
          <p className="text-[var(--color-muted)]">{customer.email}</p>
        </div>
      </div>
      <div className="flex gap-4 rounded-xl bg-[var(--color-surface)] p-4">
        <div>
          <p className="text-sm text-[var(--color-muted)]">Orders</p>
          <p className="text-xl font-semibold text-[var(--color-ink)]">
            {orderStats.get(customer.userId)?.count ?? 0}
          </p>
        </div>
        <div>
          <p className="text-sm text-[var(--color-muted)]">Total spent</p>
          <p className="text-xl font-semibold text-[var(--color-ink)]">
            {orderStats.get(customer.userId)
              ? formatCurrency(orderStats.get(customer.userId)!.totalPaise)
              : "-"}
          </p>
        </div>
      </div>

      {editing ? (
        <div className="space-y-2.5">
          <label className="block text-sm text-[var(--color-muted)]">
            Full name
            <Input
              value={fullName}
              onChange={(e) => setFullName(e.target.value)}
              className="mt-1 rounded-lg text-[15px]"
            />
          </label>
          <label className="block text-sm text-[var(--color-muted)]">
            Address
            <Input
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              className="mt-1 rounded-lg text-[15px]"
            />
          </label>
          <label className="block text-sm text-[var(--color-muted)]">
            Phone
            <Input
              value={phone}
              onChange={(e) => setPhone(e.target.value)}
              className="mt-1 rounded-lg text-[15px]"
            />
          </label>
          {error && (
            <p className="text-sm text-red-600" role="alert">
              {error}
            </p>
          )}
          <div className="flex gap-2 pt-1">
            <Button
              type="button"
              size="sm"
              disabled={updateMutation.isPending}
              onClick={() => updateMutation.mutate()}
            >
              {updateMutation.isPending ? "Saving…" : "Save changes"}
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={() => setEditing(false)}>
              Cancel
            </Button>
          </div>
        </div>
      ) : (
        <dl className="grid gap-2.5">
          <div>
            <dt className="text-sm text-[var(--color-muted)]">Signed in with</dt>
            <dd className="capitalize text-[var(--color-ink)]">{customer.authProvider}</dd>
          </div>
          <div>
            <dt className="text-sm text-[var(--color-muted)]">Customer since</dt>
            <dd className="text-[var(--color-ink)]">{formatCreateDate(customer.createDate)}</dd>
          </div>
          {customer.address && (
            <div>
              <dt className="text-sm text-[var(--color-muted)]">Address</dt>
              <dd className="text-[var(--color-ink)]">{customer.address}</dd>
            </div>
          )}
          {customer.phone && (
            <div>
              <dt className="text-sm text-[var(--color-muted)]">Phone</dt>
              <dd className="text-[var(--color-ink)]">{customer.phone}</dd>
            </div>
          )}
          <div>
            <dt className="text-sm text-[var(--color-muted)]">Customer ID</dt>
            <dd className="text-[var(--color-muted)]">{customer.userId}</dd>
          </div>
        </dl>
      )}

      {!editing && (
        <div className="flex gap-2 pt-2">
          <Button asChild className="flex-1">
            <Link href={`/imtheboss/orders?userId=${encodeURIComponent(customer.userId)}`}>
              <ExternalLink className="mr-1.5 h-4 w-4" />
              View orders
            </Link>
          </Button>
          <Button type="button" variant="outline" onClick={() => setEditing(true)}>
            <Pencil className="mr-1.5 h-4 w-4" />
            Edit
          </Button>
          <Button type="button" variant="outline" onClick={onClose}>
            Close
          </Button>
        </div>
      )}
    </div>
  );
}
