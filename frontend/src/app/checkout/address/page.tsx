"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { ArrowLeft, MapPin, Plus } from "lucide-react";
import { useSession } from "next-auth/react";
import { Country, State, City } from "country-state-city";
import { SiteHeader } from "@/components/site-header";
import { Dialog, DialogClose, DialogContent } from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { useStorefront } from "@/context/storefront-context";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { toRouteFailureUi } from "@/lib/route-state";
import { addressInputSchema } from "@/lib/validation-schemas";
import { cn } from "@/lib/utils";

type ShippingAddressRow = {
  shippingAddressId: string;
  userId?: string | null;
  road?: string | null;
  apartmentNoOrName?: string | null;
  city: string;
  stateRegion: string;
  postalCode: string;
  country: string;
};

function GoldDivider() {
  return (
    <div className="flex items-center gap-3">
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
      <div className="h-1.5 w-1.5 rounded-full bg-[#C9A646]" />
      <div className="h-px flex-1 bg-gradient-to-r from-transparent via-[#C9A646]/65 to-transparent" />
    </div>
  );
}

function sortByName<T extends { name: string }>(items: T[]): T[] {
  return [...items].sort((a, b) => a.name.localeCompare(b.name));
}

function formatAddressLine(a: ShippingAddressRow): string {
  const tail = [
    [a.apartmentNoOrName, a.road].filter(Boolean).join(", "),
    a.city,
    a.stateRegion,
    a.postalCode,
    a.country,
  ].filter((p) => p && String(p).trim());
  return tail.join(" | ") || "Saved address";
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

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [loadingList, setLoadingList] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [savingAddress, setSavingAddress] = useState(false);
  const [saveAddressError, setSaveAddressError] = useState<string | null>(null);
  const [newAddr, setNewAddr] = useState({
    countryIso: "IN",
    stateIso: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });

  useEffect(() => {
    if (status !== "authenticated") return;
    if (cartLines.length === 0) {
      router.replace("/bag");
    }
  }, [status, cartLines.length, router]);

  const loadAddresses = useCallback(async () => {
    if (status !== "authenticated") return;
    setLoadingList(true);
    setLoadError(null);
    try {
      const list = await fetchApiEnvelope<ShippingAddressRow[]>(
        "/api/account/addresses",
        { cache: "no-store" }
      );
      setAddresses(list);
      setSelectedId((prev) => {
        if (prev && list.some((a) => a.shippingAddressId === prev)) return prev;
        return list[0]?.shippingAddressId ?? null;
      });
    } catch (e) {
      setLoadError(toRouteFailureUi("account", e).message);
      setAddresses([]);
    } finally {
      setLoadingList(false);
    }
  }, [status]);

  useEffect(() => {
    void loadAddresses();
  }, [loadAddresses]);

  const countryOptions = useMemo(
    () => sortByName(Country.getAllCountries()),
    []
  );

  const stateOptions = useMemo(
    () => sortByName(State.getStatesOfCountry(newAddr.countryIso)),
    [newAddr.countryIso]
  );

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
    const countryName =
      Country.getCountryByCode(newAddr.countryIso)?.name ?? newAddr.countryIso;
    const stateName =
      stateOptions.find((s) => s.isoCode === newAddr.stateIso)?.name ??
      newAddr.stateIso;
    const parsed = addressInputSchema.safeParse({
      country: countryName.trim(),
      stateRegion: stateName.trim(),
      city: newAddr.city.trim(),
      postalCode: newAddr.postalCode.replace(/\D/g, "").slice(0, 6),
      road: newAddr.road.trim(),
      apartmentNoOrName: newAddr.apartmentNoOrName.trim() || null,
    });
    return parsed.success;
  }, [newAddr, stateOptions]);

  const saveNewAddress = async () => {
    if (!newAddrValid) return;
    setSavingAddress(true);
    setSaveAddressError(null);
    try {
      const countryName =
        Country.getCountryByCode(newAddr.countryIso)?.name ?? newAddr.countryIso;
      const stateName =
        stateOptions.find((s) => s.isoCode === newAddr.stateIso)?.name ??
        newAddr.stateIso;
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
      const input: Record<string, unknown> = parsed.data;
      const key = buildAddressIdempotencyKey(parsed.data);
      const created = await fetchApiEnvelope<ShippingAddressRow>(
        "/api/account/addresses",
        {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Idempotency-Key": key,
        },
        body: JSON.stringify({ input }),
      });
      if (!created?.shippingAddressId) {
        throw new Error("Address was not saved.");
      }
      setNewAddr({
        countryIso: "IN",
        stateIso: "",
        city: "",
        postalCode: "",
        road: "",
        apartmentNoOrName: "",
      });
      setShowAddForm(false);
      await loadAddresses();
      setSelectedId(created.shippingAddressId);
    } catch (e) {
      setSaveAddressError(toRouteFailureUi("account", e).message);
    } finally {
      setSavingAddress(false);
    }
  };

  const onPay = () => {
    if (!selectedId) return;
    void runCheckout({ shippingAddressId: selectedId });
  };

  const shell = (className: string, inner: ReactNode) => (
    <div
      className={cn(
        "min-h-screen w-full min-w-0 bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] text-[var(--foreground)]",
        className
      )}
    >
      <SiteHeader />
      <main className="mx-auto w-full max-w-7xl px-4 pb-24 pt-6 sm:px-6 sm:pt-8">
        <div className="rounded-[36px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.82),rgba(255,255,255,0.72))] p-4 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-6 lg:p-8">
          {inner}
        </div>
      </main>
    </div>
  );

  if (status === "loading") {
    return shell(
      "",
      <p className="py-16 text-center text-sm text-[#615A50]">Loading…</p>
    );
  }

  if (status === "unauthenticated") {
    return shell(
      "",
      <div className="mx-auto max-w-xl py-4">
        <Link
          href="/bag"
          className="inline-flex items-center gap-2 text-sm font-medium text-[#0F3D2E] transition hover:opacity-80"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to bag
        </Link>
        <div className="mt-6 flex flex-wrap items-center justify-between gap-3 border-b border-[#0F3D2E]/8 pb-6">
          <p className="inline-flex items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px]">
            Checkout
          </p>
        </div>
        <div className="my-6">
          <GoldDivider />
        </div>
        <h1 className="font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
          Sign in to continue
        </h1>
        <p className="mt-3 text-sm leading-relaxed text-[#615A50]">
          Sign in with Google or phone OTP, then you&apos;ll choose a delivery
          address for this order.
        </p>
        <button
          type="button"
          onClick={() => openLogin("/checkout/address")}
          className="mt-8 inline-flex h-14 w-full max-w-sm items-center justify-center rounded-full bg-[#0F3D2E] px-6 text-[11px] font-semibold uppercase tracking-[0.24em] text-[#F6F3EA] shadow-[0_18px_32px_rgba(15,61,46,0.18)] transition hover:-translate-y-0.5 hover:bg-[#0C3126]"
        >
          Sign in
        </button>
      </div>
    );
  }

  return shell(
    "",
    <div className="mx-auto max-w-3xl">
      <Link
        href="/bag"
        className="inline-flex items-center gap-2 text-sm font-medium text-[#0F3D2E] transition hover:opacity-80"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to bag
      </Link>

      <div className="mt-6 flex flex-wrap items-center justify-between gap-3 border-b border-[#0F3D2E]/8 pb-6">
        <p className="inline-flex items-center rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] sm:px-4 sm:text-[11px]">
          Checkout
        </p>
        <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">
          Step 1 · Delivery
        </p>
      </div>

      <div className="my-6">
        <GoldDivider />
      </div>

      <h1 className="font-display text-3xl leading-tight text-[#0F3D2E] sm:text-4xl">
        Delivery address
      </h1>
      <p className="mt-2 text-sm text-[#615A50]">
        Choose where we should ship this order. Manage addresses anytime from
        your profile.
      </p>

      {loadingList && (
        <p className="mt-10 text-sm text-[#615A50]">Loading your addresses…</p>
      )}

      {loadError && (
        <p className="mt-6 rounded-2xl border border-red-200 bg-red-50/80 px-4 py-3 text-sm text-red-900">
          {loadError}
        </p>
      )}

      {!loadingList && !loadError && (
        <section className="mt-10">
          <h2 className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">
            Saved addresses
          </h2>

          {addresses.length === 0 ? (
            <p className="mt-4 rounded-[22px] border border-dashed border-[#0F3D2E]/15 bg-[#FFFDF8]/80 px-4 py-8 text-center text-sm text-[#615A50]">
              You don&apos;t have any saved addresses yet.
            </p>
          ) : (
            <ul className="mt-4 space-y-3">
              {addresses.map((a) => {
                const sel = selectedId === a.shippingAddressId;
                return (
                  <li key={a.shippingAddressId}>
                    <button
                      type="button"
                      onClick={() => setSelectedId(a.shippingAddressId)}
                      className={cn(
                        "flex w-full items-start gap-4 rounded-[22px] border p-4 text-left transition",
                        sel
                          ? "border-[#0F3D2E] bg-[#0F3D2E]/[0.06] shadow-[0_2px_8px_rgba(15,61,46,0.08)]"
                          : "border-[#0F3D2E]/10 bg-[linear-gradient(180deg,#FFFDF9_0%,#FAF6EF_100%)] hover:border-[#0F3D2E]/20"
                      )}
                    >
                      <span
                        className={cn(
                          "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border-2",
                          sel
                            ? "border-[#0F3D2E] bg-[#0F3D2E]"
                            : "border-[#0F3D2E]/25 bg-white"
                        )}
                      >
                        {sel && (
                          <span className="h-2 w-2 rounded-full bg-white" />
                        )}
                      </span>
                      <span className="min-w-0 flex-1">
                        <span className="flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-[#8B816D]">
                          <MapPin className="h-3.5 w-3.5" />
                          Deliver to
                        </span>
                        <span className="mt-1 block text-sm leading-relaxed text-[#162019]">
                          {formatAddressLine(a)}
                        </span>
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}

          <div className="mt-6">
            <button
              type="button"
              onClick={() => {
                setShowAddForm(true);
                setSaveAddressError(null);
              }}
              className="inline-flex w-full items-center justify-center gap-2 rounded-[22px] border border-dashed border-[#0F3D2E]/20 bg-white/60 py-4 text-sm font-semibold text-[#0F3D2E] transition hover:border-[#C9A646]/50 hover:bg-[#FFF9EF]/90 sm:w-auto sm:px-8"
            >
              <Plus className="h-4 w-4" strokeWidth={2} />
              Add new address
            </button>
          </div>

          <Dialog
            open={showAddForm}
            onOpenChange={(open) => {
              if (!open) {
                setShowAddForm(false);
                setSaveAddressError(null);
              }
            }}
          >
            <DialogContent
              title="New address"
              titleClassName="text-[11px] uppercase tracking-[0.22em] text-[#8B816D]"
              className="max-w-lg rounded-[22px] border border-[#0F3D2E]/10"
              contentClassName="max-h-[min(85vh,640px)] overflow-y-auto p-4 sm:p-5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <input
                  value={newAddr.apartmentNoOrName}
                  onChange={(e) =>
                    setNewAddr((p) => ({
                      ...p,
                      apartmentNoOrName: e.target.value,
                    }))
                  }
                  autoComplete="address-line2"
                  placeholder="Apartment, building, or house name (optional)"
                  className="h-11 rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)] sm:col-span-2"
                />
                <input
                  value={newAddr.road}
                  onChange={(e) =>
                    setNewAddr((p) => ({ ...p, road: e.target.value }))
                  }
                  autoComplete="address-line1"
                  placeholder="Street / road"
                  className="h-11 rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)] sm:col-span-2"
                />
                <div className="sm:col-span-2">
                  <Select
                    value={newAddr.countryIso}
                    onValueChange={(countryIso) =>
                      setNewAddr((p) => ({
                        ...p,
                        countryIso,
                        stateIso: "",
                        city: "",
                      }))
                    }
                  >
                    <SelectTrigger aria-label="Country">
                      <SelectValue placeholder="Country" />
                    </SelectTrigger>
                    <SelectContent>
                      {countryOptions.map((c) => (
                        <SelectItem key={c.isoCode} value={c.isoCode}>
                          {c.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Select
                    value={newAddr.stateIso || undefined}
                    onValueChange={(stateIso) =>
                      setNewAddr((p) => ({ ...p, stateIso, city: "" }))
                    }
                    disabled={!newAddr.countryIso || stateOptions.length === 0}
                  >
                    <SelectTrigger aria-label="State or region">
                      <SelectValue placeholder="State / region" />
                    </SelectTrigger>
                    <SelectContent>
                      {stateOptions.map((s) => (
                        <SelectItem key={s.isoCode} value={s.isoCode}>
                          {s.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  {cityOptions.length > 0 ? (
                    <Select
                      value={newAddr.city || undefined}
                      onValueChange={(city) =>
                        setNewAddr((p) => ({ ...p, city }))
                      }
                      disabled={!newAddr.stateIso}
                    >
                      <SelectTrigger aria-label="City">
                        <SelectValue placeholder="City" />
                      </SelectTrigger>
                      <SelectContent>
                        {cityOptions.map((c) => (
                          <SelectItem key={c.name} value={c.name}>
                            {c.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  ) : (
                    <input
                      value={newAddr.city}
                      onChange={(e) =>
                        setNewAddr((p) => ({ ...p, city: e.target.value }))
                      }
                      disabled={!newAddr.stateIso}
                      placeholder="City"
                      className="h-11 w-full rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)] disabled:opacity-50"
                    />
                  )}
                </div>
                <input
                  value={newAddr.postalCode}
                  onChange={(e) =>
                    setNewAddr((p) => ({
                      ...p,
                      postalCode: e.target.value.replace(/\D/g, "").slice(0, 6),
                    }))
                  }
                  inputMode="numeric"
                  placeholder="Pincode (6 digits)"
                  className="h-11 rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)]"
                />
              </div>
              {saveAddressError && (
                <p className="mt-3 text-sm text-red-700">{saveAddressError}</p>
              )}
              <div className="mt-5 flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  disabled={!newAddrValid || savingAddress}
                  onClick={() => void saveNewAddress()}
                  className="rounded-full bg-[#0F3D2E] px-6 py-2.5 text-xs font-semibold uppercase tracking-[0.16em] text-[#F6F3EA] transition-opacity hover:opacity-90 disabled:opacity-50"
                >
                  {savingAddress ? "Saving…" : "Save address"}
                </button>
                <DialogClose asChild>
                  <button
                    type="button"
                    className="text-xs font-semibold uppercase tracking-[0.14em] text-[#615A50] transition hover:text-[#0F3D2E]"
                  >
                    Cancel
                  </button>
                </DialogClose>
              </div>
            </DialogContent>
          </Dialog>
        </section>
      )}

      {addresses.length > 0 && (
        <div className="mt-10 border-t border-[#0F3D2E]/8 pt-8">
          <button
            type="button"
            disabled={!selectedId || paymentLoading}
            onClick={onPay}
            className="inline-flex h-14 w-full max-w-md items-center justify-center rounded-full bg-[#C9A646] px-6 text-[11px] font-semibold uppercase tracking-[0.24em] text-white shadow-[0_16px_28px_rgba(201,166,70,0.28)] transition hover:-translate-y-0.5 hover:bg-[#B89435] disabled:opacity-50"
          >
            {paymentLoading ? "Opening payment…" : "Continue to payment"}
          </button>
          {paymentMessage && (
            <p className="mt-4 text-center text-xs text-[#615A50]">
              {paymentMessage}
            </p>
          )}
        </div>
      )}
    </div>
  );
}



