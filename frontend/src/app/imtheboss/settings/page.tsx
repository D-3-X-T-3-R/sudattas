import { Settings } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";

export default function AdminSettingsPage() {
  return (
    <AdminPageShell
      label="Settings"
      title="Store configuration"
      description="Payment, shipping, tax, and operational settings will appear here."
    >
      <AdminTableCard
        title="Coming soon"
        icon={<Settings className="h-4 w-4 text-[var(--color-green)]" />}
        className="max-w-3xl"
      >
        <div className="space-y-3 text-[15px] leading-relaxed text-[var(--color-muted)]">
          <p>
            Settings for Sudatta&apos;s will be available here once these
            controls are ready to turn on.
          </p>
          <p>
            This will include things like payment options, delivery areas, and
            tax rules.
          </p>
        </div>
      </AdminTableCard>
    </AdminPageShell>
  );
}
