import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { SectionHeading } from "@/components/ui/typography";
import { Settings } from "lucide-react";

export default function AdminSettingsPage() {
  return (
    <div className="mx-auto max-w-6xl w-full">
      <div className="mb-8">
        <p className="text-sm text-[var(--color-muted)]">Settings</p>
        <SectionHeading size="default" className="mt-1">
          Store configuration
        </SectionHeading>
        <p className="mt-1 text-sm leading-relaxed text-[var(--color-muted)]">
          Payment, shipping, tax, and feature settings will appear here.
        </p>
      </div>

      <Card className="max-w-2xl rounded-xl border-[var(--color-line)] border-l-4 border-l-slate-500 bg-white shadow-[var(--admin-card-shadow)]">
        <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
          <Settings className="h-4 w-4 text-slate-500" />
          Coming soon
        </CardTitle>
        <CardContent className="mt-2 space-y-3 text-sm leading-relaxed text-[var(--color-muted)]">
          <p>
            Settings for Sudatta&apos;s will live here once the backend
            integration is complete.
          </p>
          <p>
            Typical items include payment providers, shipping zones, tax rules,
            and feature toggles.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
