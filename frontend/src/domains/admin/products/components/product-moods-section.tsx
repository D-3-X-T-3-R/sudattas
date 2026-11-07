"use client";

import { Pencil, Trash2 } from "lucide-react";
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
  /** Rename/delete a mood — distinct from linking it to the current product (checkbox above). */
  editingMoodId: string | null;
  editingMoodName: string;
  setEditingMoodName: Dispatch<SetStateAction<string>>;
  moodManageError: string;
  updateMoodMutation: { isPending: boolean };
  onBeginEditMood: (mood: ProductMoodRow) => void;
  onCancelEditMood: () => void;
  onSaveEditMood: () => void;
  onRequestDeleteMood: (mood: ProductMoodRow) => void;
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
  editingMoodId,
  editingMoodName,
  setEditingMoodName,
  moodManageError,
  updateMoodMutation,
  onBeginEditMood,
  onCancelEditMood,
  onSaveEditMood,
  onRequestDeleteMood,
}: ProductMoodsSectionProps) {
  const moodNameInputId = "product-new-mood-name";
  const moodNameErrorId = moodCreateError ? "product-new-mood-error" : undefined;
  return (
    <div className="mt-6 border-t border-[var(--color-line)] pt-5">
      <h3 className="text-[15px] font-semibold text-[var(--color-ink)]">
        Moods (optional)
      </h3>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">
        Select one or more moods to link to this product. Use the pencil/trash icons to rename or
        remove a mood everywhere it&apos;s used.
      </p>
      <div className="mt-3 grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2 lg:grid-cols-3">
        {existingMoods.map((m) =>
          editingMoodId === m.moodId ? (
            <div key={m.moodId} className="flex items-center gap-1.5">
              <Input
                value={editingMoodName}
                onChange={(e) => setEditingMoodName(e.target.value)}
                className="h-9 min-w-0 flex-1 rounded-lg text-[15px]"
                autoFocus
              />
              <Button
                type="button"
                size="sm"
                disabled={updateMoodMutation.isPending}
                onClick={onSaveEditMood}
                className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
              >
                {updateMoodMutation.isPending ? "Saving…" : "Save"}
              </Button>
              <Button type="button" size="sm" variant="outline" onClick={onCancelEditMood}>
                Cancel
              </Button>
            </div>
          ) : (
            <div key={m.moodId} className="flex items-center gap-1.5">
              <label
                className={cn(
                  "flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 text-[15px] text-[var(--color-ink)]",
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
                  className="h-5 w-5 shrink-0 rounded border-[var(--color-line)] text-[var(--color-accent-gold)] focus:ring-[var(--color-accent-gold)]/50"
                />
                <span className="truncate">{m.moodName}</span>
              </label>
              <button
                type="button"
                onClick={() => onBeginEditMood(m)}
                aria-label={`Rename ${m.moodName}`}
                className="rounded-md p-1 text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-ink)]"
              >
                <Pencil className="h-3.5 w-3.5" />
              </button>
              <button
                type="button"
                onClick={() => onRequestDeleteMood(m)}
                aria-label={`Delete ${m.moodName}`}
                className="rounded-md p-1 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          )
        )}
      </div>
      {moodManageError && (
        <p className="mt-2 text-sm text-red-600" role="alert">
          {moodManageError}
        </p>
      )}
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
