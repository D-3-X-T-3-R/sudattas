import React from "react";
import { AdminLayout } from "../Layout";

const LIGHT = { surface: "#FFFFFF", border: "#E7E1D6", muted: "#78716c" };

export function Placeholder({ title, message }) {
  return (
    <AdminLayout title={title}>
      <div className="rounded-xl border p-8 text-center shadow-sm" style={{ borderColor: LIGHT.border, background: LIGHT.surface }}>
        <p style={{ color: LIGHT.muted }}>{message}</p>
      </div>
    </AdminLayout>
  );
}
