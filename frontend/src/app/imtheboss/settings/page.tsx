import { Card, CardContent, CardTitle } from "@/components/ui/card";

export default function AdminSettingsPage() {
  return (
    <Card className="max-w-2xl">
      <CardTitle>Settings</CardTitle>
      <CardContent className="mt-2 space-y-2 text-sm text-[var(--color-muted)]">
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
  );
}
