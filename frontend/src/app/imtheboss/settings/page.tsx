import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";

export default function AdminSettingsPage() {
  return (
    <Section compact className="max-w-2xl">
      <Kicker className="text-[var(--color-muted)]">Settings</Kicker>
      <SectionHeading size="default" className="mt-2">
        Store configuration
      </SectionHeading>

      <Card className="mt-8 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Coming soon</CardTitle>
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
    </Section>
  );
}
