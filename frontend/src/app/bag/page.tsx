"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { useSession } from "next-auth/react";
import { SiteHeader } from "@/components/site-header";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { getGuestSessionId } from "@/lib/session";
import { fetchApiEnvelope } from "@/lib/api-envelope";
import { addressInputSchema } from "@/lib/validation-schemas";
import { BagEmptyState } from "@/domains/bag/components/bag-empty-state";
import { BagContent } from "@/domains/bag/components/bag-content";
import { BagMobileCheckoutBar } from "@/domains/bag/components/bag-mobile-checkout-bar";
import { useRazorpayCheckout } from "@/hooks/use-razorpay-checkout";

type CatalogSize = { sizeId: string; sizeName: string };

type ShippingAddressRow = {
  shippingAddressId: string;
  recipientName?: string | null;
  phoneNumber?: string | null;
  isDefault?: boolean;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string | null;
  apartmentNoOrName?: string | null;
};

type ShippingEstimate = {
  shippingAmountPaise: string;
  courierName?: string | null;
  estimatedDeliveryDays?: number | null;
  itemSubtotalPaise: string;
  orderTotalPaise: string;
  quoteAvailable: boolean;
  note?: string | null;
};

type AddressFormState = {
  recipientName: string;
  phoneNumber: string;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road: string;
  apartmentNoOrName: string;
};

function formatAddress(a: ShippingAddressRow | null): string {
  if (!a) return "Select a delivery address";
  const parts = [
    a.recipientName,
    a.phoneNumber,
    [a.apartmentNoOrName, a.road].filter(Boolean).join(", "),
    a.city,
    a.stateRegion,
    a.postalCode,
    a.country,
  ].filter((v) => v && String(v).trim());
  return parts.join(" | ");
}

export default function BagPage() {
  const router = useRouter();
  const { status } = useSession();
  const { openLogin } = useStorefrontLogin();
  const { cartLines, decCart, incCart, removeCart, toggleWish, wishlist, addToCart } = useStorefront();
  const { paymentLoading, paymentMessage, runCheckout } = useRazorpayCheckout();
  const reduceMotion = !!useReducedMotion();

  const [selectedLineIds, setSelectedLineIds] = useState<Set<string>>(new Set());
  const [openSizeForId, setOpenSizeForId] = useState<string | null>(null);
  const [catalogSizes, setCatalogSizes] = useState<CatalogSize[]>([]);

  const [addresses, setAddresses] = useState<ShippingAddressRow[]>([]);
  const [selectedAddressId, setSelectedAddressId] = useState<string | null>(null);
  const [addressPickerOpen, setAddressPickerOpen] = useState(false);
  const [addAddressOpen, setAddAddressOpen] = useState(false);
  const [savingAddress, setSavingAddress] = useState(false);
  const [addressError, setAddressError] = useState<string | null>(null);
  const [newAddress, setNewAddress] = useState<AddressFormState>({
    recipientName: "",
    phoneNumber: "",
    country: "India",
    stateRegion: "",
    city: "",
    postalCode: "",
    road: "",
    apartmentNoOrName: "",
  });

  const [shippingAmount, setShippingAmount] = useState(0);
  const [shippingLoading, setShippingLoading] = useState(false);
  const [shippingNote, setShippingNote] = useState<string | null>(null);

  useEffect(() => {
    const currentIds = new Set(cartLines.map((line) => line.id));
    setSelectedLineIds((prev) => new Set([...prev].filter((id) => currentIds.has(id))));
  }, [cartLines]);

  useEffect(() => {
    if (cartLines.length > 0 && selectedLineIds.size === 0) {
      setSelectedLineIds(new Set(cartLines.map((line) => line.id)));
    }
  }, [cartLines.length, cartLines, selectedLineIds.size]);

  useEffect(() => {
    const sessionId = getGuestSessionId();
    if (!sessionId) return;
    void fetch("/api/sizes", { headers: { "x-session-id": sessionId } })
      .then((response) => response.json())
      .then((data: { sizes?: CatalogSize[] }) => setCatalogSizes(data.sizes ?? []))
      .catch(() => setCatalogSizes([]));
  }, []);

  const loadAddresses = async () => {
    if (status !== "authenticated") return;
    try {
      const list = await fetchApiEnvelope<ShippingAddressRow[]>("/api/account/addresses", { cache: "no-store" });
      setAddresses(list);
      setSelectedAddressId((prev) => {
        if (prev && list.some((a) => a.shippingAddressId === prev)) return prev;
        return list.find((a) => a.isDefault)?.shippingAddressId ?? list[0]?.shippingAddressId ?? null;
      });
    } catch {
      setAddresses([]);
      setSelectedAddressId(null);
    }
  };

  useEffect(() => {
    void loadAddresses();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  const selectedAddress = useMemo(
    () => addresses.find((a) => a.shippingAddressId === selectedAddressId) ?? null,
    [addresses, selectedAddressId]
  );

  useEffect(() => {
    const estimate = async () => {
      if (status !== "authenticated" || !selectedAddressId || cartLines.length === 0) {
        setShippingAmount(0);
        setShippingNote(status === "authenticated" ? "Select address to calculate shipping." : "Sign in to calculate shipping.");
        return;
      }
      setShippingLoading(true);
      try {
        const row = await fetchApiEnvelope<ShippingEstimate>("/api/checkout/shipping-estimate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ shippingAddressId: selectedAddressId }),
        });
        const paise = Number.parseInt(row.shippingAmountPaise, 10);
        setShippingAmount(Number.isFinite(paise) ? paise / 100 : 0);
        setShippingNote(row.note ?? null);
      } catch {
        setShippingAmount(0);
        setShippingNote("Unable to fetch live shipping right now.");
      } finally {
        setShippingLoading(false);
      }
    };
    void estimate();
  }, [status, selectedAddressId, cartLines.length]);

  const allSelected = cartLines.length > 0 && selectedLineIds.size === cartLines.length;
  const selectedLines = useMemo(
    () => cartLines.filter((line) => selectedLineIds.has(line.id)),
    [cartLines, selectedLineIds]
  );
  const selectedCount = useMemo(
    () => selectedLines.reduce((sum, line) => sum + line.qty, 0),
    [selectedLines]
  );
  const selectedSubtotal = useMemo(
    () =>
      selectedLines.reduce(
        (sum, line) => sum + line.qty * ((line.product.pricePaise ?? Math.round(line.product.price * 100)) / 100),
        0
      ),
    [selectedLines]
  );

  const addressValid = useMemo(
    () =>
      addressInputSchema.safeParse({
        recipientName: newAddress.recipientName.trim() || null,
        phoneNumber: newAddress.phoneNumber.trim() || null,
        country: newAddress.country.trim(),
        stateRegion: newAddress.stateRegion.trim(),
        city: newAddress.city.trim(),
        postalCode: newAddress.postalCode.replace(/\D/g, "").slice(0, 6),
        road: newAddress.road.trim(),
        apartmentNoOrName: newAddress.apartmentNoOrName.trim() || null,
      }).success,
    [newAddress]
  );

  const handleSaveAddress = async () => {
    if (!addressValid || savingAddress) return;
    setAddressError(null);
    setSavingAddress(true);
    try {
      const payload = {
        recipientName: newAddress.recipientName.trim() || null,
        phoneNumber: newAddress.phoneNumber.trim() || null,
        country: newAddress.country.trim(),
        stateRegion: newAddress.stateRegion.trim(),
        city: newAddress.city.trim(),
        postalCode: newAddress.postalCode.replace(/\D/g, "").slice(0, 6),
        road: newAddress.road.trim(),
        apartmentNoOrName: newAddress.apartmentNoOrName.trim() || null,
        isDefault: addresses.length === 0,
      };
      const created = await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ input: payload }),
      });
      setAddAddressOpen(false);
      setNewAddress({
        recipientName: "",
        phoneNumber: "",
        country: "India",
        stateRegion: "",
        city: "",
        postalCode: "",
        road: "",
        apartmentNoOrName: "",
      });
      await loadAddresses();
      if (created?.shippingAddressId) {
        setSelectedAddressId(created.shippingAddressId);
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to save address";
      setAddressError(message);
    } finally {
      setSavingAddress(false);
    }
  };

  const makeDefaultAddress = async (address: ShippingAddressRow) => {
    await fetchApiEnvelope<ShippingAddressRow>("/api/account/addresses", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        input: {
          shippingAddressId: address.shippingAddressId,
          recipientName: address.recipientName ?? null,
          phoneNumber: address.phoneNumber ?? null,
          country: address.country,
          stateRegion: address.stateRegion,
          city: address.city,
          postalCode: address.postalCode,
          road: address.road ?? "",
          apartmentNoOrName: address.apartmentNoOrName ?? null,
          isDefault: true,
        },
      }),
    });
  };

  const handleCheckout = () => {
    if (status !== "authenticated") {
      openLogin("/bag");
      return;
    }
    if (!selectedAddressId) {
      setAddressPickerOpen(true);
      return;
    }
    void runCheckout({
      shippingAddressId: selectedAddressId,
      onSuccess: ({ orderId }) => router.push(`/checkout/success?orderId=${encodeURIComponent(orderId)}`),
      onFailure: ({ orderId, reason }) => {
        const params = new URLSearchParams();
        if (orderId) params.set("orderId", orderId);
        if (reason) params.set("reason", reason);
        router.push(`/checkout/failed?${params.toString()}`);
      },
    });
  };

  return (
    <div className="min-h-screen w-full min-w-0 bg-[linear-gradient(135deg,#EFE9DE_0%,#F7F3EB_45%,#EEE6D8_100%)] text-[var(--foreground)]">
      <SiteHeader />
      <div className="mx-auto w-full max-w-7xl rounded-[36px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.82),rgba(255,255,255,0.72))] p-4 shadow-[0_30px_90px_rgba(15,61,46,0.10)] backdrop-blur-xl sm:p-6 lg:h-[calc(100vh-100px)] lg:overflow-hidden lg:p-8">
        {cartLines.length === 0 ? (
          <BagEmptyState />
        ) : (
          <>
            <div className="mb-5 rounded-2xl border border-[#0F3D2E]/10 bg-white/70 p-4">
              <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">Delivery address</p>
              <p className="mt-2 text-sm text-[#162019]">{status === "authenticated" ? formatAddress(selectedAddress) : "Sign in to choose address"}</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {status === "authenticated" ? (
                  <>
                    <button
                      type="button"
                      onClick={() => setAddressPickerOpen(true)}
                      className="rounded-full border border-[#0F3D2E]/20 px-4 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-[#0F3D2E]"
                    >
                      Change address
                    </button>
                    <button
                      type="button"
                      onClick={() => setAddAddressOpen(true)}
                      className="rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-[#A37D34]"
                    >
                      Add new address
                    </button>
                  </>
                ) : (
                  <button
                    type="button"
                    onClick={() => openLogin("/bag")}
                    className="rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-[#A37D34]"
                  >
                    Sign in
                  </button>
                )}
              </div>
            </div>

            <BagContent
              cartLines={cartLines}
              selectedLineIds={selectedLineIds}
              selectedLines={selectedLines}
              selectedSubtotal={selectedSubtotal}
              selectedCount={selectedCount}
              shippingAmount={shippingAmount}
              shippingLoading={shippingLoading}
              shippingNote={shippingNote || paymentMessage}
              allSelected={allSelected}
              catalogSizes={catalogSizes}
              openSizeForId={openSizeForId}
              reduceMotion={reduceMotion}
              wishlist={wishlist}
              onToggleAll={() =>
                setSelectedLineIds(allSelected ? new Set() : new Set(cartLines.map((line) => line.id)))
              }
              onToggleOne={(id) =>
                setSelectedLineIds((prev) => {
                  const next = new Set(prev);
                  if (next.has(id)) next.delete(id);
                  else next.add(id);
                  return next;
                })
              }
              onSetOpenSizeForId={setOpenSizeForId}
              onDecCart={decCart}
              onIncCart={incCart}
              onRemoveCart={removeCart}
              onToggleWish={toggleWish}
              onAddToCart={addToCart}
              onCheckout={handleCheckout}
            />
          </>
        )}
      </div>

      {cartLines.length > 0 && (
        <BagMobileCheckoutBar
          selectedSubtotal={selectedSubtotal}
          shippingAmount={shippingAmount}
          selectedCount={selectedCount}
          onCheckout={handleCheckout}
        />
      )}

      <Dialog open={addressPickerOpen} onOpenChange={setAddressPickerOpen}>
        <DialogContent title="Choose address" className="max-w-xl">
          <div className="space-y-2">
            {addresses.length === 0 ? (
              <p className="text-sm text-[#615A50]">No saved addresses yet.</p>
            ) : (
              addresses.map((a) => (
                <button
                  key={a.shippingAddressId}
                  type="button"
                  onClick={() => {
                    setSelectedAddressId(a.shippingAddressId);
                    void makeDefaultAddress(a)
                      .catch(() => undefined)
                      .finally(() => {
                        setAddressPickerOpen(false);
                        void loadAddresses();
                      });
                  }}
                  className={`w-full rounded-xl border p-3 text-left ${
                    selectedAddressId === a.shippingAddressId ? "border-[#0F3D2E] bg-[#0F3D2E]/[0.06]" : "border-[#0F3D2E]/10 bg-white"
                  }`}
                >
                  <p className="text-sm text-[#162019]">{formatAddress(a)}</p>
                </button>
              ))
            )}
            <button
              type="button"
              onClick={() => {
                setAddressPickerOpen(false);
                setAddAddressOpen(true);
              }}
              className="mt-2 rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-[#A37D34]"
            >
              Add new address
            </button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={addAddressOpen} onOpenChange={setAddAddressOpen}>
        <DialogContent title="Add new address" className="max-w-xl">
          <div className="grid gap-3 sm:grid-cols-2">
            <input value={newAddress.recipientName} onChange={(e) => setNewAddress((p) => ({ ...p, recipientName: e.target.value }))} placeholder="Recipient name" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm sm:col-span-2" />
            <input value={newAddress.phoneNumber} onChange={(e) => setNewAddress((p) => ({ ...p, phoneNumber: e.target.value }))} placeholder="Phone number" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm sm:col-span-2" />
            <input value={newAddress.road} onChange={(e) => setNewAddress((p) => ({ ...p, road: e.target.value }))} placeholder="Road / street" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm sm:col-span-2" />
            <input value={newAddress.apartmentNoOrName} onChange={(e) => setNewAddress((p) => ({ ...p, apartmentNoOrName: e.target.value }))} placeholder="Apartment / house (optional)" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm sm:col-span-2" />
            <input value={newAddress.city} onChange={(e) => setNewAddress((p) => ({ ...p, city: e.target.value }))} placeholder="City" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm" />
            <input value={newAddress.stateRegion} onChange={(e) => setNewAddress((p) => ({ ...p, stateRegion: e.target.value }))} placeholder="State / region" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm" />
            <input value={newAddress.country} onChange={(e) => setNewAddress((p) => ({ ...p, country: e.target.value }))} placeholder="Country" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm" />
            <input value={newAddress.postalCode} onChange={(e) => setNewAddress((p) => ({ ...p, postalCode: e.target.value.replace(/\D/g, "").slice(0, 6) }))} placeholder="Pincode" className="h-10 rounded-lg border border-[#0F3D2E]/20 px-3 text-sm" />
          </div>
          {addressError ? <p className="mt-3 text-sm text-red-700">{addressError}</p> : null}
          <div className="mt-4 flex gap-2">
            <button
              type="button"
              disabled={!addressValid || savingAddress}
              onClick={() => void handleSaveAddress()}
              className="rounded-full bg-[#0F3D2E] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white disabled:opacity-50"
            >
              {savingAddress ? "Saving..." : "Save address"}
            </button>
            <button
              type="button"
              onClick={() => setAddAddressOpen(false)}
              className="rounded-full border border-[#0F3D2E]/20 px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-[#0F3D2E]"
            >
              Cancel
            </button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
