"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Truck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import {
  createShipmentAdmin,
  fetchShipmentsForOrder,
  syncShipmentsFromShiprocketAdmin,
  updateShipmentAdmin,
} from "@/lib/admin-shipments";
import { OrderShipmentRow } from "@/domains/admin/orders/components/order-shipment-row";

interface OrderShipmentCardProps {
  orderId: string;
}

export function OrderShipmentCard({ orderId }: OrderShipmentCardProps) {
  const queryClient = useQueryClient();
  const queryKey = ["admin", "order-shipments", orderId];
  const [awbDraft, setAwbDraft] = useState("");
  const [carrierDraft, setCarrierDraft] = useState("");
  const [formError, setFormError] = useState("");
  const [editingShipmentId, setEditingShipmentId] = useState<string | null>(null);

  const shipmentsQuery = useQuery({ queryKey, queryFn: () => fetchShipmentsForOrder(orderId) });

  const invalidate = () => queryClient.invalidateQueries({ queryKey });

  const createMutation = useMutation({
    mutationFn: () =>
      createShipmentAdmin({
        orderId,
        awbCode: awbDraft.trim() || undefined,
        carrier: carrierDraft.trim() || undefined,
      }),
    onSuccess: () => {
      invalidate();
      setAwbDraft("");
      setCarrierDraft("");
      setFormError("");
    },
    onError: (err: Error) => setFormError(err.message || "Failed to create shipment."),
  });

  const updateMutation = useMutation({
    mutationFn: (shipmentId: string) =>
      updateShipmentAdmin({
        shipmentId,
        awbCode: awbDraft.trim() || undefined,
        carrier: carrierDraft.trim() || undefined,
      }),
    onSuccess: () => {
      invalidate();
      setEditingShipmentId(null);
      setAwbDraft("");
      setCarrierDraft("");
      setFormError("");
    },
    onError: (err: Error) => setFormError(err.message || "Failed to update shipment."),
  });

  const syncMutation = useMutation({
    mutationFn: () => syncShipmentsFromShiprocketAdmin(orderId),
    onSuccess: () => invalidate(),
    onError: (err: Error) => setFormError(err.message || "Failed to sync from Shiprocket."),
  });

  const shipments = shipmentsQuery.data ?? [];

  return (
    <Card className="bg-[var(--admin-surface-muted)]">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
          <Truck className="h-4 w-4 text-[var(--color-green)]" />
          Shipment
        </CardTitle>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={syncMutation.isPending}
          onClick={() => syncMutation.mutate()}
        >
          {syncMutation.isPending ? "Syncing…" : "Sync from Shiprocket"}
        </Button>
      </div>
      <CardContent className="mt-4">
        {shipmentsQuery.isLoading ? (
          <p className="text-sm text-[var(--color-muted)]">Loading…</p>
        ) : shipmentsQuery.isError ? (
          <p className="text-sm text-rose-700">Could not load shipment.</p>
        ) : shipments.length === 0 ? (
          <p className="text-sm text-[var(--color-muted)]">No shipment created for this order yet.</p>
        ) : (
          <div className="space-y-4">
            {shipments.map((s) => (
              <OrderShipmentRow
                key={s.shipmentId}
                shipment={s}
                onEdit={() => {
                  setEditingShipmentId(s.shipmentId);
                  setAwbDraft(s.awbCode ?? "");
                  setCarrierDraft(s.carrier ?? "");
                  setFormError("");
                }}
              />
            ))}
          </div>
        )}

        <div className="mt-4 border-t border-[var(--color-line)] pt-4">
          <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">
            {editingShipmentId ? "Edit shipment" : "Create shipment"}
          </p>
          <div className="flex flex-wrap items-end gap-2.5">
            <Input
              value={awbDraft}
              onChange={(e) => setAwbDraft(e.target.value)}
              placeholder="AWB code"
              className="h-10 max-w-[12rem] rounded-lg text-[15px]"
            />
            <Input
              value={carrierDraft}
              onChange={(e) => setCarrierDraft(e.target.value)}
              placeholder="Carrier"
              className="h-10 max-w-[12rem] rounded-lg text-[15px]"
            />
            <Button
              type="button"
              size="sm"
              disabled={createMutation.isPending || updateMutation.isPending}
              onClick={() =>
                editingShipmentId
                  ? updateMutation.mutate(editingShipmentId)
                  : createMutation.mutate()
              }
            >
              {createMutation.isPending || updateMutation.isPending
                ? "Saving…"
                : editingShipmentId
                  ? "Save changes"
                  : "Create shipment"}
            </Button>
            {editingShipmentId && (
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  setEditingShipmentId(null);
                  setAwbDraft("");
                  setCarrierDraft("");
                  setFormError("");
                }}
              >
                Cancel
              </Button>
            )}
          </div>
          {formError && (
            <p className="mt-2 text-sm text-red-600" role="alert">
              {formError}
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
