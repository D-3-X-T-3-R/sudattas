"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { MailWarning } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { useToast } from "@/components/ui/toast";
import {
  fetchAbandonedCartSettings,
  updateAbandonedCartSettings,
  type AbandonedCartSettingsRow,
} from "@/lib/admin-app-settings";

const QUERY_KEY = ["admin", "abandoned-cart-settings"];

interface DraftFields {
  enabled: boolean;
  delayHours: string;
  /** Shown to the admin in minutes; converted to/from seconds only at the API boundary,
   * matching how money fields show rupees and convert to paise only when sent. */
  pollIntervalMinutes: string;
}

function secondsToMinutesInput(seconds: string): string {
  const n = Number(seconds);
  return Number.isFinite(n) ? String(Math.round(n / 60)) : "";
}

function draftFromRow(row: AbandonedCartSettingsRow): DraftFields {
  return {
    enabled: row.enabled,
    delayHours: row.delayHours,
    pollIntervalMinutes: secondsToMinutesInput(row.pollIntervalSec),
  };
}

/** Owns the editable draft, seeded once from `initial` via useState's lazy initializer (no
 * effect needed) — remounted with a fresh key whenever a save changes the saved row. */
function AbandonedCartSettingsForm({ initial }: { initial: AbandonedCartSettingsRow }) {
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const [draft, setDraft] = useState<DraftFields>(() => draftFromRow(initial));
  const [error, setError] = useState("");

  const saveMutation = useMutation({
    mutationFn: (d: DraftFields) => {
      const delayHours = Number(d.delayHours);
      const pollMinutes = Number(d.pollIntervalMinutes);
      if (!Number.isFinite(delayHours) || delayHours <= 0) {
        throw new Error("Delay hours must be a positive number.");
      }
      if (!Number.isFinite(pollMinutes) || pollMinutes <= 0) {
        throw new Error("Check interval must be a positive number of minutes.");
      }
      return updateAbandonedCartSettings({
        enabled: d.enabled,
        delayHours: String(Math.round(delayHours)),
        pollIntervalSec: String(Math.round(pollMinutes * 60)),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
      setError("");
      showToast({ title: "Abandoned cart recovery", description: "Settings saved." });
    },
    onError: (err: Error) => setError(err.message || "Failed to save settings."),
  });

  return (
    <div className="space-y-4">
      <p className="text-sm text-[var(--color-muted)]">
        A background worker periodically emails customers who left items in their cart. Changes
        here take effect on its next check — no restart needed.
      </p>
      <label className="flex items-center gap-2 text-sm text-[var(--color-ink)]">
        <input
          type="checkbox"
          checked={draft.enabled}
          onChange={(e) => setDraft({ ...draft, enabled: e.target.checked })}
          className="h-4 w-4 rounded border-[var(--color-line)]"
        />
        Enabled
      </label>
      <div className="grid gap-3 sm:grid-cols-2">
        <div>
          <label
            htmlFor="abandoned-cart-delay-hours"
            className="mb-1 block text-xs font-medium text-[var(--color-muted)]"
          >
            Cart considered abandoned after (hours)
          </label>
          <Input
            id="abandoned-cart-delay-hours"
            value={draft.delayHours}
            onChange={(e) => setDraft({ ...draft, delayHours: e.target.value })}
            inputMode="numeric"
            className="h-10 rounded-lg text-[15px]"
          />
        </div>
        <div>
          <label
            htmlFor="abandoned-cart-poll-minutes"
            className="mb-1 block text-xs font-medium text-[var(--color-muted)]"
          >
            Check for new abandoned carts every (minutes)
          </label>
          <Input
            id="abandoned-cart-poll-minutes"
            value={draft.pollIntervalMinutes}
            onChange={(e) => setDraft({ ...draft, pollIntervalMinutes: e.target.value })}
            inputMode="numeric"
            className="h-10 rounded-lg text-[15px]"
          />
        </div>
      </div>
      {error ? (
        <p className="text-sm text-red-600" role="alert">
          {error}
        </p>
      ) : null}
      <Button
        type="button"
        size="sm"
        disabled={saveMutation.isPending}
        onClick={() => saveMutation.mutate(draft)}
      >
        {saveMutation.isPending ? "Saving…" : "Save settings"}
      </Button>
    </div>
  );
}

/** Card for the one worker-schedule setting that's admin-configurable so far (abandoned-cart
 * recovery) — backed by the generic AppSettings key/value store, so a save here takes effect
 * on the worker's next poll with no container restart. */
export function AbandonedCartSettingsCard() {
  const settingsQuery = useQuery({ queryKey: QUERY_KEY, queryFn: fetchAbandonedCartSettings });

  return (
    <AdminTableCard
      title="Abandoned cart recovery"
      icon={<MailWarning className="h-4 w-4 text-[var(--color-green)]" />}
    >
      {settingsQuery.isLoading ? (
        <p className="py-4 text-center text-sm text-[var(--color-muted)]">Loading…</p>
      ) : null}
      {settingsQuery.isError ? (
        <p className="py-4 text-center text-sm text-rose-700">Could not load settings.</p>
      ) : null}
      {settingsQuery.data ? (
        <AbandonedCartSettingsForm
          key={`${settingsQuery.data.enabled}:${settingsQuery.data.delayHours}:${settingsQuery.data.pollIntervalSec}`}
          initial={settingsQuery.data}
        />
      ) : null}
    </AdminTableCard>
  );
}
