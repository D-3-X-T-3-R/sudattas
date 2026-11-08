"use client";

import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { fetchAllCustomersList, type CustomerListRow } from "@/lib/admin-queries";
import {
  createShippingAddressAdmin,
  fetchAllShippingAddressesAdmin,
  type AdminShippingAddressRow,
} from "@/lib/admin-order-create";
import {
  OrderCreateAddressForm,
  EMPTY_ADDRESS_DRAFT,
} from "@/domains/admin/orders/components/order-create-address-form";

interface OrderCreateCustomerCardProps {
  selectedCustomer: CustomerListRow | null;
  setSelectedCustomer: (customer: CustomerListRow | null) => void;
  selectedAddressId: string;
  setSelectedAddressId: (id: string) => void;
}

export function OrderCreateCustomerCard({
  selectedCustomer,
  setSelectedCustomer,
  selectedAddressId,
  setSelectedAddressId,
}: OrderCreateCustomerCardProps) {
  const queryClient = useQueryClient();
  const [customerSearch, setCustomerSearch] = useState("");
  const [showNewAddress, setShowNewAddress] = useState(false);
  const [addressDraft, setAddressDraft] = useState(EMPTY_ADDRESS_DRAFT);
  const [addressError, setAddressError] = useState("");

  const customersQuery = useQuery({
    queryKey: ["admin", "order-create-customers"],
    queryFn: fetchAllCustomersList,
  });

  const addressesQuery = useQuery({
    queryKey: ["admin", "order-create-addresses"],
    queryFn: fetchAllShippingAddressesAdmin,
  });

  const matchingCustomers = useMemo(() => {
    const q = customerSearch.trim().toLowerCase();
    if (!q) return [];
    return (customersQuery.data ?? [])
      .filter(
        (c) =>
          c.email.toLowerCase().includes(q) ||
          (c.fullName ?? "").toLowerCase().includes(q) ||
          c.username.toLowerCase().includes(q)
      )
      .slice(0, 20);
  }, [customerSearch, customersQuery.data]);

  const customerAddresses = useMemo(
    () => (addressesQuery.data ?? []).filter((a) => a.userId === selectedCustomer?.userId),
    [addressesQuery.data, selectedCustomer]
  );

  const createAddressMutation = useMutation({
    mutationFn: () => {
      if (!selectedCustomer) throw new Error("Select a customer first.");
      if (!addressDraft.stateRegion.trim() || !addressDraft.city.trim() || !addressDraft.postalCode.trim()) {
        throw new Error("State, city, and postal code are required.");
      }
      return createShippingAddressAdmin({ userId: selectedCustomer.userId, ...addressDraft });
    },
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ["admin", "order-create-addresses"] });
      if (created) setSelectedAddressId(created.shippingAddressId);
      setShowNewAddress(false);
      setAddressDraft(EMPTY_ADDRESS_DRAFT);
      setAddressError("");
    },
    onError: (err: Error) => setAddressError(err.message || "Failed to add address."),
  });

  return (
    <AdminTableCard title="Customer" icon={<User className="h-4 w-4 text-[var(--color-green)]" />}>
      {selectedCustomer ? (
        <div className="flex items-center justify-between gap-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-3">
          <div>
            <p className="text-[15px] font-medium text-[var(--color-ink)]">
              {selectedCustomer.fullName || selectedCustomer.username}
            </p>
            <p className="text-sm text-[var(--color-muted)]">{selectedCustomer.email}</p>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              setSelectedCustomer(null);
              setSelectedAddressId("");
            }}
          >
            Change
          </Button>
        </div>
      ) : (
        <>
          <Input
            value={customerSearch}
            onChange={(e) => setCustomerSearch(e.target.value)}
            placeholder="Search by name, email, or username…"
            className="rounded-lg text-[15px]"
          />
          {customerSearch.trim() && (
            <div className="mt-2 max-h-48 overflow-y-auto rounded-lg border border-[var(--color-line)]">
              {customersQuery.isLoading ? (
                <p className="p-2.5 text-sm text-[var(--color-muted)]">Loading customers…</p>
              ) : matchingCustomers.length === 0 ? (
                <p className="p-2.5 text-sm text-[var(--color-muted)]">No matching customers.</p>
              ) : (
                <ul>
                  {matchingCustomers.map((c) => (
                    <li key={c.userId}>
                      <button
                        type="button"
                        onClick={() => setSelectedCustomer(c)}
                        className="w-full px-2.5 py-2 text-left text-[15px] hover:bg-[var(--color-surface-soft)]"
                      >
                        <span className="font-medium text-[var(--color-ink)]">
                          {c.fullName || c.username}
                        </span>{" "}
                        <span className="text-[var(--color-muted)]">({c.email})</span>
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </>
      )}

      {selectedCustomer && (
        <div className="mt-4 border-t border-[var(--color-line)] pt-4">
          <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">Shipping address</p>
          {addressesQuery.isLoading ? (
            <p className="text-sm text-[var(--color-muted)]">Loading addresses…</p>
          ) : (
            <div className="space-y-2">
              {customerAddresses.map((a) => (
                <label
                  key={a.shippingAddressId}
                  className="flex cursor-pointer items-start gap-2.5 rounded-lg border border-[var(--color-line)] p-2.5 text-sm has-[:checked]:border-[var(--color-green)]"
                >
                  <input
                    type="radio"
                    name="order-create-address"
                    checked={selectedAddressId === a.shippingAddressId}
                    onChange={() => setSelectedAddressId(a.shippingAddressId)}
                    className="mt-1"
                  />
                  <span className="text-[var(--color-ink)]">
                    {a.recipientName ? `${a.recipientName}, ` : ""}
                    {[a.road, a.apartmentNoOrName, a.city, a.stateRegion, a.postalCode]
                      .filter(Boolean)
                      .join(", ")}
                    {a.phoneNumber ? ` · ${a.phoneNumber}` : ""}
                  </span>
                </label>
              ))}
              {customerAddresses.length === 0 && !showNewAddress ? (
                <p className="text-sm text-[var(--color-muted)]">No saved addresses for this customer.</p>
              ) : null}
            </div>
          )}

          <Button
            type="button"
            variant="outline"
            size="sm"
            className="mt-2"
            onClick={() => setShowNewAddress((s) => !s)}
          >
            {showNewAddress ? "Cancel" : "+ Add new address"}
          </Button>

          {showNewAddress && (
            <OrderCreateAddressForm
              draft={addressDraft}
              setDraft={setAddressDraft}
              isPending={createAddressMutation.isPending}
              error={addressError}
              onSave={() => createAddressMutation.mutate()}
            />
          )}
        </div>
      )}
    </AdminTableCard>
  );
}

export type { AdminShippingAddressRow };
