import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";

const STATS = [
  {
    label: "Orders today",
    value: "—",
    sub: "Will show today's orders once GraphQL is connected.",
  },
  {
    label: "Revenue (MTD)",
    value: "—",
    sub: "Will show month-to-date revenue from completed orders.",
  },
  {
    label: "Products",
    value: "—",
    sub: "Will show total live products in your catalog.",
  },
  {
    label: "Customers",
    value: "—",
    sub: "Will show total registered customers.",
  },
];

export default function AdminDashboardPage() {
  return (
    <Section compact className="max-w-6xl">
      <Kicker className="text-[var(--color-muted)]">Overview</Kicker>
      <SectionHeading size="default" className="mt-2">
        At a glance
      </SectionHeading>

      <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {STATS.map((card) => (
          <Card key={card.label} className="border-[var(--color-line)]">
            <CardTitle className="text-[var(--color-muted)]">
              {card.label}
            </CardTitle>
            <CardContent className="mt-2">
              <div className="font-display text-2xl font-medium tracking-tight text-[var(--color-ink)]">
                {card.value}
              </div>
              <div className="mt-1.5 text-xs leading-relaxed text-[var(--color-muted)]">
                {card.sub}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card className="mt-10 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Get started</CardTitle>
        <CardContent className="mt-2">
          <div className="space-y-3 text-sm leading-relaxed text-[var(--color-muted)]">
            <p>
              Wire this dashboard to your GraphQL backend to see live orders,
              revenue, products, and customers here.
            </p>
            <ul className="list-disc space-y-1 pl-5">
              <li>
                Confirm <code className="rounded bg-[var(--color-line)]/50 px-1 py-0.5 font-mono text-xs">NEXT_PUBLIC_GRAPHQL_URL</code> points to the
                GraphQL gateway (e.g. <code className="rounded bg-[var(--color-line)]/50 px-1 py-0.5 font-mono text-xs">http://localhost:8080/v2</code>).
              </li>
              <li>
                Ensure admin auth is configured (token or session) so metrics
                queries can run.
              </li>
              <li>
                Once wired, replace these placeholders with real summary
                queries.
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>
    </Section>
  );
}
