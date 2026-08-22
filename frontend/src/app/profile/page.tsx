"use client";

import { useCallback, useEffect, useMemo, useRef, useState, type Dispatch, type SetStateAction } from "react";
import { signOut, useSession } from "next-auth/react";
import { RouteFailureBanner } from "@/components/route-failure-banner";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { addressInputSchema, profileUpdateSchema } from "@/lib/validation-schemas";
import { toRouteFailureUi, type RouteFailureUi } from "@/lib/route-state";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { PageShell } from "@/components/ui/page-shell";
import { ProfileAuthenticatedContent } from "@/domains/profile/components/profile-authenticated-content";
import type {
  AccountOrderDetailPayload,
  AccountOrderRow,
  AccountProfileRow,
  ShippingAddressRow,
} from "@/domains/profile/components/profile-authenticated-content";
import type { AddressFormState, ProfileFormState } from "@/domains/profile/types";

type AuthenticatedSectionProps = {
  routeFailure: RouteFailureUi | null;
  loadAccountData: () => Promise<void>;
  openLogin: (returnTo?: string) => void;
  displayName: string;
  displayEmail: string;
  loginMethodLabel: string;
  accountProfile: AccountProfileRow | null;
  error: string | null;
  loadingData: boolean;
  addresses: ShippingAddressRow[];
  orders: AccountOrderRow[];
  orderDetailsById: Record<string, AccountOrderDetailPayload>;
  form: AddressFormState;
  setForm: Dispatch<SetStateAction<AddressFormState>>;
  canSaveAddress: boolean;
  adding: boolean;
  addAddress: () => Promise<void>;
  updateAddress: (shippingAddressId: string) => Promise<void>;
  deleteAddress: (shippingAddressId: string) => Promise<void>;
  setDefaultAddress: (shippingAddressId: string) => Promise<void>;
  profileForm: ProfileFormState;
  setProfileForm: Dispatch<SetStateAction<ProfileFormState>>;
  canSaveProfile: boolean;
  savingProfile: boolean;
  updateProfile: () => Promise<void>;
  ensureOrderDetailLoaded: (orderId: string) => Promise<void>;
  refreshOrderDetail: (orderId: string) => Promise<void>;
  cancelOrder: (orderId: string) => Promise<void>;
  cancelOrderItems: (orderId: string, orderDetailIds: string[]) => Promise<void>;
  requestReturn: (orderId: string, orderDetailIds: string[], reason: string) => Promise<void>;
};

type UseAccountDataLoaderArgs = {
  authenticated: boolean;
  setLoadingData: Dispatch<SetStateAction<boolean>>;
  setRouteFailure: Dispatch<SetStateAction<RouteFailureUi | null>>;
  setAccountProfile: Dispatch<SetStateAction<AccountProfileRow | null>>;
  setAddresses: Dispatch<SetStateAction<ShippingAddressRow[]>>;
  setOrders: Dispatch<SetStateAction<AccountOrderRow[]>>;
  setOrderDetailsById: Dispatch<SetStateAction<Record<string, AccountOrderDetailPayload>>>;
};

function useAccountDataLoader({
  authenticated,
  setLoadingData,
  setRouteFailure,
  setAccountProfile,
  setAddresses,
  setOrders,
  setOrderDetailsById,
}: UseAccountDataLoaderArgs) {
  const loadAccountData = useCallback(async () => {
    if (!authenticated) return;
    setLoadingData(true);
    setRouteFailure(null);
    try {
      const [profileData, addrData, orderData] = await Promise.all([
        fetchApiEnvelope<AccountProfileRow>("/api/account/profile", { cache: "no-store" }),
        fetchApiEnvelope<ShippingAddressRow[]>("/api/account/addresses", { cache: "no-store" }),
        fetchApiEnvelope<AccountOrderRow[]>("/api/account/orders", { cache: "no-store" }),
      ]);
      setAccountProfile(profileData ?? null);
      setAddresses(addrData ?? []);
      setOrders(orderData ?? []);
      setOrderDetailsById({});
    } catch (e) {
      setRouteFailure(toRouteFailureUi("account", e));
    } finally {
      setLoadingData(false);
    }
  }, [
    authenticated,
    setAccountProfile,
    setAddresses,
    setLoadingData,
    setOrderDetailsById,
    setOrders,
    setRouteFailure,
  ]);

  useEffect(() => {
    void loadAccountData();
  }, [loadAccountData]);

  return loadAccountData;
}

function UnauthenticatedProfile({ openLogin }: { openLogin: (returnTo?: string) => void }) {
  return (
    <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-8 shadow-[var(--shadow-soft)] sm:p-10">
      <span className="inline-block rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
        Account
      </span>
      <h1 className="mt-4 font-display text-3xl font-semibold text-[var(--color-ink)] sm:text-4xl">Your Profile</h1>
      <p className="mt-3 max-w-md text-sm leading-relaxed text-[var(--color-muted)]">
        Sign in to see your account, saved addresses, and order history in the premium Sudatta&apos;s dashboard.
      </p>
      <button
        type="button"
        onClick={() => openLogin("/profile")}
        className="mt-6 inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
      >
        Sign in
      </button>
    </section>
  );
}

function AuthenticatedProfileSection(props: AuthenticatedSectionProps) {
  const {
    routeFailure,
    loadAccountData,
    openLogin,
    displayName,
    displayEmail,
    loginMethodLabel,
    accountProfile,
    error,
    loadingData,
    addresses,
    orders,
    orderDetailsById,
    form,
    setForm,
    canSaveAddress,
    adding,
  addAddress,
  updateAddress,
  deleteAddress,
  setDefaultAddress,
    profileForm,
    setProfileForm,
    canSaveProfile,
    savingProfile,
    updateProfile,
    ensureOrderDetailLoaded,
    refreshOrderDetail,
    cancelOrder,
    cancelOrderItems,
    requestReturn,
  } = props;

  return (
    <>
      {routeFailure && (
        <RouteFailureBanner
          failure={routeFailure}
          onRetry={routeFailure.kind === "retryable" ? () => void loadAccountData() : undefined}
          onSignIn={routeFailure.kind === "unauthorized" || routeFailure.kind === "stale" ? () => openLogin("/profile") : undefined}
          className="mb-4"
        />
      )}
      <ProfileAuthenticatedContent
        displayName={displayName}
        displayEmail={displayEmail}
        loginMethodLabel={loginMethodLabel}
        accountProfile={accountProfile}
        error={error}
        loadingData={loadingData}
        addresses={addresses}
        orders={orders}
        orderDetailsById={orderDetailsById}
        form={form}
        setForm={setForm}
        canSaveAddress={canSaveAddress}
        adding={adding}
        addAddress={addAddress}
        updateAddress={updateAddress}
        deleteAddress={deleteAddress}
        setDefaultAddress={setDefaultAddress}
        profileForm={profileForm}
        setProfileForm={setProfileForm}
        canSaveProfile={canSaveProfile}
        savingProfile={savingProfile}
        updateProfile={updateProfile}
        ensureOrderDetailLoaded={ensureOrderDetailLoaded}
        refreshOrderDetail={refreshOrderDetail}
        cancelOrder={cancelOrder}
        cancelOrderItems={cancelOrderItems}
        requestReturn={requestReturn}
        onSignOut={() => void signOut({ callbackUrl: "/" })}
      />
    </>
  );
}

// eslint-disable-next-line max-lines-per-function
export default function ProfilePage() {
  const { data: session, status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const { announce } = useLiveAnnouncer();

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [orders, setOrders] = useState<AccountOrderRow[]>([]);
  const [accountProfile, setAccountProfile] = useState<AccountProfileRow | null>(null);
  const orderDetailsRef = useRef<Record<string, AccountOrderDetailPayload>>({});
  const [orderDetailsById, setOrderDetailsById] = useState<Record<string, AccountOrderDetailPayload>>({});
  const [loadingData, setLoadingData] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [routeFailure, setRouteFailure] = useState<RouteFailureUi | null>(null);
  const [adding, setAdding] = useState(false);
  const [form, setForm] = useState<AddressFormState>({
    recipientName: "",
    phoneNumber: "",
    country: "India",
    stateRegion: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });
  const [profileForm, setProfileForm] = useState<ProfileFormState>({
    firstName: "",
    lastName: "",
    gender: "",
    dateOfBirth: "",
    phoneNumber: "",
  });
  const [savingProfile, setSavingProfile] = useState(false);
  const authenticated = status === "authenticated", loadingSession = status === "loading";
  const displayName = accountProfile?.fullName?.trim() || session?.user?.name?.trim() || "Member";
  const displayEmail = accountProfile?.email?.trim() || session?.user?.email?.trim() || "No email linked";
  const loginMethodLabel = useMemo(() => (session?.idToken ? "Google" : "Email"), [session?.idToken]);

  const canSaveAddress = useMemo(() => {
    const parsed = addressInputSchema.safeParse({
      recipientName: form.recipientName.trim() || null,
      phoneNumber: form.phoneNumber.trim() || null,
      country: form.country.trim(),
      stateRegion: form.stateRegion.trim(),
      city: form.city.trim(),
      postalCode: form.postalCode.replace(/\D/g, "").slice(0, 6),
      road: form.road.trim(),
      apartmentNoOrName: form.apartmentNoOrName.trim() || null,
    });
    return parsed.success;
  }, [form]);

  const profileFormSeededRef = useRef(false);
  useEffect(() => {
    if (!accountProfile || profileFormSeededRef.current) return;
    profileFormSeededRef.current = true;
    setProfileForm({
      firstName: accountProfile.firstName?.trim() ?? "",
      lastName: accountProfile.lastName?.trim() ?? "",
      gender: accountProfile.gender?.trim() ?? "",
      dateOfBirth: accountProfile.dateOfBirth?.trim() ?? "",
      phoneNumber: accountProfile.phone?.trim() ?? "",
    });
  }, [accountProfile]);

  const canSaveProfile = useMemo(() => {
    const parsed = profileUpdateSchema.safeParse({
      firstName: profileForm.firstName,
      lastName: profileForm.lastName || undefined,
      gender: profileForm.gender || undefined,
      dateOfBirth: profileForm.dateOfBirth || undefined,
      phoneNumber: profileForm.phoneNumber,
    });
    return parsed.success;
  }, [profileForm]);

  const loadAccountData = useAccountDataLoader({
    authenticated,
    setLoadingData,
    setRouteFailure,
    setAccountProfile,
    setAddresses,
    setOrders,
    setOrderDetailsById,
  });

  /** Keep ref aligned with state during render so child effects (e.g. ensureOrderDetailLoaded) never see stale cache after loadAccountData clears details. */
  orderDetailsRef.current = orderDetailsById;

  const orderDetailFetchRef = useRef<Set<string>>(new Set());

  const fetchOrderDetail = useCallback(
    async (orderId: string, forceRefresh: boolean) => {
      if (!forceRefresh && orderDetailsRef.current[orderId]) return;
      if (orderDetailFetchRef.current.has(orderId)) return;
      orderDetailFetchRef.current.add(orderId);
      setRouteFailure(null);
      try {
        const detail = await fetchApiEnvelope<AccountOrderDetailPayload>(
          `/api/account/orders/${encodeURIComponent(orderId)}`,
          { cache: "no-store" }
        );
        if (detail) {
          setOrderDetailsById((prev) => ({ ...prev, [orderId]: detail }));
        }
      } catch (e) {
        setRouteFailure(toRouteFailureUi("account", e));
      } finally {
        orderDetailFetchRef.current.delete(orderId);
      }
    },
    []
  );

  const ensureOrderDetailLoaded = useCallback(
    async (orderId: string) => {
      await fetchOrderDetail(orderId, false);
    },
    [fetchOrderDetail]
  );

  const refreshOrderDetail = useCallback(
    async (orderId: string) => {
      await fetchOrderDetail(orderId, true);
    },
    [fetchOrderDetail]
  );

  const cancelOrder = useCallback(
    async (orderId: string) => {
      setRouteFailure(null);
      try {
        await fetchApiEnvelope<{ orderId: string; statusId: string }>(
          `/api/account/orders/${encodeURIComponent(orderId)}/cancel`,
          { method: "POST" }
        );
        await loadAccountData();
        announce("Order cancelled.");
      } catch (e) {
        const ui = toRouteFailureUi("account", e);
        setRouteFailure(ui);
        announce(ui.message, "assertive");
      }
    },
    [announce, loadAccountData]
  );

  const cancelOrderItems = useCallback(
    async (orderId: string, orderDetailIds: string[]) => {
      if (!orderDetailIds.length) return;
      setRouteFailure(null);
      try {
        await fetchApiEnvelope<{ orderId: string; statusId: string }>(
          `/api/account/orders/${encodeURIComponent(orderId)}/cancel-items`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ orderDetailIds }),
          }
        );
        await loadAccountData();
        announce("Item cancelled.");
      } catch (e) {
        const ui = toRouteFailureUi("account", e);
        setRouteFailure(ui);
        announce(ui.message, "assertive");
      }
    },
    [announce, loadAccountData]
  );

  const requestReturn = useCallback(
    async (orderId: string, orderDetailIds: string[], reason: string) => {
      if (!orderDetailIds.length) return;
      const trimmedReason = reason.trim();
      if (!trimmedReason) return;
      setRouteFailure(null);
      try {
        await fetchApiEnvelope(`/api/account/orders/${encodeURIComponent(orderId)}/returns`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ orderDetailIds, reason: trimmedReason }),
        });
        await loadAccountData();
        announce("Return request submitted.");
      } catch (e) {
        const ui = toRouteFailureUi("account", e);
        setRouteFailure(ui);
        announce(ui.message, "assertive");
      }
    },
    [announce, loadAccountData]
  );

  const updateProfile = async () => {
    if (!canSaveProfile || savingProfile) return;
    setSavingProfile(true);
    setRouteFailure(null);
    try {
      const parsed = profileUpdateSchema.safeParse({
        firstName: profileForm.firstName,
        lastName: profileForm.lastName || undefined,
        gender: profileForm.gender || undefined,
        dateOfBirth: profileForm.dateOfBirth || undefined,
        phoneNumber: profileForm.phoneNumber,
      });
      if (!parsed.success) return;
      await fetchApiEnvelope("/api/account/profile", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ input: parsed.data }),
      });
      await loadAccountData();
      announce("Profile updated successfully.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setRouteFailure(ui);
      announce(ui.message, "assertive");
    } finally {
      setSavingProfile(false);
    }
  };

  const addAddress = async () => {
    if (!canSaveAddress || adding) return;
    setAdding(true);
    setError(null);
    setRouteFailure(null);
    try {
      const parsed = addressInputSchema.safeParse({
        recipientName: form.recipientName.trim() || null,
        phoneNumber: form.phoneNumber.trim() || null,
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
        body: JSON.stringify({ input: { ...parsed.data, isDefault: addresses.length === 0 } }),
      });
      setForm({
        recipientName: "",
        phoneNumber: "",
        country: "India",
        stateRegion: "",
        city: "",
        postalCode: "",
        road: "",
        apartmentNoOrName: "",
      });
      await loadAccountData();
      announce("Address saved successfully.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setRouteFailure(ui);
      announce(ui.message, "assertive");
    } finally {
      setAdding(false);
    }
  };

  const updateAddress = async (shippingAddressId: string) => {
    if (!canSaveAddress || adding) return;
    const row = addresses.find((a) => a.shippingAddressId === shippingAddressId);
    if (!row) return;
    setAdding(true);
    setError(null);
    setRouteFailure(null);
    try {
      const parsed = addressInputSchema.safeParse({
        recipientName: form.recipientName.trim() || null,
        phoneNumber: form.phoneNumber.trim() || null,
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
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          input: {
            shippingAddressId,
            ...parsed.data,
            isDefault: Boolean(row.isDefault),
          },
        }),
      });
      setForm({
        recipientName: "",
        phoneNumber: "",
        country: "India",
        stateRegion: "",
        city: "",
        postalCode: "",
        road: "",
        apartmentNoOrName: "",
      });
      await loadAccountData();
      announce("Address updated successfully.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setRouteFailure(ui);
      announce(ui.message, "assertive");
    } finally {
      setAdding(false);
    }
  };

  const deleteAddress = async (shippingAddressId: string) => {
    setRouteFailure(null);
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
      setRouteFailure(ui);
      announce(ui.message, "assertive");
    }
  };

  const setDefaultAddress = async (shippingAddressId: string) => {
    const row = addresses.find((a) => a.shippingAddressId === shippingAddressId);
    if (!row || row.isDefault) return;
    setRouteFailure(null);
    try {
      await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          input: {
            shippingAddressId: row.shippingAddressId,
            country: row.country,
            stateRegion: row.stateRegion,
            city: row.city,
            postalCode: row.postalCode,
            road: row.road ?? "",
            apartmentNoOrName: row.apartmentNoOrName ?? null,
            recipientName: row.recipientName ?? null,
            phoneNumber: row.phoneNumber ?? null,
            isDefault: true,
          },
        }),
      });
      await loadAccountData();
      announce("Default address updated.");
    } catch (e) {
      const ui = toRouteFailureUi("account", e);
      setRouteFailure(ui);
      announce(ui.message, "assertive");
    }
  };

  return (
    <div className="min-h-screen w-full min-w-0 bg-[var(--background)] text-[var(--foreground)]">
      <PageShell wide containerClassName="pt-4 pb-10 md:pt-6">
        {loadingSession ? (
          <p className="text-sm text-[var(--color-muted)]">Loading profile...</p>
        ) : !authenticated ? (
          <UnauthenticatedProfile openLogin={openLogin} />
        ) : (
          <AuthenticatedProfileSection
            routeFailure={routeFailure}
            loadAccountData={loadAccountData}
            openLogin={openLogin}
            displayName={displayName}
            displayEmail={displayEmail}
            loginMethodLabel={loginMethodLabel}
            accountProfile={accountProfile}
            error={error}
            loadingData={loadingData}
            addresses={addresses}
            orders={orders}
            orderDetailsById={orderDetailsById}
            form={form}
            setForm={setForm}
            canSaveAddress={canSaveAddress}
            adding={adding}
            addAddress={addAddress}
            updateAddress={updateAddress}
            deleteAddress={deleteAddress}
            setDefaultAddress={setDefaultAddress}
            profileForm={profileForm}
            setProfileForm={setProfileForm}
            canSaveProfile={canSaveProfile}
            savingProfile={savingProfile}
            updateProfile={updateProfile}
            ensureOrderDetailLoaded={ensureOrderDetailLoaded}
            refreshOrderDetail={refreshOrderDetail}
            cancelOrder={cancelOrder}
            cancelOrderItems={cancelOrderItems}
            requestReturn={requestReturn}
          />
        )}
      </PageShell>
    </div>
  );
}
