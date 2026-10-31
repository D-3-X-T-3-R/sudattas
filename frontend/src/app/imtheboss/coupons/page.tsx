"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Tag } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { CouponCreateForm } from "@/domains/admin/coupons/components/coupon-create-form";
import { CouponRow } from "@/domains/admin/coupons/components/coupon-row";
import {
  deleteCouponAdmin,
  fetchCouponsAdmin,
  updateCouponAdmin,
  type AdminCouponRow,
} from "@/lib/admin-coupons";

export default function AdminCouponsPage() {
  const queryClient = useQueryClient();
  const couponsQuery = useQuery({ queryKey: ["admin", "coupons"], queryFn: fetchCouponsAdmin });

  const [deleteConfirm, setDeleteConfirm] = useState<AdminCouponRow | null>(null);
  const [deleteError, setDeleteError] = useState("");
  const [toggleError, setToggleError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "coupons"] });

  const toggleMutation = useMutation({
    mutationFn: (coupon: AdminCouponRow) =>
      updateCouponAdmin({
        couponId: coupon.couponId,
        status: coupon.status === "active" ? "inactive" : "active",
      }),
    onSuccess: () => {
      invalidate();
      setToggleError("");
    },
    onError: (err: Error) => setToggleError(err.message || "Failed to update coupon status."),
  });

  const deleteMutation = useMutation({
    mutationFn: (couponId: string) => deleteCouponAdmin(couponId),
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete coupon."),
  });

  const coupons = couponsQuery.data ?? [];

  return (
    <AdminPageShell
      label="Coupons"
      title="Coupon codes"
      description="Create, activate/deactivate, and delete checkout coupons. Deleting a coupon with existing redemptions is blocked — deactivate it instead."
    >
      <AdminTableCard title="Coupons" icon={<Tag className="h-4 w-4 text-[var(--color-green)]" />}>
        {couponsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading coupons…</p>
        ) : null}
        {couponsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load coupons.</p>
        ) : null}
        {!couponsQuery.isLoading && !couponsQuery.isError && coupons.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No coupons yet. Create one below.</p>
        ) : null}

        {!couponsQuery.isLoading && !couponsQuery.isError && coupons.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[760px] border-collapse text-[15px]">
              <caption className="sr-only">Coupon codes</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Code</th>
                  <th className="pb-2 pr-4 font-medium">Discount</th>
                  <th className="pb-2 pr-4 font-medium">Min order</th>
                  <th className="pb-2 pr-4 font-medium">Usage</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 pr-4 font-medium">Window</th>
                  <th className="pb-2 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {coupons.map((coupon) => (
                  <CouponRow
                    key={coupon.couponId}
                    coupon={coupon}
                    isToggling={toggleMutation.isPending}
                    onToggleStatus={() => toggleMutation.mutate(coupon)}
                    onRequestDelete={() => {
                      setDeleteError("");
                      setDeleteConfirm(coupon);
                    }}
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
        {toggleError && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {toggleError}
          </p>
        )}

        <CouponCreateForm />
      </AdminTableCard>

      <DeleteEntityDialog
        entity={deleteConfirm ? { name: deleteConfirm.code } : null}
        label="coupon"
        isPending={deleteMutation.isPending}
        error={deleteError}
        warning="This cannot be undone. Coupons with existing redemptions can't be deleted — deactivate instead."
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.couponId);
        }}
      />
    </AdminPageShell>
  );
}
