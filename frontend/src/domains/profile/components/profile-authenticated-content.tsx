/* eslint-disable max-lines, max-lines-per-function */
"use client";

import Image from "next/image";
import Link from "next/link";
import {
  useEffect,
  useMemo,
  useState,
  type ComponentType,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
  type SVGProps,
} from "react";
import type { AddressFormState, ProfileFormState } from "@/domains/profile/types";
import { formatInrFromPaise } from "@/lib/money";
import { Check, MapPin, X } from "lucide-react";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Kicker, HeroHeading } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

export type ShippingAddressRow = {
  shippingAddressId: string;
  userId?: string | null;
  isDefault?: boolean;
  recipientName?: string | null;
  phoneNumber?: string | null;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string | null;
  apartmentNoOrName?: string | null;
};

export type AccountOrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  cancelWindowEndsAt?: string | null;
  paymentMethod?: string | null;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  statusName: string;
  invoiceId?: string | null;
  invoiceNumber?: string | null;
  invoiceGeneratedAt?: string | null;
  invoiceAvailable?: boolean | null;
  invoiceUrl?: string | null;
  cancelWindowHours?: number;
  returnWindowDays?: number;
};

export type AccountProfileRow = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
  firstName?: string | null;
  lastName?: string | null;
  gender?: string | null;
  dateOfBirth?: string | null;
};

export type AccountOrderDetailRow = {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  pricePaise: string;
  lineTotalMinor?: string;
  itemStatus?: string;
  cancelledAt?: string | null;
  priceFormatted: string;
  productDetails?: Array<{
    productId?: string;
    name?: string;
    formatted?: string;
    images?: Array<{
      url?: string | null;
      thumbnailUrl?: string | null;
      thumbnail_url?: string | null;
    }>;
  }>;
};

export type AccountOrderDetailPayload = {
  order: AccountOrderRow & {
    orderDetails?: AccountOrderDetailRow[];
  };
  statusName: string;
  /** From Orders.refund_settlement_status (GraphQL). */
  refundSettlementStatus?: string | null;
  paymentIntents: Array<{
    intentId: string;
    amountPaise: string;
    currency?: string | null;
    status: string;
    razorpayPaymentId?: string | null;
    createdAt: string;
  }>;
  shipments: Array<{
    shipmentId: string;
    status: string;
    carrier?: string | null;
    awbCode?: string | null;
    createdAt: string;
    deliveredAt?: string | null;
    trackingEventsJson?: string | null;
    shiprocketStatusId?: string | null;
    shiprocketStatusLabel?: string | null;
  }>;
  events: Array<{
    eventId: string;
    eventType: string;
    fromStatus: string;
    toStatus: string;
    actorType: string;
    message: string;
    createdAt: string;
  }>;
  fulfillmentState: string;
  paymentState: string;
  returnWindowDays: number;
  returnRequests: Array<{
    returnId: string;
    orderId: string;
    userId: string;
    status: string;
    reason: string;
    createdAt: string;
    receivedAt?: string | null;
    refundAttemptId?: string | null;
    items: Array<{
      returnId: string;
      orderDetailId: string;
      quantity: string;
      refundAmountMinor: string;
      status: string;
    }>;
  }>;
  refundSummary?: {
    itemRefundMinor: number;
    shippingRefundMinor: number;
    totalRefundMinor: number;
    totalRefundFormatted: string;
    itemRefundFormatted: string;
    shippingRefundFormatted: string;
  };
};

type ProfileNavId = "profile" | "orders" | "addresses" | "settings" | "support";
type SupportCategory = "order" | "payment" | "refund" | "shipping" | "account";

function formatAddress(a: ShippingAddressRow): string {
  const parts = [
    a.recipientName,
    a.phoneNumber,
    [a.apartmentNoOrName, a.road].filter(Boolean).join(", "),
    a.city,
    a.stateRegion,
    a.postalCode,
    a.country,
  ].filter((v) => v && String(v).trim());
  return parts.join(", ");
}

/** Address line only, without name/phone which are rendered separately in the address card. */
function formatAddressBody(a: ShippingAddressRow): string {
  const parts = [
    [a.apartmentNoOrName, a.road].filter(Boolean).join(", "),
    a.city,
    a.stateRegion,
    a.postalCode,
    a.country,
  ].filter((v) => v && String(v).trim());
  return parts.join(", ");
}

function formatOrderDateShort(raw: string): string {
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

type TrackingStepState = "done" | "current" | "pending";
type RefundTrackingState = "none" | "initiated" | "processed" | "failed";

function fulfillmentTrackingSteps(fulfillmentState: string | undefined): { label: string; step: TrackingStepState }[] {
  const f = (fulfillmentState ?? "").toLowerCase();
  const delivered = f.includes("delivered");
  const shipped = delivered || f.includes("transit") || f.includes("shipped") || f.includes("in_transit");
  return [
    { label: "Placed", step: "done" },
    { label: "Shipped", step: delivered ? "done" : shipped ? "current" : "pending" },
    { label: "Delivered", step: delivered ? "done" : "pending" },
  ];
}

export type CourierTrackingStep = { label: string; at?: string; location?: string };

/** Normalized steps for the storefront; also accepts common Shiprocket-style keys on each object. */
export function parseShipmentTrackingJson(raw: string | null | undefined): CourierTrackingStep[] {
  if (!raw?.trim()) return [];
  try {
    const data = JSON.parse(raw) as unknown;
    if (!Array.isArray(data)) return [];
    const out: CourierTrackingStep[] = [];
    for (const row of data) {
      if (!row || typeof row !== "object") continue;
      const o = row as Record<string, unknown>;
      const labelRaw =
        (typeof o.label === "string" && o.label) ||
        (typeof o.activity === "string" && o.activity) ||
        (typeof o.status === "string" && o.status) ||
        (typeof o.scan === "string" && o.scan) ||
        (typeof o.current_status === "string" && o.current_status) ||
        "";
      const label = labelRaw.trim();
      if (!label) continue;
      const at =
        (typeof o.at === "string" && o.at) ||
        (typeof o.date === "string" && o.date) ||
        (typeof o.current_timestamp === "string" && o.current_timestamp) ||
        undefined;
      const location =
        (typeof o.location === "string" && o.location) ||
        (typeof o.scan_location === "string" && o.scan_location) ||
        undefined;
      out.push({ label, at, location });
    }
    return out.sort((a, b) => {
      const ta = a.at ? Date.parse(a.at) : NaN;
      const tb = b.at ? Date.parse(b.at) : NaN;
      if (!Number.isNaN(ta) && !Number.isNaN(tb) && ta !== tb) return ta - tb;
      return 0;
    });
  } catch {
    return [];
  }
}

function formatCourierStepTime(at?: string): string | null {
  if (!at?.trim()) return null;
  const d = new Date(at);
  if (Number.isNaN(d.getTime())) return at.trim();
  return d.toLocaleString("en-IN", { dateStyle: "medium", timeStyle: "short" });
}

function primaryShipmentForOrder(detail: AccountOrderDetailPayload | undefined) {
  const list = detail?.shipments ?? [];
  const withTracks = list.find((s) => (s.trackingEventsJson ?? "").trim().length > 0);
  return withTracks ?? list[0];
}

function refundTrackingStateForOrder(
  statusName: string | undefined,
  detail: AccountOrderDetailPayload | undefined
): RefundTrackingState {
  const rs = detail?.refundSettlementStatus?.trim().toLowerCase();
  if (rs === "refund_failed") return "failed";
  if (rs === "refund_processed") return "processed";
  if (rs === "refund_pending") return "initiated";
  const status = (statusName ?? "").trim().toLowerCase();
  const events = detail?.events ?? [];
  const eventTypes = events.map((e) => (e.eventType ?? "").trim().toLowerCase());
  if (eventTypes.some((x) => x === "refund_failed")) return "failed";
  if (eventTypes.some((x) => x === "refund_recorded")) return "processed";
  if (eventTypes.some((x) => x === "refund_initiated")) return "initiated";
  if (status.includes("refund")) return "processed";
  if (status.includes("cancel")) return "initiated";
  return "none";
}

function customerOrderStatusHeadline(
  statusName: string | undefined,
  detail: AccountOrderDetailPayload | undefined
): string {
  const base = (statusName ?? "").trim();
  const sn = base.toLowerCase();
  if (sn.includes("partially_cancelled")) return "Partially cancelled";
  if (sn.includes("cancel_pending")) return "Cancellation in progress · awaiting courier";
  const paid =
    (detail?.paymentState ?? "").toLowerCase().includes("paid");
  const refundState = refundTrackingStateForOrder(statusName, detail);
  if ((sn.includes("cancelled") || sn.includes("canceled")) && paid) {
    if (refundState === "failed") return "Cancelled · refund failed";
    if (refundState === "initiated") return "Cancelled · refund processing";
    if (refundState === "processed" || sn.includes("refund")) return "Cancelled · refunded";
    return "Cancelled · settlement updating";
  }
  return base || statusName?.trim() || "";
}

function OrderTrackingPanel({ fulfillmentState }: { fulfillmentState: string | undefined }) {
  const steps = fulfillmentTrackingSteps(fulfillmentState);
  return (
    <div className="shrink-0 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-4 shadow-[var(--shadow-subtle)] sm:min-w-[140px]">
      <Kicker tone="accent">Tracking</Kicker>
      <ul className="mt-4 space-y-0">
        {steps.map((s, i) => {
          const dot =
            s.step === "done"
              ? "bg-[var(--color-green)]"
              : s.step === "current"
                ? "bg-[var(--color-gold)]"
                : "bg-[var(--color-line-strong)]";
          const textMuted = s.step === "pending" ? "text-[var(--color-muted)]" : "text-[var(--color-ink)]";
          return (
            <li key={s.label} className="flex gap-3">
              <div className="flex flex-col items-center pt-0.5">
                <span className={`h-2.5 w-2.5 shrink-0 rounded-full ${dot}`} aria-hidden />
                {i < steps.length - 1 ? <span className="my-0.5 min-h-[14px] w-px flex-1 bg-[var(--color-line)]" aria-hidden /> : null}
              </div>
              <span className={`pb-3 text-sm font-medium leading-tight ${textMuted}`}>{s.label}</span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function FulfillmentTrackingPanel({
  fulfillmentState,
  trackingEventsJson,
  awbCode,
}: {
  fulfillmentState: string | undefined;
  trackingEventsJson?: string | null;
  awbCode?: string | null;
}) {
  const courierSteps = useMemo(() => parseShipmentTrackingJson(trackingEventsJson), [trackingEventsJson]);

  if (courierSteps.length > 0) {
    const delivered = courierSteps.some((x) => x.label.toLowerCase().includes("deliver"));
    return (
      <div className="shrink-0 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-4 shadow-[var(--shadow-subtle)] sm:min-w-[180px] sm:max-w-[220px]">
        <Kicker tone="accent">Delivery updates</Kicker>
        {awbCode?.trim() ? (
          <p className="mt-1.5 font-mono text-[11px] text-[var(--color-muted)]">AWB {awbCode.trim()}</p>
        ) : null}
        <ul className="mt-3 space-y-0" aria-label="Courier tracking timeline">
          {courierSteps.map((s, i) => {
            const isLast = i === courierSteps.length - 1;
            const step: TrackingStepState = delivered ? "done" : isLast ? "current" : "done";
            const dot =
              step === "done" ? "bg-[var(--color-green)]" : step === "current" ? "bg-[var(--color-gold)]" : "bg-[var(--color-line-strong)]";
            const textMuted = "text-[var(--color-ink)]";
            const sub = [formatCourierStepTime(s.at), s.location?.trim()].filter(Boolean).join(" Â· ");
            return (
              <li key={`${i}-${s.label}`} className="flex gap-3">
                <div className="flex flex-col items-center pt-0.5">
                  <span className={`h-2.5 w-2.5 shrink-0 rounded-full ${dot}`} aria-hidden />
                  {i < courierSteps.length - 1 ? (
                    <span className="my-0.5 min-h-[14px] w-px flex-1 bg-[var(--color-line)]" aria-hidden />
                  ) : null}
                </div>
                <div className={`min-w-0 pb-3 ${textMuted}`}>
                  <span className="block text-sm font-medium leading-tight">{s.label}</span>
                  {sub ? <span className="mt-0.5 block text-xs text-[var(--color-muted)]">{sub}</span> : null}
                </div>
              </li>
            );
          })}
        </ul>
      </div>
    );
  }

  return <OrderTrackingPanel fulfillmentState={fulfillmentState} />;
}

function RefundTrackingPanel({ state }: { state: RefundTrackingState }) {
  const label =
    state === "processed"
      ? "Refund completed"
      : state === "failed"
        ? "Refund failed"
        : "Refund in progress";
  const detail =
    state === "processed"
      ? "Money has been refunded to the original payment method."
      : state === "failed"
        ? "Refund failed at payment gateway. Please contact support."
        : "Refund has been requested and is being processed by Razorpay.";
  return (
    <div className="shrink-0 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-4 shadow-[var(--shadow-subtle)] sm:min-w-[180px] sm:max-w-[240px]">
      <Kicker tone="accent">Track refund</Kicker>
      <p className="mt-2 text-sm font-semibold text-[var(--color-ink)]">{label}</p>
      <p className="mt-1 text-xs leading-relaxed text-[var(--color-muted)]">{detail}</p>
    </div>
  );
}

function thumbnailUrlFromProductImages(
  images:
    | Array<{
        url?: string | null;
        thumbnailUrl?: string | null;
        thumbnail_url?: string | null;
      }>
    | undefined
): string | null {
  if (!images?.length) return null;
  const list = images.filter((i) => i?.url || i?.thumbnailUrl || i?.thumbnail_url);
  const img0 = list[0];
  const raw = img0?.thumbnailUrl ?? img0?.thumbnail_url ?? img0?.url;
  const s = typeof raw === "string" ? raw.trim() : "";
  return s || null;
}

/** Prefer thumbnail URL for list rows; matches cart/storefront image resolution. */
function firstOrderLineThumbnailUrl(detail: AccountOrderDetailPayload | undefined): string | null {
  const lines = detail?.order?.orderDetails;
  if (!lines?.length) return null;
  return thumbnailUrlFromProductImages(lines[0].productDetails?.[0]?.images);
}

function lineThumbnailUrl(line: AccountOrderDetailRow): string | null {
  return thumbnailUrlFromProductImages(line.productDetails?.[0]?.images);
}

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

/** Line-item row label: single line keeps short copy; multi-line orders show position (Option A). */
function orderLabelForLineItem(orderId: string, lineIndex: number, lineTotal: number): string {
  if (lineTotal > 1) {
    return `Order #${orderId} \u2022 Item ${lineIndex} of ${lineTotal}`;
  }
  return `Order #${orderId}`;
}

function singleOrderLineItemPresentation(
  order: AccountOrderRow,
  line: AccountOrderDetailRow,
  lineIndex: number,
  lineTotal: number
) {
  const dateStr = formatOrderDateShort(order.orderDate);
  const name = line.productDetails?.[0]?.name?.trim() || "Item";
  const formatted = line.productDetails?.[0]?.formatted?.trim();
  const qty = Number(line.quantity) || 1;
  const bits = [formatted, `Qty ${qty}`, `Ordered on ${dateStr}`].filter(Boolean);
  const price = line.priceFormatted || formatInrFromPaise(line.pricePaise);
  return {
    title: name,
    orderLabel: orderLabelForLineItem(order.orderId, lineIndex, lineTotal),
    detailLine: bits.join(" \u2022 "),
    price,
  };
}

/** Aggregate row while order lines are not loaded yet (or empty). */
function orderLinePresentation(order: AccountOrderRow, detail: AccountOrderDetailPayload | undefined) {
  const totalStr = order.totalAmountFormatted || formatInrFromPaise(order.totalAmountPaise);
  const dateStr = formatOrderDateShort(order.orderDate);
  const lines = detail?.order?.orderDetails;
  if (!lines?.length) {
    return {
      title: "Order items",
      orderLabel: `Order #${order.orderId}`,
      detailLine: `Ordered on ${dateStr}`,
      price: totalStr,
    };
  }
  const first = lines[0];
  const name = first.productDetails?.[0]?.name?.trim() || "Item";
  const title = lines.length > 1 ? `${name} + ${lines.length - 1} more` : name;
  const qtyTotal = lines.reduce((sum, row) => sum + (Number(row.quantity) || 0), 0) || Number(first.quantity) || 1;
  const formatted = first.productDetails?.[0]?.formatted?.trim();
  const bits = [formatted, `Qty ${qtyTotal}`, `Ordered on ${dateStr}`].filter(Boolean);
  const price = lines.length === 1 ? first.priceFormatted || formatInrFromPaise(first.pricePaise) : totalStr;
  return {
    title,
    orderLabel: `Order #${order.orderId}`,
    detailLine: bits.join(" \u2022 "),
    price,
  };
}

function sortOrdersLatestFirst(list: AccountOrderRow[]): AccountOrderRow[] {
  return [...list].sort((a, b) => {
    const tb = Date.parse(b.orderDate);
    const ta = Date.parse(a.orderDate);
    if (!Number.isNaN(ta) && !Number.isNaN(tb) && tb !== ta) return tb - ta;
    return Number(b.orderId) - Number(a.orderId);
  });
}

/** Hide cancel when status is clearly terminal or past fulfilment; server still enforces rules. */
function orderWithinCancelWindow(
  orderDateRaw: string,
  cancelWindowHours: number,
  cancelWindowEndsAt?: string | null
): boolean {
  const explicitDeadline = cancelWindowEndsAt ? Date.parse(cancelWindowEndsAt) : Number.NaN;
  const deadline = Number.isNaN(explicitDeadline)
    ? (() => {
        const createdAt = Date.parse(orderDateRaw);
        if (Number.isNaN(createdAt)) return Number.NaN;
        return createdAt + cancelWindowHours * 60 * 60 * 1000;
      })()
    : explicitDeadline;
  if (Number.isNaN(deadline)) return false;
  return Date.now() < deadline;
}

/** Hide cancel when status is clearly terminal or window has elapsed; server still enforces rules. */
function orderMayBeCancelledByCustomer(
  statusName: string | undefined,
  orderDate: string,
  cancelWindowHours: number,
  cancelWindowEndsAt?: string | null
): boolean {
  const s = (statusName ?? "").toLowerCase();
  if (s.includes("cancel") && !s.includes("partially_cancelled")) return false;
  if (s.includes("deliver")) return false;
  if (s.includes("ship")) return false;
  if (s.includes("transit")) return false;
  if (s.includes("refund")) return false;
  return orderWithinCancelWindow(orderDate, cancelWindowHours, cancelWindowEndsAt);
}

function normalizePaymentMethod(raw: string | null | undefined): string {
  return (raw ?? "prepaid").trim().toLowerCase();
}

function latestDeliveredAtForOrder(detail: AccountOrderDetailPayload | undefined): number {
  const deliveredAtValues = (detail?.shipments ?? [])
    .map((s) => Date.parse(s.deliveredAt ?? ""))
    .filter((v) => Number.isFinite(v));
  if (deliveredAtValues.length) return Math.max(...deliveredAtValues);
  const status = (detail?.statusName ?? "").toLowerCase();
  const fulfillment = (detail?.fulfillmentState ?? "").toLowerCase();
  if (status.includes("deliver") || fulfillment.includes("deliver")) {
    const fallback = Date.parse(detail?.order?.orderDate ?? "");
    if (Number.isFinite(fallback)) return fallback;
  }
  return Number.NaN;
}

function orderWithinReturnWindow(deliveredAtEpochMs: number, returnWindowDays: number): boolean {
  if (!Number.isFinite(deliveredAtEpochMs)) return false;
  const days = Number.isFinite(returnWindowDays) && returnWindowDays > 0 ? returnWindowDays : 7;
  const deadline = deliveredAtEpochMs + days * 24 * 60 * 60 * 1000;
  return Date.now() <= deadline;
}

function returnStatusLabel(rawStatus: string): string {
  const s = rawStatus.trim().toLowerCase();
  if (s === "requested") return "Return requested";
  if (s === "approved" || s === "in_transit") return "Return in progress";
  if (s === "received") return "Received at store";
  if (s === "refund_pending") return "Refund processing";
  if (s === "refunded") return "Refunded";
  if (s === "rejected") return "Return rejected";
  if (s === "cancelled") return "Return cancelled";
  return "Return in progress";
}

function lineReturnStatus(
  detail: AccountOrderDetailPayload | undefined,
  orderDetailId: string
): string | null {
  const statuses: string[] = [];
  for (const request of detail?.returnRequests ?? []) {
    for (const item of request.items ?? []) {
      if (item.orderDetailId === orderDetailId) {
        const status = (item.status || request.status || "").trim();
        if (status) statuses.push(status);
      }
    }
  }
  if (statuses.length === 0) return null;
  const priority = [
    "refunded",
    "refund_pending",
    "received",
    "in_transit",
    "approved",
    "requested",
    "rejected",
    "cancelled",
  ];
  for (const state of priority) {
    const found = statuses.find((s) => s.toLowerCase() === state);
    if (found) return found;
  }
  return statuses[0] ?? null;
}

function lineHasActiveReturn(detail: AccountOrderDetailPayload | undefined, orderDetailId: string): boolean {
  const status = lineReturnStatus(detail, orderDetailId);
  if (!status) return false;
  const s = status.toLowerCase();
  return s !== "rejected" && s !== "cancelled";
}

function UserIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      <path d="M20 21a8 8 0 0 0-16 0" />
      <circle cx="12" cy="8" r="4" />
    </svg>
  );
}

function BoxIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      <path d="M12 3 4.5 7.2 12 11.5l7.5-4.3L12 3Z" />
      <path d="M4.5 7.2V16.8L12 21l7.5-4.2V7.2" />
      <path d="M12 11.5V21" />
    </svg>
  );
}

function LocationIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      <path d="M12 21s7-4.35 7-11a7 7 0 1 0-14 0c0 6.65 7 11 7 11Z" />
      <circle cx="12" cy="10" r="2.5" />
    </svg>
  );
}

function ShieldIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      <path d="M12 3l7 3v6c0 5-3.5 8-7 9-3.5-1-7-4-7-9V6l7-3Z" />
      <path d="m9.5 12 1.7 1.7 3.8-3.8" />
    </svg>
  );
}

function HeadsetIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      <path d="M4 13a8 8 0 0 1 16 0" />
      <rect x="3" y="12" width="4" height="7" rx="2" />
      <rect x="17" y="12" width="4" height="7" rx="2" />
    </svg>
  );
}

type SidebarIconComponent = ComponentType<SVGProps<SVGSVGElement>>;

function ProfileSidebarItem({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: SidebarIconComponent;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      className={`flex w-full items-center gap-3 rounded-md border-l-2 px-4 py-3 text-left transition duration-200 ${
        active
          ? "border-[var(--color-gold)] bg-[var(--color-surface-soft)] text-[var(--color-green)]"
          : "border-transparent text-[var(--color-ink)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-green)]"
      }`}
    >
      <span
        className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full transition ${
          active ? "text-[var(--color-gold)]" : "text-[var(--color-muted)]"
        }`}
      >
        <Icon className="h-5 w-5" />
      </span>
      <span className="text-[12px] font-semibold uppercase leading-5 tracking-[0.2em]">{label}</span>
    </button>
  );
}

function AccountCard({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div
      className={cn(
        "rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)] sm:p-8",
        className
      )}
    >
      {children}
    </div>
  );
}

type ProfileAuthenticatedContentProps = {
  displayName: string;
  displayEmail: string;
  loginMethodLabel: string;
  error: string | null;
  loadingData: boolean;
  addresses: ShippingAddressRow[];
  orders: AccountOrderRow[];
  orderDetailsById: Record<string, AccountOrderDetailPayload>;
  form: AddressFormState;
  setForm: Dispatch<SetStateAction<AddressFormState>>;
  canSaveAddress: boolean;
  adding: boolean;
  addAddress: () => Promise<void>;
  updateAddress: (shippingAddressId: string) => Promise<void>;
  deleteAddress: (shippingAddressId: string) => Promise<void>;
  setDefaultAddress: (shippingAddressId: string) => Promise<void>;
  profileForm: ProfileFormState;
  setProfileForm: Dispatch<SetStateAction<ProfileFormState>>;
  canSaveProfile: boolean;
  savingProfile: boolean;
  updateProfile: () => Promise<void>;
  ensureOrderDetailLoaded: (orderId: string) => Promise<void>;
  refreshOrderDetail: (orderId: string) => Promise<void>;
  cancelOrder: (orderId: string) => Promise<void>;
  cancelOrderItems: (orderId: string, orderDetailIds: string[]) => Promise<void>;
  requestReturn: (orderId: string, orderDetailIds: string[], reason: string) => Promise<void>;
  onSignOut: () => void;
};

export function ProfileAuthenticatedContent({
  displayName,
  displayEmail,
  loginMethodLabel,
  error,
  loadingData,
  addresses,
  orders,
  orderDetailsById,
  form,
  setForm,
  canSaveAddress,
  adding,
  addAddress,
  updateAddress,
  deleteAddress,
  setDefaultAddress,
  profileForm,
  setProfileForm,
  canSaveProfile,
  savingProfile,
  updateProfile,
  ensureOrderDetailLoaded,
  refreshOrderDetail,
  cancelOrder,
  cancelOrderItems,
  requestReturn,
  onSignOut,
}: ProfileAuthenticatedContentProps) {
  const [activeNav, setActiveNav] = useState<ProfileNavId>("profile");
  const [cancellingOrderId, setCancellingOrderId] = useState<string | null>(null);
  const [cancellingItemKey, setCancellingItemKey] = useState<string | null>(null);
  const [cancelDialogOrderId, setCancelDialogOrderId] = useState<string | null>(null);
  const [cancelDialogItem, setCancelDialogItem] = useState<{ orderId: string; orderDetailId: string } | null>(null);
  const [returnSelectionByOrder, setReturnSelectionByOrder] = useState<Record<string, string[]>>({});
  const [returnReasonByOrder, setReturnReasonByOrder] = useState<Record<string, string>>({});
  const [requestingReturnOrderId, setRequestingReturnOrderId] = useState<string | null>(null);
  const [refreshingOrderId, setRefreshingOrderId] = useState<string | null>(null);
  const [editingAddressId, setEditingAddressId] = useState<string | null>(null);
  const [supportCategory, setSupportCategory] = useState<SupportCategory>("order");
  const [supportOrderId, setSupportOrderId] = useState<string>("");
  const [supportMessage, setSupportMessage] = useState<string>("");
  const [supportFiles, setSupportFiles] = useState<File[]>([]);

  useEffect(() => {
    if (activeNav !== "orders" || orders.length === 0) return;
    let cancelled = false;
    const queue = sortOrdersLatestFirst(orders).map((o) => o.orderId);
    const ORDER_DETAIL_FETCH_CONCURRENCY = 4;
    let next = 0;
    async function worker() {
      while (!cancelled && next < queue.length) {
        const orderId = queue[next++];
        await ensureOrderDetailLoaded(orderId);
      }
    }
    void Promise.all(
      Array.from({ length: Math.min(ORDER_DETAIL_FETCH_CONCURRENCY, queue.length) }, worker)
    );
    return () => {
      cancelled = true;
    };
  }, [activeNav, orders, ensureOrderDetailLoaded]);

  const sortedOrders = useMemo(() => sortOrdersLatestFirst(orders), [orders]);

  const orderListEntries = useMemo(() => {
    const out: Array<{
      key: string;
      order: AccountOrderRow;
      detail: AccountOrderDetailPayload | undefined;
      line: AccountOrderDetailRow | undefined;
      lineIndex: number;
      lineTotal: number;
    }> = [];
    for (const o of sortedOrders) {
      const detail = orderDetailsById[o.orderId];
      const lines = detail?.order?.orderDetails;
      if (lines && lines.length > 0) {
        const lineTotal = lines.length;
        lines.forEach((line, idx) => {
          out.push({
            key: `${o.orderId}-${line.orderDetailId}`,
            order: o,
            detail,
            line,
            lineIndex: idx + 1,
            lineTotal,
          });
        });
      } else {
        out.push({
          key: `${o.orderId}-pending`,
          order: o,
          detail,
          line: undefined,
          lineIndex: 0,
          lineTotal: 0,
        });
      }
    }
    return out;
  }, [sortedOrders, orderDetailsById]);

  const confirmCancelOrder = async () => {
    if (cancelDialogItem) {
      const { orderId, orderDetailId } = cancelDialogItem;
      const key = `${orderId}:${orderDetailId}`;
      setCancelDialogItem(null);
      setCancellingItemKey(key);
      try {
        await cancelOrderItems(orderId, [orderDetailId]);
      } finally {
        setCancellingItemKey(null);
      }
      return;
    }
    const id = cancelDialogOrderId;
    if (!id) return;
    setCancelDialogOrderId(null);
    setCancellingOrderId(id);
    try {
      await cancelOrder(id);
    } finally {
      setCancellingOrderId(null);
    }
  };

  const toggleReturnSelection = (orderId: string, orderDetailId: string) => {
    setReturnSelectionByOrder((prev) => {
      const next = { ...prev };
      const current = new Set(next[orderId] ?? []);
      if (current.has(orderDetailId)) current.delete(orderDetailId);
      else current.add(orderDetailId);
      next[orderId] = Array.from(current);
      return next;
    });
  };

  const submitReturnRequest = async (orderId: string) => {
    const selected = returnSelectionByOrder[orderId] ?? [];
    const reason = (returnReasonByOrder[orderId] ?? "").trim();
    if (!selected.length || !reason) return;
    setRequestingReturnOrderId(orderId);
    try {
      await requestReturn(orderId, selected, reason);
      setReturnSelectionByOrder((prev) => ({ ...prev, [orderId]: [] }));
      setReturnReasonByOrder((prev) => ({ ...prev, [orderId]: "" }));
    } finally {
      setRequestingReturnOrderId((prev) => (prev === orderId ? null : prev));
    }
  };

  const firstName = displayName.trim().split(/\s+/)[0] || displayName;
  const defaultAddress = addresses.find((a) => a.isDefault) ?? addresses[0];

  const supportIssues = useMemo(() => {
    return sortedOrders
      .map((o) => {
        const detail = orderDetailsById[o.orderId];
        const refundState = refundTrackingStateForOrder(o.statusName, detail);
        const statusName = (o.statusName ?? "").toLowerCase();
        let issueType: string | null = null;
        let issueState = "Open";
        if (refundState === "failed") {
          issueType = "Refund";
          issueState = "Needs action";
        } else if (refundState === "initiated") {
          issueType = "Refund";
          issueState = "In progress";
        } else if ((detail?.fulfillmentState ?? "").toLowerCase().includes("issue")) {
          issueType = "Shipping";
          issueState = "In progress";
        } else if (statusName.includes("cancel")) {
          issueType = "Order";
          issueState = refundState === "processed" ? "Resolved" : "In progress";
        }
        if (!issueType) return null;
        return {
          orderId: o.orderId,
          type: issueType,
          state: issueState,
          at: formatOrderDateShort(o.orderDate),
        };
      })
      .filter((v): v is { orderId: string; type: string; state: string; at: string } => Boolean(v))
      .slice(0, 5);
  }, [sortedOrders, orderDetailsById]);

  const supportPaymentSummary = useMemo(() => {
    let pending = 0;
    let paid = 0;
    let refunded = 0;
    let failed = 0;
    for (const o of sortedOrders) {
      const p = (orderDetailsById[o.orderId]?.paymentState ?? "unknown").toLowerCase();
      if (p.includes("refund")) refunded += 1;
      else if (p.includes("paid") || p.includes("captured")) paid += 1;
      else if (p.includes("fail")) failed += 1;
      else pending += 1;
    }
    return { pending, paid, refunded, failed };
  }, [sortedOrders, orderDetailsById]);

  const supportLatestShipment = useMemo(() => {
    for (const o of sortedOrders) {
      const ship = primaryShipmentForOrder(orderDetailsById[o.orderId]);
      if (ship) return { orderId: o.orderId, ship };
    }
    return null;
  }, [sortedOrders, orderDetailsById]);

  const supportEmailHref = useMemo(() => {
    const subjectOrder = supportOrderId.trim() ? `Order #${supportOrderId.trim()} - ` : "";
    const subject = `${subjectOrder}${supportCategory} support request`;
    const lines = [
      `Customer: ${displayName}`,
      `Email: ${displayEmail}`,
      `Category: ${supportCategory}`,
      supportOrderId.trim() ? `Order ID: ${supportOrderId.trim()}` : "",
      "",
      supportMessage.trim() || "(Please describe your issue)",
      "",
      supportFiles.length > 0
        ? `Attachments selected in portal: ${supportFiles.map((f) => f.name).join(", ")}`
        : "",
    ].filter(Boolean);
    return `mailto:sudattasdesignerboutique@gmail.com?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(lines.join("\n"))}`;
  }, [displayEmail, displayName, supportCategory, supportFiles, supportMessage, supportOrderId]);

  const navItems: { id: ProfileNavId; label: string; Icon: SidebarIconComponent }[] = [
    { id: "profile", label: "Profile", Icon: UserIcon },
    { id: "orders", label: "Orders", Icon: BoxIcon },
    { id: "addresses", label: "Addresses", Icon: LocationIcon },
    { id: "settings", label: "Account Settings", Icon: ShieldIcon },
    { id: "support", label: "Support", Icon: HeadsetIcon },
  ];

  const mainShell = (
    <div className="mx-auto w-full max-w-[1440px] px-[var(--gutter-mobile)] pb-16 pt-6 md:px-[var(--gutter-tablet)]">
      <div className="grid gap-8 lg:grid-cols-[260px_minmax(0,1fr)] lg:items-start">
        <aside className="lg:sticky lg:top-24">
          <div className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-6">
            <Kicker tone="inverse">Sudatta&apos;s</Kicker>
            <div className="mt-6 flex items-center gap-4">
              <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-full border border-[var(--color-gold)]/30 bg-white/5 text-[var(--color-gold-soft)] shadow-[0_0_0_4px_rgba(201,166,70,0.08)]">
                <UserIcon className="h-7 w-7" />
              </div>
              <div className="min-w-0">
                <p className="font-display text-[2rem] leading-none text-white">{firstName}</p>
              </div>
            </div>
          </div>

          <nav className="mt-6 flex flex-col space-y-1" aria-label="Account sections">
            {navItems.map((item) => (
              <ProfileSidebarItem
                key={item.id}
                icon={item.Icon}
                label={item.label}
                active={activeNav === item.id}
                onClick={() => setActiveNav(item.id)}
              />
            ))}
          </nav>

          <Button type="button" variant="outline" onClick={onSignOut} className="mt-6 w-full">
            Sign out
          </Button>
        </aside>

        <main className="min-w-0">
          {activeNav === "orders" ? (
            <div className="flex flex-col gap-6">
              <section className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-7">
                <HeroHeading inverse size="sm">Recent Orders</HeroHeading>
              </section>
              <div role="region" aria-label="Order list">
                {loadingData ? (
                  <p className="text-sm text-[var(--color-muted)]">Loading orders...</p>
                ) : orders.length === 0 ? (
                  <p className="text-sm text-[var(--color-muted)]">No orders yet.</p>
                ) : (
                  <div className="space-y-5 pb-2">
                    {orderListEntries.map((entry, index) => {
                      const { key, order: o, detail, line, lineIndex, lineTotal } = entry;
                      const cancelWindowHours = Math.max(1, Number(o.cancelWindowHours ?? 12));
                      const pres = line
                        ? singleOrderLineItemPresentation(o, line, lineIndex, lineTotal)
                        : orderLinePresentation(o, detail);
                      const thumbUrl = line ? lineThumbnailUrl(line) : firstOrderLineThumbnailUrl(detail);
                      const isFirstForOrder =
                        orderListEntries.findIndex((e) => e.order.orderId === o.orderId) === index;
                      const orderCanCancel = orderMayBeCancelledByCustomer(
                        o.statusName,
                        o.orderDate,
                        cancelWindowHours,
                        o.cancelWindowEndsAt
                      );
                      const activeLineCount = (detail?.order?.orderDetails ?? []).filter(
                        (row) => !((row.itemStatus ?? "").toLowerCase().includes("cancel"))
                      ).length;
                      const showFullOrderCancel = isFirstForOrder && orderCanCancel && activeLineCount > 1;
                      const showCancel =
                        !!line &&
                        !((line.itemStatus ?? "").toLowerCase().includes("cancel")) &&
                        orderCanCancel;
                      const itemCancelDisabled =
                        !showCancel ||
                        cancellingOrderId === o.orderId ||
                        cancellingItemKey === `${o.orderId}:${line?.orderDetailId ?? ""}`;
                      const orderCancelDisabled =
                        cancellingOrderId === o.orderId ||
                        cancellingItemKey?.startsWith(`${o.orderId}:`) === true;
                      const ship = primaryShipmentForOrder(detail);
                      const refundTrackingState = refundTrackingStateForOrder(o.statusName, detail);
                      const showRefundTracking = refundTrackingState !== "none";
                      const paymentMethod = normalizePaymentMethod(
                        o.paymentMethod ?? detail?.order?.paymentMethod
                      );
                      const isCodOrder = paymentMethod === "cod";
                      const deliveredAt = latestDeliveredAtForOrder(detail);
                      const deliveredForReturns = Number.isFinite(deliveredAt);
                      const returnWindowDays = Math.max(
                        1,
                        Number(
                          detail?.returnWindowDays ??
                            o.returnWindowDays ??
                            7
                        )
                      );
                      const withinReturnWindow = orderWithinReturnWindow(
                        deliveredAt,
                        returnWindowDays
                      );
                      const lineReturnRawStatus = line
                        ? lineReturnStatus(detail, line.orderDetailId)
                        : null;
                      const lineReturnLabel = lineReturnRawStatus
                        ? returnStatusLabel(lineReturnRawStatus)
                        : null;
                      const lineHasOpenReturnRequest = line
                        ? lineHasActiveReturn(detail, line.orderDetailId)
                        : false;
                      const showReturnWindowClosed =
                        !!line &&
                        !isCodOrder &&
                        deliveredForReturns &&
                        !withinReturnWindow &&
                        !lineHasOpenReturnRequest;
                      const lineEligibleForReturn =
                        !!line &&
                        !isCodOrder &&
                        deliveredForReturns &&
                        withinReturnWindow &&
                        !((line.itemStatus ?? "").toLowerCase().includes("cancel")) &&
                        !lineHasOpenReturnRequest;
                      const eligibleLineIdsForOrder = (detail?.order?.orderDetails ?? [])
                        .filter((row) => {
                          const rowHasReturn = lineHasActiveReturn(
                            detail,
                            row.orderDetailId
                          );
                          return (
                            !isCodOrder &&
                            deliveredForReturns &&
                            withinReturnWindow &&
                            !((row.itemStatus ?? "").toLowerCase().includes("cancel")) &&
                            !rowHasReturn
                          );
                        })
                        .map((row) => row.orderDetailId);
                      const selectedReturnIdsForOrder =
                        returnSelectionByOrder[o.orderId] ?? [];
                      const selectedReturnIdSet = new Set(selectedReturnIdsForOrder);
                      const returnReason = returnReasonByOrder[o.orderId] ?? "";
                      const canSubmitReturn =
                        selectedReturnIdsForOrder.length > 0 &&
                        returnReason.trim().length > 0 &&
                        requestingReturnOrderId !== o.orderId;
                      return (
                        <article
                          key={key}
                          className="group flex flex-col gap-6 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)] sm:flex-row sm:items-stretch sm:p-6"
                        >
                          <div className="relative h-28 w-full shrink-0 overflow-hidden rounded-md bg-[var(--color-surface-soft)] sm:h-auto sm:min-h-[7rem] sm:w-24">
                            {thumbUrl ? (
                              <Image
                                src={thumbUrl}
                                alt={pres.title}
                                fill
                                className="object-cover transition duration-500 ease-out group-hover:scale-[1.03]"
                                sizes="(max-width: 640px) 100vw, 96px"
                                unoptimized={isExternalProductImage(thumbUrl)}
                              />
                            ) : null}
                          </div>
                          <div className="min-w-0 flex-1">
                            <h2 className="font-display text-xl font-semibold text-[var(--color-ink)] sm:text-2xl">{pres.title}</h2>
                            <p className="mt-2 text-sm text-[var(--color-muted)]">{pres.orderLabel}</p>
                            <p className="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">{pres.detailLine}</p>
                            <p className="mt-3 font-sans text-xl font-semibold text-[var(--color-green)] md:text-2xl">{pres.price}</p>
                            {o.statusName ? (
                              <p className="mt-2 text-xs font-medium uppercase tracking-[0.12em] text-[var(--color-muted)]">
                                {customerOrderStatusHeadline(o.statusName, detail)}
                              </p>
                            ) : null}
                            {line && (line.itemStatus ?? "").toLowerCase().includes("cancel") ? (
                              <p className="mt-3 text-[11px] font-semibold uppercase tracking-[0.16em] text-[#A34A4A]">Cancelled</p>
                            ) : showCancel ? (
                              <button
                                type="button"
                                onClick={() =>
                                  setCancelDialogItem({
                                    orderId: o.orderId,
                                    orderDetailId: line.orderDetailId,
                                  })
                                }
                                disabled={itemCancelDisabled}
                                className="mt-3 rounded-full border border-[#A34A4A]/45 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#A34A4A] transition hover:bg-[#A34A4A]/10 disabled:cursor-not-allowed disabled:opacity-50"
                              >
                                {cancellingItemKey === `${o.orderId}:${line.orderDetailId}`
                                  ? "Cancelling..."
                                  : "Cancel item"}
                              </button>
                            ) : line ? (
                              <p className="mt-3 text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--color-muted)]">
                                Cancellation window closed. You can refuse delivery.
                              </p>
                            ) : null}
                            {line && lineEligibleForReturn ? (
                              <label className="mt-2 inline-flex cursor-pointer items-center gap-2 text-xs text-[var(--color-ink)]">
                                <input
                                  type="checkbox"
                                  checked={selectedReturnIdSet.has(line.orderDetailId)}
                                  onChange={() =>
                                    toggleReturnSelection(o.orderId, line.orderDetailId)
                                  }
                                  disabled={requestingReturnOrderId === o.orderId}
                                  aria-label={`Select line ${line.orderDetailId} for return`}
                                  className="h-4 w-4 rounded border-[var(--color-gold)]/40 text-[var(--color-ink)]"
                                />
                                Select for return
                              </label>
                            ) : null}
                            {lineReturnLabel ? (
                              <p className="mt-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--color-ink)]">
                                {lineReturnLabel}
                              </p>
                            ) : isCodOrder && line ? (
                              <p className="mt-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--color-muted)]">
                                Returns are available only for prepaid orders.
                              </p>
                            ) : showReturnWindowClosed ? (
                              <p className="mt-2 text-[11px] font-semibold uppercase tracking-[0.12em] text-[var(--color-muted)]">
                                Return window closed.
                              </p>
                            ) : null}
                            {showFullOrderCancel ? (
                              <button
                                type="button"
                                onClick={() => setCancelDialogOrderId(o.orderId)}
                                disabled={orderCancelDisabled}
                                className="mt-2 rounded-full border border-[#A34A4A]/45 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#A34A4A] transition hover:bg-[#A34A4A]/10 disabled:cursor-not-allowed disabled:opacity-50"
                              >
                                {cancellingOrderId === o.orderId ? "Cancelling..." : "Cancel full order"}
                              </button>
                            ) : null}
                            {isFirstForOrder && eligibleLineIdsForOrder.length > 0 ? (
                              <div className="mt-3 rounded-lg border border-[var(--color-gold)]/25 bg-[var(--color-surface-soft)] p-3">
                                <Kicker tone="accent">Request return</Kicker>
                                <textarea
                                  value={returnReason}
                                  onChange={(e) =>
                                    setReturnReasonByOrder((prev) => ({
                                      ...prev,
                                      [o.orderId]: e.target.value,
                                    }))
                                  }
                                  placeholder="Reason for return"
                                  rows={2}
                                  className="mt-2 w-full rounded-md border border-[var(--color-line)] bg-white px-3 py-2 text-sm text-[var(--color-ink)]"
                                />
                                <Button
                                  type="button"
                                  variant="outline"
                                  size="sm"
                                  onClick={() => {
                                    void submitReturnRequest(o.orderId);
                                  }}
                                  disabled={!canSubmitReturn}
                                  className="mt-2 rounded-full"
                                >
                                  {requestingReturnOrderId === o.orderId
                                    ? "Submitting..."
                                    : "Return selected items"}
                                </Button>
                              </div>
                            ) : null}
                            {isFirstForOrder && detail?.refundSummary ? (
                              <p className="mt-2 text-xs text-[var(--color-muted)]">
                                {refundTrackingState === "processed"
                                  ? "Refunded"
                                  : refundTrackingState === "failed"
                                    ? "Refund issue (expected)"
                                    : "Estimated refund"}: {detail.refundSummary.totalRefundFormatted}
                                {detail.refundSummary.shippingRefundMinor > 0
                                  ? ` (items ${detail.refundSummary.itemRefundFormatted} + shipping ${detail.refundSummary.shippingRefundFormatted})`
                                  : ` (items ${detail.refundSummary.itemRefundFormatted})`}
                              </p>
                            ) : null}
                          </div>
                          <div className="flex shrink-0 flex-col items-stretch gap-2 sm:min-w-[180px]">
                            {showRefundTracking ? (
                              <RefundTrackingPanel state={refundTrackingState} />
                            ) : (
                              <FulfillmentTrackingPanel
                                fulfillmentState={detail?.fulfillmentState}
                                trackingEventsJson={ship?.trackingEventsJson}
                                awbCode={ship?.awbCode}
                              />
                            )}
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                setRefreshingOrderId(o.orderId);
                                void refreshOrderDetail(o.orderId).finally(() => {
                                  setRefreshingOrderId((prev) => (prev === o.orderId ? null : prev));
                                });
                              }}
                              disabled={refreshingOrderId === o.orderId}
                              className="rounded-full"
                            >
                              {refreshingOrderId === o.orderId
                                ? "Refreshing..."
                                : showRefundTracking
                                  ? "Refresh refund"
                                  : "Refresh tracking"}
                            </Button>
                            {detail?.order?.invoiceAvailable ? (
                              <Button asChild variant="outline" size="sm" className="rounded-full">
                                <a href={`/api/account/orders/${encodeURIComponent(o.orderId)}/invoice`}>
                                  Download Invoice
                                </a>
                              </Button>
                            ) : null}
                          </div>
                        </article>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="flex flex-col gap-8">
        {activeNav === "profile" && (
          <>
            <section className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-7">
              <HeroHeading inverse size="sm">{displayName}</HeroHeading>
              <p className="mt-3 text-[var(--color-on-deep-muted)]">{displayEmail}</p>
            </section>

            <AccountCard className="mt-8">
              <Kicker>Edit Profile</Kicker>

              <div className="mt-6 grid gap-4 sm:grid-cols-2">
                <div>
                  <label htmlFor="profile-first-name" className="mb-1 block text-xs text-[var(--color-muted)]">
                    First name *
                  </label>
                  <Input
                    id="profile-first-name"
                    value={profileForm.firstName}
                    onChange={(e) => setProfileForm((p) => ({ ...p, firstName: e.target.value }))}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-last-name" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Last name
                  </label>
                  <Input
                    id="profile-last-name"
                    value={profileForm.lastName}
                    onChange={(e) => setProfileForm((p) => ({ ...p, lastName: e.target.value }))}
                    className="h-10"
                  />
                </div>

                <div>
                  <p className="mb-1 block text-xs text-[var(--color-muted)]">Gender</p>
                  <div className="flex h-10 items-center gap-5">
                    {(["male", "female", "other"] as const).map((option) => (
                      <label key={option} className="inline-flex cursor-pointer items-center gap-1.5 text-sm text-[var(--color-ink)]">
                        <input
                          type="radio"
                          name="profile-gender"
                          value={option}
                          checked={profileForm.gender === option}
                          onChange={() => setProfileForm((p) => ({ ...p, gender: option }))}
                          className="h-4 w-4 accent-[var(--color-green)]"
                        />
                        <span className="capitalize">{option}</span>
                      </label>
                    ))}
                  </div>
                </div>

                <div>
                  <label htmlFor="profile-dob" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Date of Birth
                  </label>
                  <Input
                    id="profile-dob"
                    type="date"
                    value={profileForm.dateOfBirth}
                    max={new Date().toISOString().slice(0, 10)}
                    onChange={(e) => setProfileForm((p) => ({ ...p, dateOfBirth: e.target.value }))}
                    className="h-10"
                  />
                </div>

                <div>
                  <label htmlFor="profile-mobile-number" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Mobile Number *
                  </label>
                  <Input
                    id="profile-mobile-number"
                    value={profileForm.phoneNumber}
                    onChange={(e) => setProfileForm((p) => ({ ...p, phoneNumber: e.target.value }))}
                    className="h-10"
                  />
                </div>

                <div>
                  <div className="mb-1 flex items-center justify-between">
                    <p className="text-xs text-[var(--color-muted)]">Default Address</p>
                    <button
                      type="button"
                      onClick={() => setActiveNav("addresses")}
                      className="text-xs font-semibold text-[var(--color-green)] underline-offset-2 hover:underline"
                    >
                      Change/Edit
                    </button>
                  </div>
                  <p className="min-h-10 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-3 py-2 text-sm leading-relaxed text-[var(--color-ink)]">
                    {defaultAddress ? formatAddress(defaultAddress) : "No default address saved yet."}
                  </p>
                </div>
              </div>

              <div className="mt-6 flex justify-center">
                <Button
                  type="button"
                  onClick={() => void updateProfile()}
                  disabled={!canSaveProfile || savingProfile}
                  className="px-10"
                >
                  {savingProfile ? "Saving..." : "Save"}
                </Button>
              </div>
            </AccountCard>
          </>
        )}

        {activeNav === "addresses" && (
          <div className="space-y-8">
            <section className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-7">
              <HeroHeading inverse size="sm">Saved Addresses</HeroHeading>
            </section>

            {error && (
              <p id="profile-form-error" role="alert" className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                {error}
              </p>
            )}

            {loadingData ? (
              <p className="text-sm text-[var(--color-muted)]">Loading addresses...</p>
            ) : addresses.length === 0 ? (
              <p className="text-sm text-[var(--color-muted)]">No addresses saved yet.</p>
            ) : (
              <ul className="grid gap-4 sm:grid-cols-2">
                {addresses.map((a) => (
                  <li
                    key={a.shippingAddressId}
                    className="flex flex-col gap-4 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-5 shadow-[var(--shadow-subtle)]"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex min-w-0 items-start gap-3">
                        <MapPin className="mt-0.5 h-4 w-4 shrink-0 text-[var(--color-gold)]" />
                        <div className="min-w-0">
                          {a.recipientName ? (
                            <p className="text-sm font-semibold text-[var(--color-ink)]">{a.recipientName}</p>
                          ) : null}
                          <p className="mt-0.5 text-sm leading-relaxed text-[var(--color-muted)]">
                            {formatAddressBody(a)}
                          </p>
                          {a.phoneNumber ? (
                            <p className="mt-1 text-xs text-[var(--color-muted)]">{a.phoneNumber}</p>
                          ) : null}
                        </div>
                      </div>
                      {a.isDefault ? (
                        <Kicker tone="accent" className="shrink-0">Default</Kicker>
                      ) : null}
                    </div>
                    <div className="flex flex-wrap items-center gap-2 border-t border-[var(--color-line)] pt-4">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="rounded-full"
                        onClick={() => {
                          setEditingAddressId(a.shippingAddressId);
                          setForm({
                            recipientName: a.recipientName ?? "",
                            phoneNumber: a.phoneNumber ?? "",
                            country: a.country ?? "",
                            stateRegion: a.stateRegion ?? "",
                            city: a.city ?? "",
                            postalCode: a.postalCode ?? "",
                            road: a.road ?? "",
                            apartmentNoOrName: a.apartmentNoOrName ?? "",
                          });
                        }}
                      >
                        Edit
                      </Button>
                      {!a.isDefault ? (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          className="rounded-full"
                          onClick={() => void setDefaultAddress(a.shippingAddressId)}
                        >
                          Make default
                        </Button>
                      ) : null}
                      <button
                        type="button"
                        onClick={() => void deleteAddress(a.shippingAddressId)}
                        className="ml-auto rounded-full border border-[#A34A4A]/45 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#A34A4A] transition hover:bg-[#A34A4A]/10"
                      >
                        Remove
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
            )}

            <AccountCard>
              <h3 className="font-display text-lg font-semibold text-[var(--color-ink)] sm:text-xl">
                {editingAddressId ? "Edit address" : "Add a new address"}
              </h3>

              <div className="mt-5 grid gap-x-6 gap-y-5 sm:grid-cols-2">
                <div>
                  <label htmlFor="profile-recipient-name" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Recipient name
                  </label>
                  <Input
                    id="profile-recipient-name"
                    value={form.recipientName}
                    onChange={(e) => setForm((p) => ({ ...p, recipientName: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-phone-number" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Phone number
                  </label>
                  <Input
                    id="profile-phone-number"
                    value={form.phoneNumber}
                    onChange={(e) => setForm((p) => ({ ...p, phoneNumber: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-road" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Road / street
                  </label>
                  <Input
                    id="profile-road"
                    value={form.road}
                    onChange={(e) => setForm((p) => ({ ...p, road: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-apartment" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Apartment / house (optional)
                  </label>
                  <Input
                    id="profile-apartment"
                    value={form.apartmentNoOrName}
                    onChange={(e) => setForm((p) => ({ ...p, apartmentNoOrName: e.target.value }))}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-city" className="mb-1 block text-xs text-[var(--color-muted)]">
                    City
                  </label>
                  <Input
                    id="profile-city"
                    value={form.city}
                    onChange={(e) => setForm((p) => ({ ...p, city: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-state" className="mb-1 block text-xs text-[var(--color-muted)]">
                    State / region
                  </label>
                  <Input
                    id="profile-state"
                    value={form.stateRegion}
                    onChange={(e) => setForm((p) => ({ ...p, stateRegion: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-country" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Country
                  </label>
                  <Input
                    id="profile-country"
                    value={form.country}
                    onChange={(e) => setForm((p) => ({ ...p, country: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
                <div>
                  <label htmlFor="profile-pincode" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Pincode
                  </label>
                  <Input
                    id="profile-pincode"
                    value={form.postalCode}
                    onChange={(e) => setForm((p) => ({ ...p, postalCode: e.target.value.replace(/\D/g, "").slice(0, 6) }))}
                    inputMode="numeric"
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10"
                  />
                </div>
              </div>

              <div className="mt-5 flex flex-wrap items-center gap-3">
                <Button
                  type="button"
                  onClick={() => {
                    if (editingAddressId) {
                      void updateAddress(editingAddressId).then(() => setEditingAddressId(null));
                      return;
                    }
                    void addAddress();
                  }}
                  disabled={!canSaveAddress || adding}
                >
                  {adding ? "Saving..." : editingAddressId ? "Update Address" : "Save Address"}
                </Button>
                {editingAddressId ? (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setEditingAddressId(null);
                      setForm({
                        recipientName: "",
                        phoneNumber: "",
                        country: "India",
                        stateRegion: "",
                        city: "",
                        postalCode: "",
                        road: "",
                        apartmentNoOrName: "",
                      });
                    }}
                  >
                    Cancel Edit
                  </Button>
                ) : null}
              </div>
            </AccountCard>
          </div>
        )}

        {activeNav === "settings" && (
          <div className="space-y-8">
            <section className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-7">
              <HeroHeading inverse size="sm">Account Settings</HeroHeading>
            </section>
            <AccountCard>
              <p className="text-sm text-[var(--color-muted)]">
                Signed in as <span className="font-medium text-[var(--color-ink)]">{displayEmail}</span> via {loginMethodLabel}.
              </p>
              <Button type="button" onClick={onSignOut} className="mt-6">
                Sign out everywhere on this device
              </Button>
            </AccountCard>
          </div>
        )}

        {activeNav === "support" && (
          <div className="space-y-8">
            <section className="bg-deep-feature flex min-h-[196px] flex-col justify-center rounded-lg p-7">
              <HeroHeading inverse size="sm">Support</HeroHeading>
            </section>
            <div className="grid gap-6 lg:grid-cols-2">
              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>Order help</Kicker>
                <div className="mt-4 flex flex-wrap gap-2">
                  <Button type="button" variant="outline" size="sm" onClick={() => setActiveNav("orders")} className="rounded-full">
                    Track shipment
                  </Button>
                  <Button type="button" variant="outline" size="sm" onClick={() => setActiveNav("orders")} className="rounded-full">
                    Track refund
                  </Button>
                  <Button asChild variant="outline" size="sm" className="rounded-full">
                    <Link href="/returns-exchanges">Return policy</Link>
                  </Button>
                </div>
                <p className="mt-4 text-xs leading-6 text-[var(--color-muted)]">
                  Cancellation is available only within the configured cancellation window after order creation. After the window closes, you can refuse delivery and support will assist with return/refund updates.
                </p>
              </section>

              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>Contact options</Kicker>
                <ul className="mt-3 space-y-2 text-sm text-[var(--color-muted)]">
                  <li>
                    Email:{" "}
                    <a href="mailto:sudattasdesignerboutique@gmail.com" className="font-semibold text-[var(--color-ink)] underline-offset-2 hover:underline">
                      sudattasdesignerboutique@gmail.com
                    </a>
                  </li>
                  <li>
                    Phone/WhatsApp:{" "}
                    <a href="tel:+919073764577" className="font-semibold text-[var(--color-ink)] underline-offset-2 hover:underline">
                      +91 90737 64577
                    </a>
                  </li>
                  <li>Hours: Mon-Sat, 10:00 AM - 7:00 PM IST</li>
                  <li>Typical response: within 24 hours</li>
                </ul>
              </section>
            </div>

            <div className="grid gap-6 lg:grid-cols-2">
              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>My recent issues</Kicker>
                {supportIssues.length === 0 ? (
                  <p className="mt-3 text-sm text-[var(--color-muted)]">No active support issues right now.</p>
                ) : (
                  <ul className="mt-3 space-y-2">
                    {supportIssues.map((i) => (
                      <li key={`${i.orderId}-${i.type}`} className="flex items-center justify-between rounded-xl border border-[var(--color-line)] bg-white px-3 py-2">
                        <span className="text-sm text-[var(--color-ink)]">
                          {i.type} issue - Order #{i.orderId}
                        </span>
                        <span className="text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-muted)]">{i.state}</span>
                      </li>
                    ))}
                  </ul>
                )}
              </section>

              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>Shipping info</Kicker>
                {supportLatestShipment ? (
                  <div className="mt-3 space-y-2 text-sm text-[var(--color-muted)]">
                    <p>
                      Latest shipment order: <span className="font-semibold text-[var(--color-ink)]">#{supportLatestShipment.orderId}</span>
                    </p>
                    <p>
                      Courier:{" "}
                      <span className="font-semibold text-[var(--color-ink)]">{supportLatestShipment.ship.carrier || "Pending assignment"}</span>
                    </p>
                    <p>
                      AWB: <span className="font-mono text-[var(--color-ink)]">{supportLatestShipment.ship.awbCode || "Not assigned"}</span>
                    </p>
                    <p>
                      Status:{" "}
                      <span className="font-semibold text-[var(--color-ink)]">
                        {supportLatestShipment.ship.shiprocketStatusLabel || supportLatestShipment.ship.status}
                      </span>
                    </p>
                    <p>Updated: {formatOrderDateShort(supportLatestShipment.ship.createdAt)}</p>
                  </div>
                ) : (
                  <p className="mt-3 text-sm text-[var(--color-muted)]">No shipment records yet.</p>
                )}
              </section>
            </div>

            <div className="grid gap-6 lg:grid-cols-2">
              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>Payment and refund info</Kicker>
                <div className="mt-3 grid grid-cols-2 gap-2 text-sm">
                  <div className="rounded-xl border border-[var(--color-line)] bg-white px-3 py-2 text-[var(--color-muted)]">
                    Paid: <span className="font-semibold text-[var(--color-ink)]">{supportPaymentSummary.paid}</span>
                  </div>
                  <div className="rounded-xl border border-[var(--color-line)] bg-white px-3 py-2 text-[var(--color-muted)]">
                    Pending: <span className="font-semibold text-[var(--color-ink)]">{supportPaymentSummary.pending}</span>
                  </div>
                  <div className="rounded-xl border border-[var(--color-line)] bg-white px-3 py-2 text-[var(--color-muted)]">
                    Refunded: <span className="font-semibold text-[var(--color-ink)]">{supportPaymentSummary.refunded}</span>
                  </div>
                  <div className="rounded-xl border border-[var(--color-line)] bg-white px-3 py-2 text-[var(--color-muted)]">
                    Failed: <span className="font-semibold text-[var(--color-ink)]">{supportPaymentSummary.failed}</span>
                  </div>
                </div>
                <p className="mt-3 text-xs leading-6 text-[var(--color-muted)]">Razorpay reference IDs and refund state are visible in your order details timeline.</p>
              </section>

              <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-subtle)]">
                <Kicker>Raise a request</Kicker>
                <div className="mt-3 grid gap-2 sm:grid-cols-2">
                  <select
                    value={supportCategory}
                    onChange={(e) => setSupportCategory(e.target.value as SupportCategory)}
                    className="h-10 rounded-md border border-[var(--color-line)] bg-white px-3 text-sm text-[var(--color-ink)]"
                  >
                    <option value="order">Order</option>
                    <option value="payment">Payment</option>
                    <option value="refund">Refund</option>
                    <option value="shipping">Shipping</option>
                    <option value="account">Account</option>
                  </select>
                  <select
                    value={supportOrderId}
                    onChange={(e) => setSupportOrderId(e.target.value)}
                    className="h-10 rounded-md border border-[var(--color-line)] bg-white px-3 text-sm text-[var(--color-ink)]"
                  >
                    <option value="">Select order (optional)</option>
                    {sortedOrders.map((o) => (
                      <option key={o.orderId} value={o.orderId}>
                        Order #{o.orderId}
                      </option>
                    ))}
                  </select>
                </div>
                <textarea
                  value={supportMessage}
                  onChange={(e) => setSupportMessage(e.target.value)}
                  placeholder="Tell us what happened..."
                  className="mt-2 min-h-[92px] w-full rounded-md border border-[var(--color-line)] bg-white px-3 py-2 text-sm text-[var(--color-ink)]"
                />
                <input
                  type="file"
                  multiple
                  accept="image/*"
                  onChange={(e) => setSupportFiles(Array.from(e.target.files ?? []))}
                  className="mt-2 block w-full text-xs text-[var(--color-muted)]"
                />
                {supportFiles.length > 0 ? (
                  <p className="mt-1 text-xs text-[var(--color-muted)]">{supportFiles.length} file(s) selected</p>
                ) : null}
                <Button
                  asChild
                  className="mt-3 rounded-full border-[var(--color-gold)] bg-[var(--color-gold)] text-[var(--color-deep)] hover:border-[var(--color-gold-soft)] hover:bg-[var(--color-gold-soft)]"
                >
                  <a href={supportEmailHref}>Create support email draft</a>
                </Button>
              </section>
            </div>

            <section className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-6 text-sm text-[var(--color-muted)] shadow-[var(--shadow-subtle)]">
              <Kicker>FAQ</Kicker>
              <ul className="mt-3 space-y-2">
                <li>Where is my order? Use Orders &gt; Refresh tracking for latest courier scan.</li>
                <li>When will refund arrive? Usually 3-7 business days after refund is processed.</li>
                <li>How do I change address? Update default address from Addresses tab before checkout.</li>
                <li>How to contact quickly? Use WhatsApp/phone during support hours for urgent issues.</li>
              </ul>
              {defaultAddress ? (
                <p className="mt-3 text-xs text-[var(--color-muted)]">Default delivery address on file: {formatAddress(defaultAddress)}</p>
              ) : null}
            </section>
          </div>
        )}
            </div>
          )}
        </main>
      </div>
    </div>
  );

  return (
    <section>
      {mainShell}
      <Dialog
        open={!!cancelDialogOrderId || !!cancelDialogItem}
        onOpenChange={(open) => {
          if (!open) {
            setCancelDialogOrderId(null);
            setCancelDialogItem(null);
          }
        }}
      >
        <DialogContent
          title=""
          className="max-w-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-soft)] [&>div:first-of-type]:border-0 [&>div:first-of-type]:p-4 [&>div:first-of-type]:pb-0"
          contentClassName="space-y-6 px-6 pb-8 pt-2 sm:px-8"
        >
          <h2 className="text-center font-display text-2xl font-semibold leading-tight tracking-tight text-[var(--color-ink)] sm:text-[1.65rem]">
            Wait! We&apos;re sad to see you go.
          </h2>
          <p className="text-center text-sm leading-[1.7] text-[var(--color-muted)] sm:text-[0.9375rem]">
            {cancelDialogItem
              ? "Only this item will be cancelled. Shipping will not be refunded unless all items in this order are cancelled."
              : `Each piece at Sudatta's is carefully prepared to ensure it reaches you in perfect condition. If there's a specific reason for your cancellation\u2014like a change in size or a delivery timing concern\u2014please let us know. We'd love the chance to make it right before we halt the process.`}
          </p>
          <div className="flex flex-col gap-3 sm:flex-row sm:gap-4">
            <Button
              type="button"
              className="flex-1"
              onClick={() => setCancelDialogOrderId(null)}
            >
              <Check className="h-5 w-5 shrink-0" strokeWidth={2.25} aria-hidden />
              Keep My Selection
            </Button>
            <Button
              type="button"
              variant="outline"
              className="flex-1"
              onClick={() => void confirmCancelOrder()}
              disabled={!cancelDialogOrderId && !cancelDialogItem}
            >
              <X className="h-5 w-5 shrink-0 opacity-80" strokeWidth={2} aria-hidden />
              Continue with Cancellation
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
