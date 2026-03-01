import React, { useState } from "react";
import { Link } from "react-router-dom";
import {
  LayoutDashboard,
  Package,
  ShoppingCart,
  Users,
  Settings,
  Menu,
  X,
} from "lucide-react";

export const NAV = [
  { to: "/imtheboss", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/imtheboss/orders", icon: ShoppingCart, label: "Orders" },
  { to: "/imtheboss/products", icon: Package, label: "Products" },
  { to: "/imtheboss/customers", icon: Users, label: "Customers" },
  { to: "/imtheboss/settings", icon: Settings, label: "Settings" },
];

// Light warm palette: ivory, cream, warm grays
const LIGHT = {
  bg: "#F7F5F0",       // ivory (matches storefront)
  surface: "#FFFFFF",  // white cards/sidebar
  border: "#E7E1D6",   // warm line
  muted: "#78716c",    // warm gray text (stone-500)
  text: "#1c1917",     // warm ink (stone-900)
  hover: "#F5F5F4",    // warm hover (stone-100)
  overlay: "rgba(247,245,240,0.85)",
};

export function AdminLayout({ children, title }) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="min-h-screen flex" style={{ background: LIGHT.bg, color: LIGHT.text }}>
      {sidebarOpen && (
        <button
          type="button"
          onClick={() => setSidebarOpen(false)}
          className="fixed inset-0 z-30 md:hidden"
          style={{ background: LIGHT.overlay }}
          aria-label="Close menu"
        />
      )}

      <aside
        className={`
          fixed md:static inset-y-0 left-0 z-40 w-64
          transform transition-transform duration-200 ease-out
          ${sidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"}
        `}
        style={{ background: LIGHT.surface, borderRight: `1px solid ${LIGHT.border}` }}
      >
        <div className="flex items-center justify-between h-14 px-4" style={{ borderBottom: `1px solid ${LIGHT.border}` }}>
          <span className="text-sm font-semibold tracking-wide" style={{ color: LIGHT.text }}>ADMIN</span>
          <button
            type="button"
            onClick={() => setSidebarOpen(false)}
            className="md:hidden p-2 rounded hover:opacity-80 border"
            style={{ borderColor: LIGHT.border }}
            aria-label="Close menu"
          >
            <X className="h-5 w-5" style={{ color: LIGHT.text }} />
          </button>
        </div>
        <nav className="p-3 space-y-1">
          {NAV.map(({ to, icon: Icon, label }) => (
            <Link
              key={to}
              to={to}
              onClick={() => setSidebarOpen(false)}
              className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors"
              style={{ color: LIGHT.muted }}
              onMouseEnter={(e) => { e.currentTarget.style.background = LIGHT.hover; e.currentTarget.style.color = LIGHT.text; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = LIGHT.muted; }}
            >
              <Icon className="h-5 w-5 shrink-0" />
              {label}
            </Link>
          ))}
        </nav>
        <div className="absolute bottom-0 left-0 right-0 p-3" style={{ borderTop: `1px solid ${LIGHT.border}` }}>
          <Link
            to="/"
            className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors"
            style={{ color: LIGHT.muted }}
            onMouseEnter={(e) => { e.currentTarget.style.background = LIGHT.hover; e.currentTarget.style.color = LIGHT.text; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = LIGHT.muted; }}
          >
            ← Back to store
          </Link>
        </div>
      </aside>

      <main className="flex-1 flex flex-col min-w-0">
        <header className="sticky top-0 z-20 flex items-center h-14 px-4 backdrop-blur border-b" style={{ borderColor: LIGHT.border, background: `${LIGHT.bg}ee` }}>
          <button
            type="button"
            onClick={() => setSidebarOpen(true)}
            className="md:hidden p-2 rounded mr-2 border hover:opacity-80"
            style={{ borderColor: LIGHT.border }}
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" style={{ color: LIGHT.text }} />
          </button>
          <h1 className="text-lg font-semibold" style={{ color: LIGHT.text }}>{title}</h1>
        </header>
        <div className="flex-1 p-4 md:p-6">{children}</div>
      </main>
    </div>
  );
}
