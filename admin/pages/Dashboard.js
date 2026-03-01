import React from "react";
import { AdminLayout } from "../Layout";

const LIGHT = {
  surface: "#FFFFFF",
  border: "#E7E1D6",
  muted: "#78716c",
  text: "#1c1917",
};

export function Dashboard() {
  return (
    <AdminLayout title="Dashboard">
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[
          { label: "Orders today", value: "—", sub: "Connect backend" },
          { label: "Revenue (MTD)", value: "—", sub: "Connect backend" },
          { label: "Products", value: "—", sub: "Connect backend" },
          { label: "Customers", value: "—", sub: "Connect backend" },
        ].map((card) => (
          <div
            key={card.label}
            className="rounded-xl border p-5 shadow-sm"
            style={{ borderColor: LIGHT.border, background: LIGHT.surface }}
          >
            <div className="text-xs font-medium uppercase tracking-wider" style={{ color: LIGHT.muted }}>
              {card.label}
            </div>
            <div className="mt-2 text-2xl font-semibold" style={{ color: LIGHT.text }}>{card.value}</div>
            <div className="mt-1 text-xs" style={{ color: LIGHT.muted }}>{card.sub}</div>
          </div>
        ))}
      </div>
      <div className="mt-8 rounded-xl border p-6 shadow-sm" style={{ borderColor: LIGHT.border, background: LIGHT.surface }}>
        <h2 className="text-sm font-semibold" style={{ color: LIGHT.muted }}>Quick actions</h2>
        <p className="mt-2 text-sm" style={{ color: LIGHT.muted }}>
          Wire this panel to your GraphQL backend (orders, products, inventory) to manage Sudatta's.
        </p>
      </div>
    </AdminLayout>
  );
}
