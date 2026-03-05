import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";

export default function AdminOrdersPage() {
  return (
    <Section compact className="max-w-2xl">
      <Kicker className="text-[var(--color-muted)]">Orders</Kicker>
      <SectionHeading size="default" className="mt-2">
        Order management
      </SectionHeading>

      <Card className="mt-8 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Coming soon</CardTitle>
        <CardContent className="mt-2 space-y-3 text-sm leading-relaxed text-[var(--color-muted)]">
          <p>
            When your backend is connected, recent orders and their statuses will
            appear here.
          </p>
          <p>
            Start by wiring GraphQL queries for orders, then add filters for date
            range, status, and payment state.
          </p>
        </CardContent>
      </Card>
    </Section>
  );
}
