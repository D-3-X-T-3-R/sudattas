/* eslint-disable max-lines */
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
import type { AddressFormState } from "@/domains/profile/types";
import { formatInrFromPaise } from "@/lib/money";
import { Check, X } from "lucide-react";
import { Dialog, DialogContent } from "@/components/ui/dialog";

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
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  statusName: string;
};

export type AccountProfileRow = {
  userId: string;
  email: string;
  fullName?: string | null;
  address?: string | null;
  phone?: string | null;
  createDate: string;
};

export type AccountOrderDetailRow = {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  pricePaise: string;
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
  if (sn.includes("cancel_pending")) return "Cancellation in progress · awaiting courier";
  const paid =
    (detail?.paymentState ?? "").toLowerCase().includes("paid") ||
    (detail?.paymentState ?? "").toLowerCase().includes("captured");
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
    <div className="shrink-0 rounded-2xl border border-[#C9A646]/35 bg-white px-4 py-4 shadow-[0_8px_24px_rgba(15,61,46,0.06)] sm:min-w-[140px]">
      <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#C9A646]">Tracking</p>
      <ul className="mt-4 space-y-0">
        {steps.map((s, i) => {
          const dot =
            s.step === "done"
              ? "bg-[#0F3D2E]"
              : s.step === "current"
                ? "bg-[#C9A646]"
                : "bg-[#D4D0C8]";
          const textMuted = s.step === "pending" ? "text-[#A8A29A]" : "text-[#0F3D2E]";
          return (
            <li key={s.label} className="flex gap-3">
              <div className="flex flex-col items-center pt-0.5">
                <span className={`h-2.5 w-2.5 shrink-0 rounded-full ${dot}`} aria-hidden />
                {i < steps.length - 1 ? <span className="my-0.5 min-h-[14px] w-px flex-1 bg-[#E0DCD4]" aria-hidden /> : null}
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
      <div className="shrink-0 rounded-2xl border border-[#C9A646]/35 bg-white px-4 py-4 shadow-[0_8px_24px_rgba(15,61,46,0.06)] sm:min-w-[180px] sm:max-w-[220px]">
        <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#C9A646]">Delivery updates</p>
        {awbCode?.trim() ? (
          <p className="mt-1.5 font-mono text-[11px] text-[#615A50]">AWB {awbCode.trim()}</p>
        ) : null}
        <ul className="mt-3 space-y-0" aria-label="Courier tracking timeline">
          {courierSteps.map((s, i) => {
            const isLast = i === courierSteps.length - 1;
            const step: TrackingStepState = delivered ? "done" : isLast ? "current" : "done";
            const dot =
              step === "done" ? "bg-[#0F3D2E]" : step === "current" ? "bg-[#C9A646]" : "bg-[#D4D0C8]";
            const textMuted = "text-[#0F3D2E]";
            const sub = [formatCourierStepTime(s.at), s.location?.trim()].filter(Boolean).join(" Â· ");
            return (
              <li key={`${i}-${s.label}`} className="flex gap-3">
                <div className="flex flex-col items-center pt-0.5">
                  <span className={`h-2.5 w-2.5 shrink-0 rounded-full ${dot}`} aria-hidden />
                  {i < courierSteps.length - 1 ? (
                    <span className="my-0.5 min-h-[14px] w-px flex-1 bg-[#E0DCD4]" aria-hidden />
                  ) : null}
                </div>
                <div className={`min-w-0 pb-3 ${textMuted}`}>
                  <span className="block text-sm font-medium leading-tight">{s.label}</span>
                  {sub ? <span className="mt-0.5 block text-xs text-[#8B816D]">{sub}</span> : null}
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
    <div className="shrink-0 rounded-2xl border border-[#C9A646]/35 bg-white px-4 py-4 shadow-[0_8px_24px_rgba(15,61,46,0.06)] sm:min-w-[180px] sm:max-w-[240px]">
      <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#C9A646]">Track refund</p>
      <p className="mt-2 text-sm font-semibold text-[#0F3D2E]">{label}</p>
      <p className="mt-1 text-xs leading-relaxed text-[#615A50]">{detail}</p>
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
function orderMayBeCancelledByCustomer(statusName: string | undefined): boolean {
  const s = (statusName ?? "").toLowerCase();
  if (s.includes("cancel")) return false;
  if (s.includes("deliver")) return false;
  if (s.includes("ship")) return false;
  if (s.includes("transit")) return false;
  if (s.includes("refund")) return false;
  return true;
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
      className={`flex w-full items-center gap-4 rounded-[22px] px-5 py-4 text-left transition duration-300 ${
        active
          ? "bg-[linear-gradient(135deg,rgba(255,255,255,0.12),rgba(255,255,255,0.08))] text-white shadow-[0_14px_30px_rgba(0,0,0,0.16)]"
          : "text-white/82 hover:bg-white/5 hover:text-white"
      }`}
    >
      <span
        className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-full border transition ${
          active
            ? "border-[#C9A646]/18 bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.18),rgba(255,255,255,0.05))] text-[#E7CF82]"
            : "border-white/6 bg-white/8 text-white/86"
        }`}
      >
        <Icon className="h-5 w-5" />
      </span>
      <span className="text-[12px] font-semibold uppercase leading-5 tracking-[0.24em]">{label}</span>
    </button>
  );
}

function SectionBadge({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex rounded-full border border-[#C9A646]/30 bg-[#FFF9EF] px-4 py-1.5 text-[11px] font-semibold uppercase tracking-[0.28em] text-[#A37D34]">
      {children}
    </span>
  );
}

type ProfileAuthenticatedContentProps = {
  displayName: string;
  displayEmail: string;
  loginMethodLabel: string;
  accountProfile: AccountProfileRow | null;
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
  ensureOrderDetailLoaded: (orderId: string) => Promise<void>;
  refreshOrderDetail: (orderId: string) => Promise<void>;
  cancelOrder: (orderId: string) => Promise<void>;
  onSignOut: () => void;
};

function PillButton({
  children,
  onClick,
  type = "button",
}: {
  children: ReactNode;
  onClick?: () => void;
  type?: "button" | "submit";
}) {
  return (
    <button
      type={type}
      onClick={onClick}
      className="rounded-full border border-[#C9A646]/30 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-[#A37D34] transition hover:bg-[#fff7e6]"
    >
      {children}
    </button>
  );
}

function DashboardCard({
  label,
  value,
  action,
}: {
  label: string;
  value: ReactNode;
  action: ReactNode;
}) {
  return (
    <div className="flex flex-col rounded-[24px] border border-[#0F3D2E]/8 bg-[#FAF6EE] p-5 shadow-[0_12px_28px_rgba(15,61,46,0.05)]">
      <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[#8B816D]">{label}</p>
      <p className="mt-3 text-lg font-medium text-[#0F3D2E]">{value}</p>
      <div className="mt-4">{action}</div>
    </div>
  );
}

// eslint-disable-next-line max-lines-per-function
export function ProfileAuthenticatedContent({
  displayName,
  displayEmail,
  loginMethodLabel,
  accountProfile,
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
  ensureOrderDetailLoaded,
  refreshOrderDetail,
  cancelOrder,
  onSignOut,
}: ProfileAuthenticatedContentProps) {
  const [activeNav, setActiveNav] = useState<ProfileNavId>("profile");
  const [emailHint, setEmailHint] = useState(false);
  const [loginHint, setLoginHint] = useState(false);
  const [cancellingOrderId, setCancellingOrderId] = useState<string | null>(null);
  const [cancelDialogOrderId, setCancelDialogOrderId] = useState<string | null>(null);
  const [refreshingOrderId, setRefreshingOrderId] = useState<string | null>(null);
  const [editingAddressId, setEditingAddressId] = useState<string | null>(null);
  const [supportCategory, setSupportCategory] = useState<SupportCategory>("order");
  const [supportOrderId, setSupportOrderId] = useState<string>("");
  const [supportMessage, setSupportMessage] = useState<string>("");
  const [supportFiles, setSupportFiles] = useState<File[]>([]);

  useEffect(() => {
    if (activeNav !== "orders" || orders.length === 0) return;
    void Promise.all(orders.map((o) => ensureOrderDetailLoaded(o.orderId)));
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

  useEffect(() => {
    if (activeNav !== "orders") return;
    const summary = sortedOrders.map((o) => {
      const detail = orderDetailsById[o.orderId];
      const refundState = refundTrackingStateForOrder(o.statusName, detail);
      return {
        orderId: o.orderId,
        statusName: o.statusName,
        fulfillmentState: detail?.fulfillmentState ?? null,
        paymentState: detail?.paymentState ?? null,
        refundTrackingState: refundState,
      };
    });
    console.info("[orders-flow][customer-ui] orders tab rendered", {
      totalOrders: sortedOrders.length,
      summary,
    });
  }, [activeNav, sortedOrders, orderDetailsById]);

  const confirmCancelOrder = async () => {
    const id = cancelDialogOrderId;
    if (!id) return;
    console.info("[orders-flow][customer-ui] cancel dialog confirmed", { orderId: id });
    setCancelDialogOrderId(null);
    setCancellingOrderId(id);
    try {
      await cancelOrder(id);
    } finally {
      setCancellingOrderId(null);
    }
  };

  const firstName = displayName.trim().split(/\s+/)[0] || displayName;
  const phoneDisplay = accountProfile?.phone?.trim() ? accountProfile.phone.trim() : "Not available";
  const phoneMissing = !accountProfile?.phone?.trim();
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
    return `mailto:support@sudattas.com?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(lines.join("\n"))}`;
  }, [displayEmail, displayName, supportCategory, supportFiles, supportMessage, supportOrderId]);

  const navItems: { id: ProfileNavId; label: string; Icon: SidebarIconComponent }[] = [
    { id: "profile", label: "Profile", Icon: UserIcon },
    { id: "orders", label: "Orders", Icon: BoxIcon },
    { id: "addresses", label: "Addresses", Icon: LocationIcon },
    { id: "settings", label: "Account Settings", Icon: ShieldIcon },
    { id: "support", label: "Support", Icon: HeadsetIcon },
  ];

  const mainShell = (
    <div className="mx-auto max-w-7xl rounded-[36px] border border-white/70 bg-[#FBF8F1] p-4 shadow-[0_30px_90px_rgba(15,61,46,0.08)] sm:p-6 lg:p-8">
      <div className="grid min-h-0 gap-6 lg:h-[min(720px,calc(100vh-10.5rem))] lg:max-h-[min(720px,calc(100vh-10.5rem))] lg:grid-cols-[300px_minmax(0,1fr)]">
        <aside className="flex min-h-0 flex-col overflow-hidden rounded-[30px] bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.14),transparent_24%),linear-gradient(145deg,#1E5B43_0%,#0F3D2E_32%,#0A2C22_72%,#083126_100%)] p-5 text-white shadow-[0_24px_52px_rgba(15,61,46,0.22)] lg:h-full lg:max-h-full">
          <div className="rounded-[26px] border border-white/8 bg-[linear-gradient(135deg,rgba(255,255,255,0.08),rgba(255,255,255,0.04))] p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
            <p className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[#E7CF82]">Sudatta&apos;s</p>
            <div className="mt-5 flex items-center gap-4">
              <div className="flex h-14 w-14 items-center justify-center rounded-full border border-white/10 bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.20),rgba(255,255,255,0.06))] text-[#E7CF82] shadow-[0_10px_24px_rgba(0,0,0,0.14)]">
                <UserIcon className="h-6 w-6" />
              </div>
              <div className="min-w-0">
                <p className="font-[family-name:var(--font-display)] text-[2rem] leading-none text-white">{firstName}</p>
              </div>
            </div>
          </div>

          <nav className="mt-6 flex min-h-0 flex-1 flex-col space-y-2.5 overflow-y-auto overscroll-contain" aria-label="Account sections">
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

          <button
            type="button"
            onClick={onSignOut}
            className="mt-8 w-full rounded-[22px] border border-white/15 bg-[linear-gradient(135deg,rgba(255,255,255,0.1),rgba(255,255,255,0.04))] py-3.5 text-center text-[11px] font-semibold uppercase tracking-[0.2em] text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] transition duration-300 hover:border-[#C9A646]/35 hover:bg-white/10 hover:text-[#E7CF82]"
          >
            Sign out
          </button>
        </aside>

        <main className="flex min-h-0 min-w-0 flex-col overflow-hidden rounded-[30px] border border-[#0F3D2E]/8 bg-[linear-gradient(180deg,#FFFDF9_0%,#FAF6EF_100%)] shadow-[0_18px_42px_rgba(15,61,46,0.06)] lg:h-full lg:max-h-full">
          {activeNav === "orders" ? (
            <div className="flex min-h-0 flex-1 flex-col overflow-hidden pt-5 px-6 pb-6 sm:px-8 sm:pb-8">
              <section className="shrink-0 rounded-[28px] bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.12),transparent_30%),linear-gradient(135deg,#0F3D2E,#0A2C22)] p-7 text-white shadow-[0_20px_45px_rgba(15,61,46,0.18)]">
                <h1 className="font-[family-name:var(--font-display)] text-4xl sm:text-5xl">Recent Orders</h1>
              </section>
              <div
                className="mt-6 min-h-0 flex-1 overflow-y-auto overscroll-y-contain [-webkit-overflow-scrolling:touch] [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden"
                role="region"
                aria-label="Order list"
              >
                {loadingData ? (
                  <p className="text-sm text-[#615A50]">Loading orders...</p>
                ) : orders.length === 0 ? (
                  <p className="text-sm text-[#615A50]">No orders yet.</p>
                ) : (
                  <div className="space-y-5 pb-2">
                    {orderListEntries.map((entry, index) => {
                      const { key, order: o, detail, line, lineIndex, lineTotal } = entry;
                      const pres = line
                        ? singleOrderLineItemPresentation(o, line, lineIndex, lineTotal)
                        : orderLinePresentation(o, detail);
                      const thumbUrl = line ? lineThumbnailUrl(line) : firstOrderLineThumbnailUrl(detail);
                      const isFirstForOrder =
                        orderListEntries.findIndex((e) => e.order.orderId === o.orderId) === index;
                      const showCancel =
                        isFirstForOrder && orderMayBeCancelledByCustomer(o.statusName);
                      const ship = primaryShipmentForOrder(detail);
                      const refundTrackingState = refundTrackingStateForOrder(o.statusName, detail);
                      const showRefundTracking = refundTrackingState !== "none";
                      return (
                        <article
                          key={key}
                          className="flex flex-col gap-6 rounded-[24px] border border-[#0F3D2E]/10 bg-[#FAF6EE] p-5 shadow-[0_12px_28px_rgba(15,61,46,0.06)] sm:flex-row sm:items-stretch sm:p-6"
                        >
                          <div className="relative h-28 w-full shrink-0 overflow-hidden rounded-2xl bg-[#E8DCC8] sm:h-auto sm:min-h-[7rem] sm:w-24">
                            {thumbUrl ? (
                              <Image
                                src={thumbUrl}
                                alt={pres.title}
                                fill
                                className="object-cover"
                                sizes="(max-width: 640px) 100vw, 96px"
                                unoptimized={isExternalProductImage(thumbUrl)}
                              />
                            ) : null}
                          </div>
                          <div className="min-w-0 flex-1">
                            <h2 className="font-[family-name:var(--font-display)] text-xl font-semibold text-[#0F3D2E] sm:text-2xl">{pres.title}</h2>
                            <p className="mt-2 text-sm text-[#6B6560]">{pres.orderLabel}</p>
                            <p className="mt-2 text-sm leading-relaxed text-[#615A50]">{pres.detailLine}</p>
                            <p className="mt-3 text-lg font-semibold text-[#0F3D2E]">{pres.price}</p>
                            {o.statusName ? (
                              <p className="mt-2 text-xs font-medium uppercase tracking-[0.12em] text-[#8B816D]">
                                {customerOrderStatusHeadline(o.statusName, detail)}
                              </p>
                            ) : null}
                            {showCancel ? (
                              <button
                                type="button"
                                onClick={() => setCancelDialogOrderId(o.orderId)}
                                disabled={cancellingOrderId === o.orderId}
                                className="mt-3 rounded-full border border-[#C45C5C]/45 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#A34A4A] transition hover:bg-[#fff5f5] disabled:cursor-not-allowed disabled:opacity-50"
                              >
                                {cancellingOrderId === o.orderId ? "Cancelling..." : "Cancel order"}
                              </button>
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
                            <button
                              type="button"
                              onClick={() => {
                                setRefreshingOrderId(o.orderId);
                                void refreshOrderDetail(o.orderId).finally(() => {
                                  setRefreshingOrderId((prev) => (prev === o.orderId ? null : prev));
                                });
                              }}
                              disabled={refreshingOrderId === o.orderId}
                              className="rounded-full border border-[#C9A646]/35 px-4 py-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-[#A37D34] transition hover:bg-[#fff7e6] disabled:cursor-not-allowed disabled:opacity-50"
                            >
                              {refreshingOrderId === o.orderId
                                ? "Refreshing..."
                                : showRefundTracking
                                  ? "Refresh refund"
                                  : "Refresh tracking"}
                            </button>
                          </div>
                        </article>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="min-h-0 flex-1 overflow-y-auto overscroll-y-contain p-6 sm:p-8 [-webkit-overflow-scrolling:touch]">
        {activeNav === "profile" && (
          <>
            <section className="rounded-[28px] bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.12),transparent_30%),linear-gradient(135deg,#0F3D2E,#0A2C22)] p-7 text-white shadow-[0_20px_45px_rgba(15,61,46,0.18)]">
              <p className="font-[family-name:var(--font-display)] text-5xl">{displayName}</p>
              <p className="mt-3 text-white/75">{displayEmail}</p>
            </section>

            {emailHint && (
              <p className="mt-4 text-sm text-[var(--color-muted)]" role="status">
                Sign-in email is managed by your account provider. For changes, contact support.
              </p>
            )}
            {loginHint && (
              <p className="mt-4 text-sm text-[var(--color-muted)]" role="status">
                You signed in with {loginMethodLabel}. Session security is handled by our auth provider.
              </p>
            )}

            <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
              <DashboardCard
                label="Email"
                value={displayEmail}
                action={<PillButton onClick={() => setEmailHint((v) => !v)}>Edit</PillButton>}
              />
              <DashboardCard
                label="Phone"
                value={phoneDisplay}
                action={
                  phoneMissing ? (
                    <PillButton onClick={() => setActiveNav("support")}>Add</PillButton>
                  ) : (
                    <PillButton onClick={() => setActiveNav("support")}>Edit</PillButton>
                  )
                }
              />
              <DashboardCard
                label="Login method"
                value={loginMethodLabel}
                action={<PillButton onClick={() => setLoginHint((v) => !v)}>Review</PillButton>}
              />
              <DashboardCard
                label="Orders"
                value="View recent purchases"
                action={<PillButton onClick={() => setActiveNav("orders")}>Open</PillButton>}
              />
              <DashboardCard
                label="Addresses"
                value="Manage delivery details"
                action={<PillButton onClick={() => setActiveNav("addresses")}>Manage</PillButton>}
              />
            </div>
          </>
        )}

        {activeNav === "addresses" && (
          <div className="space-y-8">
            <header className="border-b border-[#0F3D2E]/8 pb-8">
              <SectionBadge>Addresses</SectionBadge>
              <h1 className="mt-4 font-[family-name:var(--font-display)] text-6xl text-[#0F3D2E]">Saved Addresses</h1>
              <p className="mt-4 max-w-2xl text-sm leading-7 text-[#615A50]">Manage delivery destinations for checkout.</p>
            </header>

            {error && (
              <p id="profile-form-error" role="alert" className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                {error}
              </p>
            )}

            <div className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)] sm:p-7">
              {loadingData ? (
                <p className="text-sm text-[var(--color-muted)]">Loading addresses...</p>
              ) : addresses.length === 0 ? (
                <p className="text-sm text-[var(--color-muted)]">No addresses saved yet.</p>
              ) : (
                <ul className="space-y-2">
                  {addresses.map((a) => (
                    <li key={a.shippingAddressId} className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-[var(--color-line)] px-3 py-2">
                      <span className="text-sm text-[var(--color-ink)]">
                        {formatAddress(a)}
                        {a.isDefault ? (
                          <span className="ml-2 rounded-full bg-[var(--color-line)] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.08em]">Default</span>
                        ) : null}
                      </span>
                      <div className="flex flex-wrap gap-2">
                        <button type="button" onClick={() => void deleteAddress(a.shippingAddressId)} className="text-xs font-semibold uppercase tracking-[0.12em] text-red-600">
                          Remove
                        </button>
                        <button
                          type="button"
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
                          className="text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-accent-brown)]"
                        >
                          Edit
                        </button>
                        {!a.isDefault ? (
                          <button type="button" onClick={() => void setDefaultAddress(a.shippingAddressId)} className="text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-accent-brown)]">
                            Make default
                          </button>
                        ) : null}
                      </div>
                    </li>
                  ))}
                </ul>
              )}

              <div className="mt-5 grid gap-2 sm:grid-cols-2">
                <div>
                  <label htmlFor="profile-recipient-name" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Recipient name
                  </label>
                  <input
                    id="profile-recipient-name"
                    value={form.recipientName}
                    onChange={(e) => setForm((p) => ({ ...p, recipientName: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-phone-number" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Phone number
                  </label>
                  <input
                    id="profile-phone-number"
                    value={form.phoneNumber}
                    onChange={(e) => setForm((p) => ({ ...p, phoneNumber: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-road" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Road / street
                  </label>
                  <input
                    id="profile-road"
                    value={form.road}
                    onChange={(e) => setForm((p) => ({ ...p, road: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-apartment" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Apartment / house (optional)
                  </label>
                  <input
                    id="profile-apartment"
                    value={form.apartmentNoOrName}
                    onChange={(e) => setForm((p) => ({ ...p, apartmentNoOrName: e.target.value }))}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-city" className="mb-1 block text-xs text-[var(--color-muted)]">
                    City
                  </label>
                  <input
                    id="profile-city"
                    value={form.city}
                    onChange={(e) => setForm((p) => ({ ...p, city: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-state" className="mb-1 block text-xs text-[var(--color-muted)]">
                    State / region
                  </label>
                  <input
                    id="profile-state"
                    value={form.stateRegion}
                    onChange={(e) => setForm((p) => ({ ...p, stateRegion: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-country" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Country
                  </label>
                  <input
                    id="profile-country"
                    value={form.country}
                    onChange={(e) => setForm((p) => ({ ...p, country: e.target.value }))}
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
                <div>
                  <label htmlFor="profile-pincode" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Pincode
                  </label>
                  <input
                    id="profile-pincode"
                    value={form.postalCode}
                    onChange={(e) => setForm((p) => ({ ...p, postalCode: e.target.value.replace(/\D/g, "").slice(0, 6) }))}
                    inputMode="numeric"
                    aria-invalid={!!error}
                    aria-describedby={error ? "profile-form-error" : undefined}
                    className="h-10 w-full rounded-md border border-[var(--color-line)] bg-white px-3 text-sm"
                  />
                </div>
              </div>

              <div className="mt-3 flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  onClick={() => {
                    if (editingAddressId) {
                      void updateAddress(editingAddressId).then(() => setEditingAddressId(null));
                      return;
                    }
                    void addAddress();
                  }}
                  disabled={!canSaveAddress || adding}
                  className="rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white disabled:opacity-50"
                >
                  {adding ? "Saving..." : editingAddressId ? "Update Address" : "Save Address"}
                </button>
                {editingAddressId ? (
                  <button
                    type="button"
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
                    className="rounded-full border border-[var(--color-line)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)]"
                  >
                    Cancel Edit
                  </button>
                ) : null}
              </div>
            </div>
          </div>
        )}

        {activeNav === "settings" && (
          <div className="space-y-8">
            <header className="border-b border-[#0F3D2E]/8 pb-8">
              <SectionBadge>Settings</SectionBadge>
              <h1 className="mt-4 font-[family-name:var(--font-display)] text-6xl text-[#0F3D2E]">Account Settings</h1>
              <p className="mt-4 max-w-2xl text-sm leading-7 text-[#615A50]">Session and sign-in for your account.</p>
            </header>
            <div className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-7 shadow-[0_4px_24px_rgba(10,42,32,0.04)] sm:p-8">
              <p className="text-sm text-[var(--color-muted)]">
                Signed in as <span className="font-medium text-[var(--color-ink)]">{displayEmail}</span> via {loginMethodLabel}.
              </p>
              <button
                type="button"
                onClick={onSignOut}
                className="mt-6 rounded-full bg-[var(--color-accent-gold)] px-5 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white"
              >
                Sign out everywhere on this device
              </button>
            </div>
          </div>
        )}

        {activeNav === "support" && (
          <div className="space-y-8">
            <header className="border-b border-[#0F3D2E]/8 pb-8">
              <SectionBadge>Support</SectionBadge>
              <h1 className="mt-4 font-[family-name:var(--font-display)] text-6xl text-[#0F3D2E]">Support</h1>
              <p className="mt-4 max-w-2xl text-sm leading-7 text-[#615A50]">We are here for order and account questions.</p>
            </header>
            <div className="grid gap-4 lg:grid-cols-2">
              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">Order help</p>
                <div className="mt-4 flex flex-wrap gap-2">
                  <button
                    type="button"
                    onClick={() => setActiveNav("orders")}
                    className="rounded-full border border-[#C9A646]/35 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[#A37D34]"
                  >
                    Track shipment
                  </button>
                  <button
                    type="button"
                    onClick={() => setActiveNav("orders")}
                    className="rounded-full border border-[#C9A646]/35 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[#A37D34]"
                  >
                    Track refund
                  </button>
                  <Link
                    href="/returns-exchanges"
                    className="rounded-full border border-[#C9A646]/35 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[#A37D34]"
                  >
                    Return policy
                  </Link>
                </div>
                <p className="mt-4 text-xs leading-6 text-[#615A50]">
                  Cancellation remains available until pickup is completed. After pickup completion, delivery exceptions and returns are handled through courier updates and support review.
                </p>
              </section>

              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">Contact options</p>
                <ul className="mt-3 space-y-2 text-sm text-[#615A50]">
                  <li>
                    Email:{" "}
                    <a href="mailto:support@sudattas.com" className="font-semibold text-[#0F3D2E] underline-offset-2 hover:underline">
                      support@sudattas.com
                    </a>
                  </li>
                  <li>
                    Phone/WhatsApp:{" "}
                    <a href="tel:+919739097329" className="font-semibold text-[#0F3D2E] underline-offset-2 hover:underline">
                      +91 97390 97329
                    </a>
                  </li>
                  <li>Hours: Mon-Sat, 10:00 AM - 7:00 PM IST</li>
                  <li>Typical response: within 24 hours</li>
                </ul>
              </section>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">My recent issues</p>
                {supportIssues.length === 0 ? (
                  <p className="mt-3 text-sm text-[#615A50]">No active support issues right now.</p>
                ) : (
                  <ul className="mt-3 space-y-2">
                    {supportIssues.map((i) => (
                      <li key={`${i.orderId}-${i.type}`} className="flex items-center justify-between rounded-xl border border-[#E6E0D5] bg-white px-3 py-2">
                        <span className="text-sm text-[#0F3D2E]">
                          {i.type} issue - Order #{i.orderId}
                        </span>
                        <span className="text-xs font-semibold uppercase tracking-[0.12em] text-[#8B816D]">{i.state}</span>
                      </li>
                    ))}
                  </ul>
                )}
              </section>

              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">Shipping info</p>
                {supportLatestShipment ? (
                  <div className="mt-3 space-y-2 text-sm text-[#615A50]">
                    <p>
                      Latest shipment order: <span className="font-semibold text-[#0F3D2E]">#{supportLatestShipment.orderId}</span>
                    </p>
                    <p>
                      Courier:{" "}
                      <span className="font-semibold text-[#0F3D2E]">{supportLatestShipment.ship.carrier || "Pending assignment"}</span>
                    </p>
                    <p>
                      AWB: <span className="font-mono text-[#0F3D2E]">{supportLatestShipment.ship.awbCode || "Not assigned"}</span>
                    </p>
                    <p>
                      Status:{" "}
                      <span className="font-semibold text-[#0F3D2E]">
                        {supportLatestShipment.ship.shiprocketStatusLabel || supportLatestShipment.ship.status}
                      </span>
                    </p>
                    <p>Updated: {formatOrderDateShort(supportLatestShipment.ship.createdAt)}</p>
                  </div>
                ) : (
                  <p className="mt-3 text-sm text-[#615A50]">No shipment records yet.</p>
                )}
              </section>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">Payment and refund info</p>
                <div className="mt-3 grid grid-cols-2 gap-2 text-sm">
                  <div className="rounded-xl border border-[#E6E0D5] bg-white px-3 py-2 text-[#615A50]">
                    Paid: <span className="font-semibold text-[#0F3D2E]">{supportPaymentSummary.paid}</span>
                  </div>
                  <div className="rounded-xl border border-[#E6E0D5] bg-white px-3 py-2 text-[#615A50]">
                    Pending: <span className="font-semibold text-[#0F3D2E]">{supportPaymentSummary.pending}</span>
                  </div>
                  <div className="rounded-xl border border-[#E6E0D5] bg-white px-3 py-2 text-[#615A50]">
                    Refunded: <span className="font-semibold text-[#0F3D2E]">{supportPaymentSummary.refunded}</span>
                  </div>
                  <div className="rounded-xl border border-[#E6E0D5] bg-white px-3 py-2 text-[#615A50]">
                    Failed: <span className="font-semibold text-[#0F3D2E]">{supportPaymentSummary.failed}</span>
                  </div>
                </div>
                <p className="mt-3 text-xs leading-6 text-[#615A50]">Razorpay reference IDs and refund state are visible in your order details timeline.</p>
              </section>

              <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
                <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">Raise a request</p>
                <div className="mt-3 grid gap-2 sm:grid-cols-2">
                  <select
                    value={supportCategory}
                    onChange={(e) => setSupportCategory(e.target.value as SupportCategory)}
                    className="h-10 rounded-md border border-[#DDD4C7] bg-white px-3 text-sm text-[#0F3D2E]"
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
                    className="h-10 rounded-md border border-[#DDD4C7] bg-white px-3 text-sm text-[#0F3D2E]"
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
                  className="mt-2 min-h-[92px] w-full rounded-md border border-[#DDD4C7] bg-white px-3 py-2 text-sm text-[#0F3D2E]"
                />
                <input
                  type="file"
                  multiple
                  accept="image/*"
                  onChange={(e) => setSupportFiles(Array.from(e.target.files ?? []))}
                  className="mt-2 block w-full text-xs text-[#615A50]"
                />
                {supportFiles.length > 0 ? (
                  <p className="mt-1 text-xs text-[#615A50]">{supportFiles.length} file(s) selected</p>
                ) : null}
                <a
                  href={supportEmailHref}
                  className="mt-3 inline-flex rounded-full bg-[#C9A646] px-4 py-2 text-xs font-semibold uppercase tracking-[0.14em] text-white"
                >
                  Create support email draft
                </a>
              </section>
            </div>

            <section className="rounded-2xl border border-[#E8E0D4] bg-[#FFFCF8] p-6 text-sm text-[#615A50] shadow-[0_4px_24px_rgba(10,42,32,0.04)]">
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[#8B816D]">FAQ</p>
              <ul className="mt-3 space-y-2">
                <li>Where is my order? Use Orders &gt; Refresh tracking for latest courier scan.</li>
                <li>When will refund arrive? Usually 3-7 business days after refund is processed.</li>
                <li>How do I change address? Update default address from Addresses tab before checkout.</li>
                <li>How to contact quickly? Use WhatsApp/phone during support hours for urgent issues.</li>
              </ul>
              {defaultAddress ? (
                <p className="mt-3 text-xs text-[#8B816D]">Default delivery address on file: {formatAddress(defaultAddress)}</p>
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
        open={!!cancelDialogOrderId}
        onOpenChange={(open) => {
          if (!open) setCancelDialogOrderId(null);
        }}
      >
        <DialogContent
          title=""
          className="max-w-lg border border-[#E4DDD4] bg-[#FAF7F2] shadow-[0_28px_64px_-16px_rgba(40,32,28,0.18)] [&>div:first-of-type]:border-0 [&>div:first-of-type]:p-4 [&>div:first-of-type]:pb-0"
          contentClassName="space-y-6 px-6 pb-8 pt-2 sm:px-8"
        >
          <h2 className="text-center font-[family-name:var(--font-display)] text-2xl font-semibold leading-tight tracking-tight text-[#1C1917] sm:text-[1.65rem]">
            Wait! We&apos;re sad to see you go.
          </h2>
          <p className="text-center text-sm leading-[1.7] text-[#5C5650] sm:text-[0.9375rem]">
            {`Each piece at Sudatta's is carefully prepared to ensure it reaches you in perfect condition. If there's a specific reason for your cancellation\u2014like a change in size or a delivery timing concern\u2014please let us know. We'd love the chance to make it right before we halt the process.`}
          </p>
          <div className="flex flex-col gap-3 sm:flex-row sm:gap-4">
            <button
              type="button"
              className="flex flex-1 items-center justify-center gap-2 rounded-2xl bg-[#4A2F2C] px-4 py-3.5 text-sm font-semibold text-[#FAF4EB] shadow-[0_6px_20px_rgba(74,47,44,0.25)] transition hover:bg-[#3E2826]"
              onClick={() => setCancelDialogOrderId(null)}
            >
              <Check className="h-5 w-5 shrink-0 text-[#E7CF82]" strokeWidth={2.25} aria-hidden />
              Keep My Selection
            </button>
            <button
              type="button"
              className="flex flex-1 items-center justify-center gap-2 rounded-2xl border border-[#2C2620]/18 bg-[#F0E8DE] px-4 py-3.5 text-sm font-semibold text-[#2C2620] transition hover:bg-[#E8DFD2]"
              onClick={() => void confirmCancelOrder()}
              disabled={!cancelDialogOrderId}
            >
              <X className="h-5 w-5 shrink-0 opacity-80" strokeWidth={2} aria-hidden />
              Continue with Cancellation
            </button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
