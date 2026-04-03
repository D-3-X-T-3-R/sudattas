"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useSession } from "next-auth/react";
import { Country, State, City } from "country-state-city";
import { SiteHeader } from "@/components/site-header";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { useStorefront } from "@/context/storefront-context";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { toRouteFailureUi, type RouteFailureUi } from "@/lib/route-state";
import { addressInputSchema } from "@/lib/validation-schemas";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import {
  CheckoutAuthenticatedView,
  CheckoutShell,
  CheckoutUnauthenticatedView,
  type NewAddressState,
  type ShippingAddressRow,
} from "@/domains/checkout/address/components/checkout-address-views";

function sortByName<T extends { name: string }>(items: T[]): T[] {
  return [...items].sort((a, b) => a.name.localeCompare(b.name));
}

function buildAddressIdempotencyKey(input: {
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road: string;
  apartmentNoOrName?: string | null;
}): string {
  const canonical = [
    input.country.trim().toLowerCase(),
    input.stateRegion.trim().toLowerCase(),
    input.city.trim().toLowerCase(),
    input.postalCode.replace(/\D/g, ""),
    input.road.trim().toLowerCase(),
    (input.apartmentNoOrName ?? "").trim().toLowerCase(),
  ].join("|");
  return `checkout-address-${canonical}`;
}

export default function CheckoutAddressPage() {
  const router = useRouter();
  const { status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const { cartLines } = useStorefront();
  const { paymentLoading, paymentMessage, runCheckout } = useRazorpayTest();
  const { announce } = useLiveAnnouncer();

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [loadFailure, setLoadFailure] = useState<RouteFailureUi | null>(null);
  const [loadingList, setLoadingList] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [savingAddress, setSavingAddress] = useState(false);
  const [saveAddressError, setSaveAddressError] = useState<string | null>(null);
  const [newAddr, setNewAddr] = useState<NewAddressState>({
    countryIso: "IN",
    stateIso: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });

  useEffect(() => {
    if (status !== "authenticated") return;
    if (cartLines.length === 0) router.replace("/bag");
  }, [status, cartLines.length, router]);

  const loadAddresses = useCallback(async () => {
    if (status !== "authenticated") return;
    setLoadingList(true);
    setLoadFailure(null);
    try {
      const list = await fetchApiEnvelope<ShippingAddressRow[]>("/api/account/addresses", { cache: "no-store" });
      setAddresses(list);
      setSelectedId((prev) => (prev && list.some((a) => a.shippingAddressId === prev) ? prev : list[0]?.shippingAddressId ?? null));
    } catch (e) {
      setLoadFailure(toRouteFailureUi("account", e));
      setAddresses([]);
    } finally {
      setLoadingList(false);
    }
  }, [status]);

  useEffect(() => {
    void loadAddresses();
  }, [loadAddresses]);

  const countryOptions = useMemo(() => sortByName(Country.getAllCountries()), []);
  const stateOptions = useMemo(() => sortByName(State.getStatesOfCountry(newAddr.countryIso)), [newAddr.countryIso]);
  const cityOptions = useMemo(() => {
    if (!newAddr.stateIso) return [];
    const raw = City.getCitiesOfState(newAddr.countryIso, newAddr.stateIso);
    const seen = new Set<string>();
    const unique = raw.filter((c) => {
      if (seen.has(c.name)) return false;
      seen.add(c.name);
      return true;
    });
    return sortByName(unique);
  }, [newAddr.countryIso, newAddr.stateIso]);

  const newAddrValid = useMemo(() => {
    const countryName = Country.getCountryByCode(newAddr.countryIso)?.name ?? newAddr.countryIso;
    const stateName = stateOptions.find((s) => s.isoCode === newAddr.stateIso)?.name ?? newAddr.stateIso;
    return addressInputSchema.safeParse({
      country: countryName.trim(),
      stateRegion: stateName.trim(),
      city: newAddr.city.trim(),
      postalCode: newAddr.postalCode.replace(/\D/g, "").slice(0, 6),
      road: newAddr.road.trim(),
      apartmentNoOrName: newAddr.apartmentNoOrName.trim() || null,
    }).success;
  }, [newAddr, stateOptions]);

  const saveNewAddress = async () => {
    if (!newAddrValid) return;
    setSavingAddress(true);
    setSaveAddressError(null);
    try {
      const countryName = Country.getCountryByCode(newAddr.countryIso)?.name ?? newAddr.countryIso;
      const stateName = stateOptions.find((s) => s.isoCode === newAddr.stateIso)?.name ?? newAddr.stateIso;
      const parsed = addressInputSchema.safeParse({
        country: countryName.trim(),
        stateRegion: stateName.trim(),
        city: newAddr.city.trim(),
        postalCode: newAddr.postalCode.replace(/\D/g, "").slice(0, 6),
        road: newAddr.road.trim(),
        apartmentNoOrName: newAddr.apartmentNoOrName.trim() || null,
      });
      if (!parsed.success) {
        setSaveAddressError(parsed.error.issues[0]?.message ?? "Invalid address.");
        return;
      }
      const key = buildAddressIdempotencyKey(parsed.data);
      const created = await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
        method: "POST",
        headers: { "Content-Type": "application/json", "Idempotency-Key": key },
        body: JSON.stringify({ input: parsed.data }),
      });
      if (!created?.shippingAddressId) throw new Error("Address was not saved.");
      setNewAddr({ countryIso: "IN", stateIso: "", city: "", postalCode: "", road: "", apartmentNoOrName: "" });
      setShowAddForm(false);
      await loadAddresses();
      setSelectedId(created.shippingAddressId);
      announce("Address saved successfully.");
    } catch (e) {
      const message = toRouteFailureUi("account", e).message;
      setSaveAddressError(message);
      announce(message, "assertive");
    } finally {
      setSavingAddress(false);
    }
  };

  const onPay = () => {
    if (!selectedId) return;
    void runCheckout({ shippingAddressId: selectedId });
  };

  return (
    <div className="min-h-screen w-full min-w-0 bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] text-[var(--foreground)]">
      <SiteHeader />
      <CheckoutShell>
        {status === "loading" ? (
          <p className="py-16 text-center text-sm text-[#615A50]">Loading...</p>
        ) : status === "unauthenticated" ? (
          <CheckoutUnauthenticatedView onSignIn={() => openLogin("/checkout/address")} />
        ) : (
            <CheckoutAuthenticatedView
              loadingList={loadingList}
              loadFailure={loadFailure}
              onRetryLoadAddresses={() => void loadAddresses()}
              onSignInAgain={() => openLogin("/checkout/address")}
              addresses={addresses}
            selectedId={selectedId}
            setSelectedId={setSelectedId}
            showAddForm={showAddForm}
            setShowAddForm={setShowAddForm}
            saveAddressError={saveAddressError}
            setSaveAddressError={setSaveAddressError}
            newAddr={newAddr}
            setNewAddr={(updater) => setNewAddr(updater)}
            countryOptions={countryOptions}
            stateOptions={stateOptions}
            cityOptions={cityOptions}
            newAddrValid={newAddrValid}
            savingAddress={savingAddress}
            onSaveAddress={() => void saveNewAddress()}
            paymentLoading={paymentLoading}
            paymentMessage={paymentMessage}
            onPay={onPay}
          />
        )}
      </CheckoutShell>
    </div>
  );
}
