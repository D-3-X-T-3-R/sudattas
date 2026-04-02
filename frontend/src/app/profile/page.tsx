"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { signOut, useSession } from "next-auth/react";
import { SiteHeader } from "@/components/site-header";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { addressInputSchema } from "@/lib/validation-schemas";
import { toRouteFailureUi } from "@/lib/route-state";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { ProfileAuthenticatedContent } from "@/domains/profile/components/profile-authenticated-content";
import type {
  AccountOrderDetailPayload,
  AccountOrderRow,
  AccountProfileRow,
  ShippingAddressRow,
} from "@/domains/profile/components/profile-authenticated-content";
import type { AddressFormState } from "@/domains/profile/types";

export default function ProfilePage() {
  const { data: session, status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const { announce } = useLiveAnnouncer();

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [orders, setOrders] = useState<AccountOrderRow[]>([]);
  const [accountProfile, setAccountProfile] = useState<AccountProfileRow | null>(null);
  const [expandedOrderId, setExpandedOrderId] = useState<string | null>(null);
  const [loadingOrderDetailId, setLoadingOrderDetailId] = useState<string | null>(null);
  const [orderDetailsById, setOrderDetailsById] = useState<Record<string, AccountOrderDetailPayload>>({});
  const [loadingData, setLoadingData] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const [form, setForm] = useState<AddressFormState>({
    country: "India",
    stateRegion: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });

  const authenticated = status === "authenticated";
  const loadingSession = status === "loading";
  const displayName = accountProfile?.fullName?.trim() || session?.user?.name?.trim() || "Member";
  const displayEmail = accountProfile?.email?.trim() || session?.user?.email?.trim() || "No email linked";

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
        fetchApiEnvelope<AccountProfileRow>("/api/account/profile", { cache: "no-store" }),
        fetchApiEnvelope<ShippingAddressRow[]>("/api/account/addresses", { cache: "no-store" }),
        fetchApiEnvelope<AccountOrderRow[]>("/api/account/orders", { cache: "no-store" }),
      ]);
      setAccountProfile(profileData ?? null);
      setAddresses(addrData ?? []);
      setOrders(orderData ?? []);
      setExpandedOrderId(null);
      setOrderDetailsById({});
    } catch (e) {
      setError(toRouteFailureUi("account", e).message);
    } finally {
      setLoadingData(false);
    }
  }, [authenticated]);

  useEffect(() => {
    void loadAccountData();
  }, [loadAccountData]);

  const addAddress = async () => {
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
      await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ input: parsed.data }),
      });
      setForm({ country: "India", stateRegion: "", city: "", postalCode: "", road: "", apartmentNoOrName: "" });
      await loadAccountData();
      announce("Address saved successfully.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setError(ui.message);
      announce(ui.message, "assertive");
    } finally {
      setAdding(false);
    }
  };

  const deleteAddress = async (shippingAddressId: string) => {
    setError(null);
    try {
      await fetchApiEnvelope<boolean>("/api/account/addresses", {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ shippingAddressId }),
      });
      await loadAccountData();
      announce("Address removed.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setError(ui.message);
      announce(ui.message, "assertive");
    }
  };

  const toggleOrderDetails = async (orderId: string) => {
    if (expandedOrderId === orderId) {
      setExpandedOrderId(null);
      return;
    }
    setExpandedOrderId(orderId);
    if (orderDetailsById[orderId]) return;
    setLoadingOrderDetailId(orderId);
    setError(null);
    try {
      const detail = await fetchApiEnvelope<AccountOrderDetailPayload>(`/api/account/orders/${encodeURIComponent(orderId)}`, { cache: "no-store" });
      if (detail) setOrderDetailsById((prev) => ({ ...prev, [orderId]: detail }));
    } catch (e) {
      setError(toRouteFailureUi("account", e).message);
    } finally {
      setLoadingOrderDetailId((prev) => (prev === orderId ? null : prev));
    }
  };

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
            <button type="button" onClick={() => openLogin("/profile")} className="mt-4 rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white">Sign in</button>
          </section>
        ) : (
          <ProfileAuthenticatedContent
            displayName={displayName}
            displayEmail={displayEmail}
            accountProfile={accountProfile}
            error={error}
            loadingData={loadingData}
            addresses={addresses}
            orders={orders}
            expandedOrderId={expandedOrderId}
            loadingOrderDetailId={loadingOrderDetailId}
            orderDetailsById={orderDetailsById}
            form={form}
            setForm={setForm}
            canSaveAddress={canSaveAddress}
            adding={adding}
            addAddress={addAddress}
            deleteAddress={deleteAddress}
            toggleOrderDetails={toggleOrderDetails}
            onSignOut={() => void signOut({ callbackUrl: "/" })}
          />
        )}
      </main>
    </div>
  );
}
