import { Card, CardContent } from "@/components/ui/card";

export default function AdminOrdersPage() {
  return (
    <Card>
      <CardContent className="py-8 text-center">
        <p className="text-[var(--color-muted)]">
          Orders list — connect to your backend.
        </p>
      </CardContent>
    </Card>
  );
}
