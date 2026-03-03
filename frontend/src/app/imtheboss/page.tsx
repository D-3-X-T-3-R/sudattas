import { Card, CardContent, CardTitle } from "@/components/ui/card";

const STATS = [
  { label: "Orders today", value: "—", sub: "Connect backend" },
  { label: "Revenue (MTD)", value: "—", sub: "Connect backend" },
  { label: "Products", value: "—", sub: "Connect backend" },
  { label: "Customers", value: "—", sub: "Connect backend" },
];

export default function AdminDashboardPage() {
  return (
    <>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {STATS.map((card) => (
          <Card key={card.label}>
            <CardTitle>{card.label}</CardTitle>
            <CardContent className="mt-2">
              <div className="text-2xl font-semibold text-[var(--color-ink)]">
                {card.value}
              </div>
              <div className="mt-1 text-xs text-[var(--color-muted)]">
                {card.sub}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      <Card className="mt-8">
        <CardTitle>Quick actions</CardTitle>
        <CardContent className="mt-2">
          <p className="text-sm text-[var(--color-muted)]">
            Wire this panel to your GraphQL backend (orders, products, inventory)
            to manage Sudatta&apos;s.
          </p>
        </CardContent>
      </Card>
    </>
  );
}
