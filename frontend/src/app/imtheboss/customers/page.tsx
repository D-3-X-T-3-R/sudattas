import { Card, CardContent } from "@/components/ui/card";

export default function AdminCustomersPage() {
  return (
    <Card>
      <CardContent className="py-8 text-center">
        <p className="text-[var(--color-muted)]">
          Customers — connect to your backend.
        </p>
      </CardContent>
    </Card>
  );
}
