"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export interface AddressDraft {
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road: string;
  apartmentNoOrName: string;
  recipientName: string;
  phoneNumber: string;
}

export const EMPTY_ADDRESS_DRAFT: AddressDraft = {
  country: "India",
  stateRegion: "",
  city: "",
  postalCode: "",
  road: "",
  apartmentNoOrName: "",
  recipientName: "",
  phoneNumber: "",
};

interface OrderCreateAddressFormProps {
  draft: AddressDraft;
  setDraft: (updater: (draft: AddressDraft) => AddressDraft) => void;
  isPending: boolean;
  error: string;
  onSave: () => void;
}

/** New-address form used inline by OrderCreateCustomerCard when the selected customer has no
 * saved address to reuse. Split out purely to keep that component under the line-count limit. */
export function OrderCreateAddressForm({
  draft,
  setDraft,
  isPending,
  error,
  onSave,
}: OrderCreateAddressFormProps) {
  return (
    <div className="mt-3 grid grid-cols-1 gap-2.5 rounded-lg border border-[var(--color-line)] p-3 sm:grid-cols-2">
      <Input
        value={draft.recipientName}
        onChange={(e) => setDraft((d) => ({ ...d, recipientName: e.target.value }))}
        placeholder="Recipient name"
        className="rounded-lg text-[15px]"
      />
      <Input
        value={draft.phoneNumber}
        onChange={(e) => setDraft((d) => ({ ...d, phoneNumber: e.target.value }))}
        placeholder="Phone number"
        className="rounded-lg text-[15px]"
      />
      <Input
        value={draft.road}
        onChange={(e) => setDraft((d) => ({ ...d, road: e.target.value }))}
        placeholder="Road / street"
        className="rounded-lg text-[15px] sm:col-span-2"
      />
      <Input
        value={draft.apartmentNoOrName}
        onChange={(e) => setDraft((d) => ({ ...d, apartmentNoOrName: e.target.value }))}
        placeholder="Apartment / building"
        className="rounded-lg text-[15px] sm:col-span-2"
      />
      <Input
        value={draft.city}
        onChange={(e) => setDraft((d) => ({ ...d, city: e.target.value }))}
        placeholder="City *"
        className="rounded-lg text-[15px]"
      />
      <Input
        value={draft.stateRegion}
        onChange={(e) => setDraft((d) => ({ ...d, stateRegion: e.target.value }))}
        placeholder="State *"
        className="rounded-lg text-[15px]"
      />
      <Input
        value={draft.postalCode}
        onChange={(e) => setDraft((d) => ({ ...d, postalCode: e.target.value }))}
        placeholder="Postal code *"
        className="rounded-lg text-[15px]"
      />
      <Input
        value={draft.country}
        onChange={(e) => setDraft((d) => ({ ...d, country: e.target.value }))}
        placeholder="Country"
        className="rounded-lg text-[15px]"
      />
      <Button
        type="button"
        size="sm"
        disabled={isPending}
        onClick={onSave}
        className="sm:col-span-2"
      >
        {isPending ? "Saving…" : "Save address"}
      </Button>
      {error && (
        <p className="text-sm text-red-600 sm:col-span-2" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
