# Order confirmation email — outbox payload and content

This document describes what the backend **already emits**, how to **build the email**, and how **OrderPlaced** relates to **PaymentCaptured**.

## 1. Outbox row (all transactional events)

| Column           | `OrderPlaced` value |
|------------------|---------------------|
| `event_type`     | `OrderPlaced` (constant `ORDER_PLACED` in `handlers/outbox/enqueue.rs`) |
| `aggregate_type` | `order` |
| `aggregate_id`   | Order id as string (same as `payload.order_id` as string) |
| `payload`        | JSON object (see below) |
| `status`         | `Pending` until the outbox worker delivers successfully |

Worker: `procedures/outbox_worker.rs` → `notifications::delivery::deliver_event` (currently a **stub** that only logs).

---

## 2. Current `OrderPlaced` JSON payload (as implemented)

Emitted in `procedures/orders/place_order.rs` immediately after the order and line items are created:

```json
{
  "order_id": 123,
  "user_id": 456
}
```

- **`order_id`**: `i64` — primary key of `Orders`.
- **`user_id`**: `i64` — `Orders.UserID`.

Integration test expectation: `integration_place_order_enqueues_order_placed_outbox` in `tests/integration_abandoned_cart_outbox.rs`.

There is **no** email, line items, totals, or address in the payload today. The delivery layer must **load** those from the database using `order_id` (and `user_id` for recipient lookup).

---

## 3. When this event fires vs payment

- **`place_order`** creates the order, order details, payment intent, then enqueues **`OrderPlaced`**. The customer may still need to **complete payment** (e.g. Razorpay).
- **`PaymentCaptured`** is enqueued from `order_state_machine::transition_order_status` when the order moves to **`confirmed`** (paid), together with outbox type `PaymentCaptured`.

So you typically want **two** transactional emails (or one combined policy):

| Event               | Typical email purpose |
|---------------------|------------------------|
| `OrderPlaced`       | “We received your order” + order summary + **pay now** / next steps if checkout is not fully paid. |
| `PaymentCaptured`   | “Payment received” / **receipt** + same or fuller financial detail (closer to a “bill”). |

If your storefront only calls `place_order` **after** payment success, adjust copy so `OrderPlaced` reads like a receipt; still consider a dedicated **paid** email on `PaymentCaptured` for clarity.

`PaymentCaptured` outbox payload today (from `order_state_machine.rs`) is:

```json
{ "order_id": <i64>, "user_id": <i64> }
```

Same minimal shape — enrichment at delivery time applies to both.

---

## 4. Enriching at delivery time (recommended)

In `deliver_event` (or a module it calls), branch on `event.event_type`:

1. Parse `payload` JSON; read `order_id` (and `user_id` if useful).
2. In a **read-only** DB transaction (or separate connection), load:
   - **Order**: `grand_total_minor`, `currency`, `order_date`, `status_id`, `shipping_address_id`, optional `order_number`.
   - **Order lines** (`OrderDetails` + variant/product): title/SKU, quantity, line amounts (you already store `title` on details in `place_order`).
   - **Shipping address** (from `shipping_address_id`): name, phone, lines, city, pincode, etc. (per your `ShippingAddresses` schema).
   - **User** (for `user_id`): email (and name) for **To:** — from your `Users` / PII tables as appropriate.
3. Render HTML + plain text and send via your provider (SES, SendGrid, Resend, …).
4. Return **`Ok(())`** only after the provider accepts; return **`Err`** so the event stays **Pending** and retries.

This avoids bloating the outbox row and keeps a single source of truth in the DB.

---

## 5. Optional: expanded payload at enqueue (v2)

If you prefer **no DB read in the worker** (e.g. serverless with no DB access), extend the `json!` in `place_order` to include a denormalized snapshot, for example:

```json
{
  "order_id": 123,
  "user_id": 456,
  "customer_email": "user@example.com",
  "currency": "INR",
  "grand_total_paise": 79999,
  "placed_at": "2026-04-05T12:00:00Z",
  "lines": [
    { "title": "Saree A", "quantity": 1, "line_total_paise": 79999 }
  ],
  "shipping_summary": "…"
}
```

Trade-offs: larger rows, possible drift if order is amended later, must update integration tests. Prefer **delivery-time load** unless you have a hard constraint.

---

## 6. Suggested email content (OrderPlaced)

Use your brand voice; structurally include:

1. **Subject**  
   - Example: `Order #12345 received — Sudattas`  
   - Use `order_number` if populated, else `order_id`.

2. **Greeting**  
   - Prefer customer name from user/shipping profile.

3. **Order summary**  
   - Order id / order number  
   - Date/time (from `order_date`)  
   - Line items: description, quantity, line total  
   - Subtotal / shipping / tax / discount (if you model them on the order)  
   - **Grand total** (from `grand_total_minor`, formatted as INR)

4. **Shipping address**  
   - Full formatted address block.

5. **Payment / next steps**  
   - If payment may be pending: short CTA or link to pay / order status page.  
   - If already paid before email: say “Payment will be confirmed in a separate message” or skip if you only send after pay.

6. **Footer**  
   - Support email, store URL, legal entity name (for trust; invoice-style footers if you need GST later).

7. **Plain-text part**  
   - Mirror the same information for deliverability.

---

## 7. Suggested email content (PaymentCaptured / receipt)

When implementing `PaymentCaptured` delivery:

- Emphasize **payment received** and repeat **order id**, **date**, **amount paid**, **line items**, **shipping address**.
- Add any **invoice number / GST fields** your accountant requires (often a separate template or PDF).

---

## 8. Code references

| Piece | Location |
|-------|-----------|
| Enqueue `OrderPlaced` | `core_operations/src/procedures/orders/place_order.rs` |
| Outbox constants | `core_operations/src/handlers/outbox/enqueue.rs` |
| Worker | `core_operations/src/procedures/outbox_worker.rs` |
| Delivery stub | `core_operations/src/notifications/delivery.rs` |
| `PaymentCaptured` enqueue | `core_operations/src/order_state_machine.rs` (`transition_order_status` / outbox match) |

---

## 9. Next implementation step

Replace `deliver_event` with: parse `event_type` + `payload` → load data for `OrderPlaced` / `PaymentCaptured` / others → send email → `Ok(())` or `Err` for retry. Keep templates and provider config in env (API keys, from-address, base URL for links).
