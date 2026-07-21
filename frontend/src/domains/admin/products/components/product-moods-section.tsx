"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { ProductMoodRow } from "@/lib/admin-queries";
import type { Dispatch, SetStateAction } from "react";

type ProductMoodsSectionProps = {
  existingMoods: ProductMoodRow[];
  selectedMoodIds: string[];
  setSelectedMoodIds: Dispatch<SetStateAction<string[]>>;
  newMoodName: string;
  setNewMoodName: Dispatch<SetStateAction<string>>;
  moodCreateError: string;
  setMoodCreateError: Dispatch<SetStateAction<string>>;
  createMoodMutation: {
    isPending: boolean;
    mutate: (name: string) => void;
  };
};

export function ProductMoodsSection({
  existingMoods,
  selectedMoodIds,
  setSelectedMoodIds,
  newMoodName,
  setNewMoodName,
  moodCreateError,
  setMoodCreateError,
  createMoodMutation,
}: ProductMoodsSectionProps) {
  const moodNameInputId = "product-new-mood-name";
  const moodNameErrorId = moodCreateError ? "product-new-mood-error" : undefined;
  return (
    <div className="mt-6 border-t border-[var(--color-line)] pt-5">
      <h3 className="text-[15px] font-semibold text-[var(--color-ink)]">
        Moods (optional)
      </h3>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">
        Select one or more moods to link to this product.
      </p>
      <div className="mt-3 grid grid-cols-2 gap-x-6 gap-y-2.5 sm:grid-cols-3 lg:grid-cols-4">
        {existingMoods.map((m) => (
          <label
            key={m.moodId}
            className={cn(
              "flex cursor-pointer items-center gap-2.5 text-[15px] text-[var(--color-ink)]",
              "focus-within:outline-none focus-within:ring-2 focus-within:ring-[var(--color-accent-gold)]/50 focus-within:ring-offset-1 rounded"
            )}
          >
            <input
              type="checkbox"
              checked={selectedMoodIds.includes(m.moodId)}
              onChange={() => {
                setSelectedMoodIds((prev) =>
                  prev.includes(m.moodId) ? prev.filter((id) => id !== m.moodId) : [...prev, m.moodId]
                );
              }}
              className="h-5 w-5 rounded border-[var(--color-line)] text-[var(--color-accent-gold)] focus:ring-[var(--color-accent-gold)]/50"
            />
            <span>{m.moodName}</span>
          </label>
        ))}
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-2.5">
        <label htmlFor={moodNameInputId} className="sr-only">
          New mood name
        </label>
        <Input
          id={moodNameInputId}
          placeholder="New mood name"
          value={newMoodName}
          onChange={(e) => {
            setNewMoodName(e.target.value);
            setMoodCreateError("");
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              const name = newMoodName.trim();
              if (name) createMoodMutation.mutate(name);
            }
          }}
          className="h-11 max-w-[14rem] rounded-lg text-[15px]"
          aria-invalid={Boolean(moodCreateError)}
          aria-describedby={moodNameErrorId}
        />
        <Button
          type="button"
          variant="outline"
          disabled={!newMoodName.trim() || createMoodMutation.isPending}
          onClick={() => {
            const name = newMoodName.trim();
            if (name) createMoodMutation.mutate(name);
          }}
        >
          {createMoodMutation.isPending ? "Adding..." : "Add mood"}
        </Button>
      </div>
      {moodCreateError && (
        <p id="product-new-mood-error" className="mt-2 text-sm text-red-600" role="alert">
          {moodCreateError}
        </p>
      )}
      {existingMoods.length === 0 && !newMoodName && (
        <p className="mt-2 text-sm text-[var(--color-muted)]">
          No moods in the system yet. Add one above or add moods from seed data.
        </p>
      )}
    </div>
  );
}
