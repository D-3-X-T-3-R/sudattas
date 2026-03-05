import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";

export default function AdminCustomersPage() {
  return (
    <Section compact className="max-w-2xl">
      <Kicker className="text-[var(--color-muted)]">Customers</Kicker>
      <SectionHeading size="default" className="mt-2">
        Customer list
      </SectionHeading>

      <Card className="mt-8 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Coming soon</CardTitle>
        <CardContent className="mt-2 space-y-3 text-sm leading-relaxed text-[var(--color-muted)]">
          <p>
            Once wired to your backend, this view will list customers and their
            recent activity.
          </p>
          <p>
            You can later add search, filters (for example, &quot;high value&quot;, &quot;recent&quot;),
            and links into individual customer profiles.
          </p>
        </CardContent>
      </Card>
    </Section>
  );
}
