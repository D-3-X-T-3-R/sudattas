import React from "react";
import { Routes, Route } from "react-router-dom";
import { Dashboard, Placeholder, Products } from "./pages";

export default function AdminRoutes() {
  return (
    <Routes>
      <Route index element={<Dashboard />} />
      <Route
        path="orders"
        element={
          <Placeholder title="Orders" message="Orders list — connect to your backend." />
        }
      />
      <Route path="products" element={<Products />} />
      <Route
        path="customers"
        element={
          <Placeholder title="Customers" message="Customers — connect to your backend." />
        }
      />
      <Route
        path="settings"
        element={
          <Placeholder title="Settings" message="Settings — connect to your backend." />
        }
      />
    </Routes>
  );
}
