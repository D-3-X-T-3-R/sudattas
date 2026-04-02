"use client";

import Link from "next/link";
import { Fragment, useCallback, useEffect, useMemo, useState } from "react";
import { signOut, useSession } from "next-auth/react";
import { SiteHeader } from "@/components/site-header";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { addressInputSchema } from "@/lib/validation-schemas";
import { formatInrFromPaise } from "@/lib/money";

type ShippingAddressRow = {
  shippingAddressId: string;
  userId?: string | null;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string | null;
  apartmentNoOrName?: string | null;
};

type AccountOrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  statusName: string;
};

type AccountProfileRow = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
};

type AccountOrderDetailRow = {
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

type AccountOrderDetailPayload = {
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

export default function ProfilePage() {
  const { data: session, status } = useSession();
  const { openLogin } = useStorefrontLogin();

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [orders, setOrders] = useState<AccountOrderRow[]>([]);
  const [accountProfile, setAccountProfile] = useState<AccountProfileRow | null>(null);
  const [expandedOrderId, setExpandedOrderId] = useState<string | null>(null);
  const [loadingOrderDetailId, setLoadingOrderDetailId] = useState<string | null>(null);
  const [orderDetailsById, setOrderDetailsById] = useState<
    Record<string, AccountOrderDetailPayload>
  >({});
  const [loadingData, setLoadingData] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const [form, setForm] = useState({
    country: "India",
    stateRegion: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });

  const authenticated = status === "authenticated";
  const loadingSession = status === "loading";
  const displayName =
    accountProfile?.fullName?.trim() ||
    session?.user?.name?.trim() ||
    "Member";
  const displayEmail =
    accountProfile?.email?.trim() ||
    session?.user?.email?.trim() ||
    "No email linked";

  const canSaveAddress = useMemo(() => {
    const parsed = addressInputSchema.safeParse({
      country: form.country.trim(),
      stateRegion: form.stateRegion.trim(),
      city: form.city.trim(),
      postalCode: form.postalCode.replace(/\D/g, "").slice(0, 6),
      road: form.road.trim(),
      apartmentNoOrName: form.apartmentNoOrName.trim() || null,
    });
    return parsed.success;
  }, [form]);

  const loadAccountData = useCallback(async () => {
    if (!authenticated) return;
    setLoadingData(true);
    setError(null);
    try {
      const [profileData, addrData, orderData] = await Promise.all([
        fetchApiEnvelope<AccountProfileRow>("/api/account/profile", {
          cache: "no-store",
        }),
        fetchApiEnvelope<ShippingAddressRow[]>("/api/account/addresses", {
          cache: "no-store",
        }),
        fetchApiEnvelope<AccountOrderRow[]>("/api/account/orders", {
          cache: "no-store",
        }),
      ]);
      setAccountProfile(profileData ?? null);
      setAddresses(addrData ?? []);
      setOrders(orderData ?? []);
      setExpandedOrderId(null);
      setOrderDetailsById({});
    } catch (e) {
      setError((e as Error).message || "Could not load account data.");
    } finally {
      setLoadingData(false);
    }
  }, [authenticated]);

  useEffect(() => {
    void loadAccountData();
  }, [loadAccountData]);

  async function addAddress() {
    if (!canSaveAddress || adding) return;
    setAdding(true);
    setError(null);
    try {
      const parsed = addressInputSchema.safeParse({
        country: form.country.trim(),
        stateRegion: form.stateRegion.trim(),
        city: form.city.trim(),
        postalCode: form.postalCode.replace(/\D/g, "").slice(0, 6),
        road: form.road.trim(),
        apartmentNoOrName: form.apartmentNoOrName.trim() || null,
      });
      if (!parsed.success) {
        setError(parsed.error.issues[0]?.message ?? "Invalid address.");
        return;
      }
      const input = parsed.data;
      await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ input }),
      });
      setForm({
        country: "India",
        stateRegion: "",
        city: "",
        postalCode: "",
        road: "",
        apartmentNoOrName: "",
      });
      await loadAccountData();
    } catch (e) {
      setError((e as Error).message || "Could not save address.");
    } finally {
      setAdding(false);
    }
  }

  async function deleteAddress(shippingAddressId: string) {
    setError(null);
    try {
      await fetchApiEnvelope<boolean>("/api/account/addresses", {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ shippingAddressId }),
      });
      await loadAccountData();
    } catch (e) {
      setError((e as Error).message || "Could not delete address.");
    }
  }

  async function toggleOrderDetails(orderId: string) {
    if (expandedOrderId === orderId) {
      setExpandedOrderId(null);
      return;
    }
    setExpandedOrderId(orderId);
    if (orderDetailsById[orderId]) return;
    setLoadingOrderDetailId(orderId);
    setError(null);
    try {
      const detail = await fetchApiEnvelope<AccountOrderDetailPayload>(
        `/api/account/orders/${encodeURIComponent(orderId)}`,
        { cache: "no-store" }
      );
      if (detail) {
        setOrderDetailsById((prev) => ({ ...prev, [orderId]: detail }));
      }
    } catch (e) {
      setError((e as Error).message || "Could not load order details.");
    } finally {
      setLoadingOrderDetailId((prev) => (prev === orderId ? null : prev));
    }
  }

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <SiteHeader />
      <main className="mx-auto max-w-5xl px-4 py-8">
        {loadingSession ? (
          <p className="text-sm text-[var(--color-muted)]">Loading profile...</p>
        ) : !authenticated ? (
          <section className="rounded-xl border border-[var(--color-line)] bg-white p-6">
            <h1 className="text-2xl font-semibold text-[var(--color-ink)]">Your Profile</h1>
            <p className="mt-2 text-sm text-[var(--color-muted)]">Sign in to see your account, saved addresses, and order history.</p>
            <button
              type="button"
              onClick={() => openLogin("/profile")}
              className="mt-4 rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white"
            >
              Sign in
            </button>
          </section>
        ) : (
          <section className="space-y-6">
            <div className="rounded-xl border border-[var(--color-line)] bg-white p-6">
              <h1 className="text-2xl font-semibold text-[var(--color-ink)]">Your Profile</h1>
              <p className="mt-1 text-sm text-[var(--color-muted)]">{displayName}</p>
              <p className="text-sm text-[var(--color-muted)]">{displayEmail}</p>
              {accountProfile?.phone?.trim() && (
                <p className="text-sm text-[var(--color-muted)]">{accountProfile.phone}</p>
              )}
              {accountProfile?.address?.trim() && (
                <p className="text-sm text-[var(--color-muted)]">{accountProfile.address}</p>
              )}
              <div className="mt-4 flex gap-3">
                <Link href="/bag" className="text-sm text-[var(--color-accent-gold)] underline">
                  View Bag
                </Link>
                <Link href="/wishlist" className="text-sm text-[var(--color-accent-gold)] underline">
                  View Wishlist
                </Link>
                <button
                  type="button"
                  onClick={() => signOut({ callbackUrl: "/" })}
                  className="text-sm text-[var(--color-accent-gold)] underline"
                >
                  Sign out
                </button>
              </div>
            </div>

            {error && (
              <p className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
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
                    <li
                      key={a.shippingAddressId}
                      className="flex items-center justify-between rounded-lg border border-[var(--color-line)] px-3 py-2"
                    >
                      <span className="text-sm text-[var(--color-ink)]">{formatAddress(a)}</span>
                      <button
                        type="button"
                        onClick={() => void deleteAddress(a.shippingAddressId)}
                        className="text-xs font-semibold uppercase tracking-[0.12em] text-red-600"
                      >
                        Remove
                      </button>
                    </li>
                  ))}
                </ul>
              )}

              <div className="mt-5 grid gap-2 sm:grid-cols-2">
                <input
                  value={form.road}
                  onChange={(e) => setForm((p) => ({ ...p, road: e.target.value }))}
                  placeholder="Road / street"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
                <input
                  value={form.apartmentNoOrName}
                  onChange={(e) => setForm((p) => ({ ...p, apartmentNoOrName: e.target.value }))}
                  placeholder="Apartment / house (optional)"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
                <input
                  value={form.city}
                  onChange={(e) => setForm((p) => ({ ...p, city: e.target.value }))}
                  placeholder="City"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
                <input
                  value={form.stateRegion}
                  onChange={(e) => setForm((p) => ({ ...p, stateRegion: e.target.value }))}
                  placeholder="State / region"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
                <input
                  value={form.country}
                  onChange={(e) => setForm((p) => ({ ...p, country: e.target.value }))}
                  placeholder="Country"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
                <input
                  value={form.postalCode}
                  onChange={(e) =>
                    setForm((p) => ({
                      ...p,
                      postalCode: e.target.value.replace(/\D/g, "").slice(0, 6),
                    }))
                  }
                  placeholder="Pincode"
                  className="h-10 rounded-md border border-[var(--color-line)] px-3 text-sm"
                />
              </div>

              <button
                type="button"
                onClick={() => void addAddress()}
                disabled={!canSaveAddress || adding}
                className="mt-3 rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white disabled:opacity-50"
              >
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
                          <tr
                            key={o.orderId}
                            className="cursor-pointer border-b border-[var(--color-line)] hover:bg-[var(--color-line)]/10"
                            onClick={() => void toggleOrderDetails(o.orderId)}
                          >
                            <td className="py-2 pr-3 font-mono text-[var(--color-ink)]">{o.orderId}</td>
                            <td className="py-2 pr-3 text-[var(--color-ink)]">{formatOrderDate(o.orderDate)}</td>
                          <td className="py-2 pr-3 text-[var(--color-ink)]">
                            {o.totalAmountFormatted || formatInrFromPaise(o.totalAmountPaise)}
                          </td>
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
                                      <span>
                                        <strong>Status:</strong> {orderDetailsById[o.orderId].statusName}
                                      </span>
                                      <span>
                                        <strong>Payment:</strong> {orderDetailsById[o.orderId].paymentState}
                                      </span>
                                      <span>
                                        <strong>Fulfillment:</strong> {orderDetailsById[o.orderId].fulfillmentState}
                                      </span>
                                    </div>
                                    <div>
                                      <p className="font-semibold text-[var(--color-ink)]">Items</p>
                                      {(orderDetailsById[o.orderId].order.orderDetails ?? []).length === 0 ? (
                                        <p className="text-[var(--color-muted)]">No line items available.</p>
                                      ) : (
                                        <ul className="mt-1 space-y-1">
                                          {(orderDetailsById[o.orderId].order.orderDetails ?? []).map((d) => (
                                            <li key={d.orderDetailId} className="text-[var(--color-ink)]">
                                              {(d.productDetails?.[0]?.name || `Variant ${d.variantId}`)} - Qty {d.quantity} - {d.priceFormatted || d.pricePaise}
                                            </li>
                                          ))}
                                        </ul>
                                      )}
                                    </div>
                                    <div className="grid gap-3 sm:grid-cols-2">
                                      <div>
                                        <p className="font-semibold text-[var(--color-ink)]">Payments</p>
                                        {orderDetailsById[o.orderId].paymentIntents.length === 0 ? (
                                          <p className="text-[var(--color-muted)]">No payment records yet.</p>
                                        ) : (
                                          <ul className="mt-1 space-y-1">
                                            {orderDetailsById[o.orderId].paymentIntents.map((p) => (
                                              <li key={p.intentId} className="text-[var(--color-ink)]">
                                                {p.status} ({formatInrFromPaise(p.amountPaise)})
                                              </li>
                                            ))}
                                          </ul>
                                        )}
                                      </div>
                                      <div>
                                        <p className="font-semibold text-[var(--color-ink)]">Shipments</p>
                                        {orderDetailsById[o.orderId].shipments.length === 0 ? (
                                          <p className="text-[var(--color-muted)]">Not shipped yet.</p>
                                        ) : (
                                          <ul className="mt-1 space-y-1">
                                            {orderDetailsById[o.orderId].shipments.map((s) => (
                                              <li key={s.shipmentId} className="text-[var(--color-ink)]">
                                                {s.status}
                                                {s.carrier ? ` - ${s.carrier}` : ""}
                                                {s.awbCode ? ` (${s.awbCode})` : ""}
                                              </li>
                                            ))}
                                          </ul>
                                        )}
                                      </div>
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
        )}
      </main>
    </div>
  );
}
