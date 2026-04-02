"use client";

import Link from "next/link";
import { Fragment, type Dispatch, type SetStateAction } from "react";
import type { AddressFormState } from "@/domains/profile/types";
import { formatInrFromPaise } from "@/lib/money";

export type ShippingAddressRow = {
  shippingAddressId: string;
  userId?: string | null;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string | null;
  apartmentNoOrName?: string | null;
};

export type AccountOrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  statusName: string;
};

export type AccountProfileRow = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
};

export type AccountOrderDetailRow = {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  pricePaise: string;
  priceFormatted: string;
  productDetails?: Array<{
    productId?: string;
    name?: string;
    formatted?: string;
  }>;
};

export type AccountOrderDetailPayload = {
  order: AccountOrderRow & {
    orderDetails?: AccountOrderDetailRow[];
  };
  statusName: string;
  paymentIntents: Array<{
    intentId: string;
    amountPaise: string;
    currency?: string | null;
    status: string;
    razorpayPaymentId?: string | null;
    createdAt: string;
  }>;
  shipments: Array<{
    shipmentId: string;
    status: string;
    carrier?: string | null;
    awbCode?: string | null;
    createdAt: string;
    deliveredAt?: string | null;
  }>;
  events: Array<{
    eventId: string;
    eventType: string;
    fromStatus: string;
    toStatus: string;
    actorType: string;
    message: string;
    createdAt: string;
  }>;
  fulfillmentState: string;
  paymentState: string;
};

function formatAddress(a: ShippingAddressRow): string {
  const parts = [
    [a.apartmentNoOrName, a.road].filter(Boolean).join(", "),
    a.city,
    a.stateRegion,
    a.postalCode,
    a.country,
  ].filter((v) => v && String(v).trim());
  return parts.join(", ");
}

function formatOrderDate(raw: string): string {
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", {
    day: "2-digit",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

type ProfileAuthenticatedContentProps = {
  displayName: string;
  displayEmail: string;
  accountProfile: AccountProfileRow | null;
  error: string | null;
  loadingData: boolean;
  addresses: ShippingAddressRow[];
  orders: AccountOrderRow[];
  expandedOrderId: string | null;
  loadingOrderDetailId: string | null;
  orderDetailsById: Record<string, AccountOrderDetailPayload>;
  form: AddressFormState;
  setForm: Dispatch<SetStateAction<AddressFormState>>;
  canSaveAddress: boolean;
  adding: boolean;
  addAddress: () => Promise<void>;
  deleteAddress: (shippingAddressId: string) => Promise<void>;
  toggleOrderDetails: (orderId: string) => Promise<void>;
  onSignOut: () => void;
};

export function ProfileAuthenticatedContent({
  displayName,
  displayEmail,
  accountProfile,
  error,
  loadingData,
  addresses,
  orders,
  expandedOrderId,
  loadingOrderDetailId,
  orderDetailsById,
  form,
  setForm,
  canSaveAddress,
  adding,
  addAddress,
  deleteAddress,
  toggleOrderDetails,
  onSignOut,
}: ProfileAuthenticatedContentProps) {
  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-[var(--color-line)] bg-white p-6">
        <h1 className="text-2xl font-semibold text-[var(--color-ink)]">Your Profile</h1>
        <p className="mt-1 text-sm text-[var(--color-muted)]">{displayName}</p>
        <p className="text-sm text-[var(--color-muted)]">{displayEmail}</p>
        {accountProfile?.phone?.trim() && <p className="text-sm text-[var(--color-muted)]">{accountProfile.phone}</p>}
        {accountProfile?.address?.trim() && <p className="text-sm text-[var(--color-muted)]">{accountProfile.address}</p>}
        <div className="mt-4 flex gap-3">
          <Link href="/bag" className="text-sm text-[var(--color-accent-gold)] underline">View Bag</Link>
          <Link href="/wishlist" className="text-sm text-[var(--color-accent-gold)] underline">View Wishlist</Link>
          <button type="button" onClick={onSignOut} className="text-sm text-[var(--color-accent-gold)] underline">Sign out</button>
        </div>
      </div>

      {error && (
        <p id="profile-form-error" role="alert" className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </p>
      )}

      <div className="rounded-xl border border-[var(--color-line)] bg-white p-6">
        <h2 className="text-xl font-semibold text-[var(--color-ink)]">Saved Addresses</h2>
        {loadingData ? (
          <p className="mt-2 text-sm text-[var(--color-muted)]">Loading addresses...</p>
        ) : addresses.length === 0 ? (
          <p className="mt-2 text-sm text-[var(--color-muted)]">No addresses saved yet.</p>
        ) : (
          <ul className="mt-3 space-y-2">
            {addresses.map((a) => (
              <li key={a.shippingAddressId} className="flex items-center justify-between rounded-lg border border-[var(--color-line)] px-3 py-2">
                <span className="text-sm text-[var(--color-ink)]">{formatAddress(a)}</span>
                <button type="button" onClick={() => void deleteAddress(a.shippingAddressId)} className="text-xs font-semibold uppercase tracking-[0.12em] text-red-600">
                  Remove
                </button>
              </li>
            ))}
          </ul>
        )}

        <div className="mt-5 grid gap-2 sm:grid-cols-2">
          <div>
            <label htmlFor="profile-road" className="mb-1 block text-xs text-[var(--color-muted)]">Road / street</label>
            <input id="profile-road" value={form.road} onChange={(e) => setForm((p) => ({ ...p, road: e.target.value }))} aria-invalid={!!error} aria-describedby={error ? "profile-form-error" : undefined} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
          <div>
            <label htmlFor="profile-apartment" className="mb-1 block text-xs text-[var(--color-muted)]">Apartment / house (optional)</label>
            <input id="profile-apartment" value={form.apartmentNoOrName} onChange={(e) => setForm((p) => ({ ...p, apartmentNoOrName: e.target.value }))} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
          <div>
            <label htmlFor="profile-city" className="mb-1 block text-xs text-[var(--color-muted)]">City</label>
            <input id="profile-city" value={form.city} onChange={(e) => setForm((p) => ({ ...p, city: e.target.value }))} aria-invalid={!!error} aria-describedby={error ? "profile-form-error" : undefined} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
          <div>
            <label htmlFor="profile-state" className="mb-1 block text-xs text-[var(--color-muted)]">State / region</label>
            <input id="profile-state" value={form.stateRegion} onChange={(e) => setForm((p) => ({ ...p, stateRegion: e.target.value }))} aria-invalid={!!error} aria-describedby={error ? "profile-form-error" : undefined} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
          <div>
            <label htmlFor="profile-country" className="mb-1 block text-xs text-[var(--color-muted)]">Country</label>
            <input id="profile-country" value={form.country} onChange={(e) => setForm((p) => ({ ...p, country: e.target.value }))} aria-invalid={!!error} aria-describedby={error ? "profile-form-error" : undefined} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
          <div>
            <label htmlFor="profile-pincode" className="mb-1 block text-xs text-[var(--color-muted)]">Pincode</label>
            <input id="profile-pincode" value={form.postalCode} onChange={(e) => setForm((p) => ({ ...p, postalCode: e.target.value.replace(/\D/g, "").slice(0, 6) }))} inputMode="numeric" aria-invalid={!!error} aria-describedby={error ? "profile-form-error" : undefined} className="h-10 w-full rounded-md border border-[var(--color-line)] px-3 text-sm" />
          </div>
        </div>

        <button type="button" onClick={() => void addAddress()} disabled={!canSaveAddress || adding} className="mt-3 rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white disabled:opacity-50">
          {adding ? "Saving..." : "Save Address"}
        </button>
      </div>

      <div className="rounded-xl border border-[var(--color-line)] bg-white p-6">
        <h2 className="text-xl font-semibold text-[var(--color-ink)]">Orders</h2>
        {loadingData ? (
          <p className="mt-2 text-sm text-[var(--color-muted)]">Loading orders...</p>
        ) : orders.length === 0 ? (
          <p className="mt-2 text-sm text-[var(--color-muted)]">No orders yet.</p>
        ) : (
          <div className="mt-3 overflow-x-auto">
            <table className="w-full min-w-[520px] border-collapse text-sm">
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                  <th className="pb-2 pr-3 font-medium">Order ID</th>
                  <th className="pb-2 pr-3 font-medium">Date</th>
                  <th className="pb-2 pr-3 font-medium">Amount</th>
                  <th className="pb-2 font-medium">Status</th>
                </tr>
              </thead>
              <tbody>
                {orders.map((o) => (
                  <Fragment key={o.orderId}>
                    <tr className="cursor-pointer border-b border-[var(--color-line)] hover:bg-[var(--color-line)]/10" onClick={() => void toggleOrderDetails(o.orderId)}>
                      <td className="py-2 pr-3 font-mono text-[var(--color-ink)]">{o.orderId}</td>
                      <td className="py-2 pr-3 text-[var(--color-ink)]">{formatOrderDate(o.orderDate)}</td>
                      <td className="py-2 pr-3 text-[var(--color-ink)]">{o.totalAmountFormatted || formatInrFromPaise(o.totalAmountPaise)}</td>
                      <td className="py-2 text-[var(--color-ink)]">{o.statusName}</td>
                    </tr>
                    {expandedOrderId === o.orderId && (
                      <tr className="border-b border-[var(--color-line)] last:border-0">
                        <td colSpan={4} className="px-2 py-3">
                          {loadingOrderDetailId === o.orderId ? (
                            <p className="text-xs text-[var(--color-muted)]">Loading order details...</p>
                          ) : orderDetailsById[o.orderId] ? (
                            <div className="space-y-3 rounded-lg bg-[var(--color-line)]/10 p-3 text-xs">
                              <div className="flex flex-wrap gap-4 text-[var(--color-ink)]">
                                <span><strong>Status:</strong> {orderDetailsById[o.orderId].statusName}</span>
                                <span><strong>Payment:</strong> {orderDetailsById[o.orderId].paymentState}</span>
                                <span><strong>Fulfillment:</strong> {orderDetailsById[o.orderId].fulfillmentState}</span>
                              </div>
                            </div>
                          ) : (
                            <p className="text-xs text-[var(--color-muted)]">Could not load details.</p>
                          )}
                        </td>
                      </tr>
                    )}
                  </Fragment>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
