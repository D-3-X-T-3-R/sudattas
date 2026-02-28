# Database Schema

MySQL database for the Sudattas e-commerce backend.
Entities are generated via `sea-orm-cli generate entity` and live in `core_db_entities/src/entity/`.

## Regenerating Entities

**CI verifies** that committed entities match the schema. If you change the schema, regenerate entities before pushing:

```powershell
# From backend/: start MySQL (docker-compose up -d), then regenerate
cd backend/core_db_entities
.\regenerate_entities.ps1
```

Or manually:
```bash
# 1. Start MySQL (cd backend && docker-compose up -d)
# 2. Wait for MySQL to be ready
cargo install sea-orm-cli --locked
sea-orm-cli generate entity -u "mysql://root:12345678@localhost:3306/SUDATTAS" \
  -o backend/core_db_entities/src/entity --with-serde both --date-time-crate chrono
```

---

## Core Domain Tables

### Users
Primary user record.

| Column | Type | Notes |
|---|---|---|
| UserID | BIGINT PK | Auto-increment |
| Username | VARCHAR | |
| Password | VARCHAR | Legacy plain-text (deprecated) |
| password_hash | VARCHAR | Argon2id hash (use this) |
| Email | VARCHAR UNIQUE | |
| email_verified | TINYINT | Boolean flag |
| FullName | VARCHAR | |
| Phone | VARCHAR | |
| status | ENUM(active, inactive, suspended) | |
| CreateDate | TIMESTAMP | |
| updated_at | TIMESTAMP | |

### Products
Product catalogue.

| Column | Type | Notes |
|---|---|---|
| ProductID | BIGINT PK | |
| Name | VARCHAR | |
| Description | TEXT | |
| Price | DECIMAL(10,2) | |
| StockQuantity | BIGINT | |
| CategoryID | BIGINT FK→Categories | |

### Categories
Product taxonomy.

| Column | Type | Notes |
|---|---|---|
| CategoryID | BIGINT PK | |
| Name | VARCHAR | |
| Description | TEXT | |
| ParentCategoryID | BIGINT | Self-referencing for nesting |

### Cart
Per-user or per-session shopping cart line items.

| Column | Type | Notes |
|---|---|---|
| CartID | BIGINT PK | |
| UserID | BIGINT FK→Users | Null for guest carts |
| session_id | VARCHAR | Guest identifier |
| ProductID | BIGINT FK→Products | |
| Quantity | INT | |
| AddedAt | TIMESTAMP | |

---

## Order Tables

### Orders

| Column | Type | Notes |
|---|---|---|
| OrderID | BIGINT PK | |
| order_number | VARCHAR UNIQUE | Human-readable reference |
| UserID | BIGINT FK→Users | |
| OrderDate | TIMESTAMP | |
| ShippingAddressID | BIGINT FK→ShippingAddresses | |
| TotalAmount | DECIMAL(10,2) | After coupon deduction |
| StatusID | BIGINT FK→OrderStatus | |
| payment_status | ENUM(pending, authorized, captured, failed) | |
| currency | VARCHAR | ISO 4217 (e.g. INR) |
| updated_at | TIMESTAMP | |

### OrderDetails
Line items for each order.

| Column | Type | Notes |
|---|---|---|
| OrderDetailID | BIGINT PK | |
| OrderID | BIGINT FK→Orders | |
| ProductID | BIGINT FK→Products | |
| Quantity | INT | |
| Price | DECIMAL(10,2) | Price at time of order |

### order_events
Immutable audit log for order lifecycle changes.

| Column | Type | Notes |
|---|---|---|
| event_id | BIGINT PK | |
| order_id | BIGINT FK→Orders | |
| event_type | VARCHAR | e.g. `order_placed`, `status_changed` |
| from_status | VARCHAR | Previous status value |
| to_status | VARCHAR | New status value |
| actor_type | ENUM(customer, admin, system) | Who triggered the event |
| message | TEXT | Optional human-readable note |
| created_at | TIMESTAMP | |

### OrderStatus
Lookup table for order status labels.

| Column | Type | Notes |
|---|---|---|
| StatusID | BIGINT PK | |
| StatusName | VARCHAR | e.g. Pending, Shipped, Delivered |

---

## Payment & Fulfilment Tables

### payment_intents
Razorpay payment intent lifecycle tracking.

| Column | Type | Notes |
|---|---|---|
| intent_id | BIGINT PK | Internal ID |
| razorpay_order_id | VARCHAR UNIQUE | Razorpay's order reference |
| order_id | BIGINT FK→Orders | |
| user_id | BIGINT FK→Users | |
| amount_paise | INT | Amount in paise (×100 for ₹) |
| currency | VARCHAR | |
| status | ENUM(pending, processed, failed) | |
| razorpay_payment_id | VARCHAR | Set after capture |
| metadata | JSON | Raw Razorpay response |
| created_at | TIMESTAMP | |
| expires_at | TIMESTAMP | Intent expiry |

### shipments
Fulfilment tracking per order.

| Column | Type | Notes |
|---|---|---|
| shipment_id | BIGINT PK | |
| order_id | BIGINT FK→Orders | |
| shiprocket_order_id | VARCHAR | Shiprocket reference |
| awb_code | VARCHAR | Air waybill / tracking number |
| carrier | VARCHAR | e.g. BlueDart, Delhivery |
| status | ENUM(pending, processed, failed) | |
| tracking_events | JSON | Array of carrier status pushes |
| created_at | TIMESTAMP | |
| delivered_at | TIMESTAMP | Set when status = processed |

### webhook_events
Idempotent inbound webhook records.

| Column | Type | Notes |
|---|---|---|
| event_id | BIGINT PK | |
| provider | VARCHAR | e.g. `razorpay` |
| event_type | VARCHAR | e.g. `payment.captured` |
| webhook_id | VARCHAR UNIQUE | Provider's event ID (idempotency key) |
| payload | JSON | Full raw body |
| status | ENUM(pending, processed, failed) | Processing state |
| received_at | TIMESTAMP | |

### outbox_events (P1)
Transactional notification outbox: worker publishes pending events (emails/SMS) idempotently.

| Column | Type | Notes |
|---|---|---|
| event_id | BIGINT PK | |
| event_type | VARCHAR | OrderPlaced, PaymentCaptured, Shipped, Delivered, Refunded |
| aggregate_type | VARCHAR | e.g. `order` |
| aggregate_id | VARCHAR | e.g. order_id |
| payload | JSON | Template variables |
| status | ENUM(pending, processed, failed) | processed = published |
| created_at | TIMESTAMP | |
| published_at | TIMESTAMP | Set when status = processed |

---

## Promotions & Discounts

### coupons

| Column | Type | Notes |
|---|---|---|
| coupon_id | BIGINT PK | |
| code | VARCHAR UNIQUE | Human-entered coupon code |
| discount_type | ENUM(percentage, fixed_amount) | |
| discount_value | INT | % or paise depending on type |
| min_order_value_paise | INT | Minimum cart value to apply |
| usage_limit | INT | Max total uses; null = unlimited |
| usage_count | INT | Current use count |
| status | ENUM | Active/inactive |
| starts_at | TIMESTAMP | Valid from |
| ends_at | TIMESTAMP | Valid until; null = no expiry |

### Discounts
Product-level price reductions (not coupon-based).

| Column | Type | Notes |
|---|---|---|
| DiscountID | BIGINT PK | |
| ProductID | BIGINT FK→Products | |
| DiscountPercentage | DECIMAL | |
| StartDate / EndDate | TIMESTAMP | |

### Promotions
Marketing promotions (banner-level, not cart-level).

---

## Auth & Sessions

### sessions
Guest and authenticated session records.

| Column | Type | Notes |
|---|---|---|
| session_id | VARCHAR PK | UUID, also stored in Redis |
| user_id | BIGINT FK→Users | Null for guest sessions |
| data | JSON | Session payload |
| ip_address | VARCHAR | |
| last_activity | TIMESTAMP | |
| expires_at | TIMESTAMP | |
| created_at | TIMESTAMP | |

---

## Shipping

### ShippingAddresses
Per-user saved addresses.

| Column | Type | Notes |
|---|---|---|
| ShippingAddressID | BIGINT PK | |
| UserID | BIGINT FK→Users | |
| AddressLine1/2 | VARCHAR | |
| City / State / PostalCode / Country | VARCHAR | |

### ShippingMethods / ShippingZones
Carrier options and geographic zones for rate calculation.

---

## Product Extras

### ProductImages
Stores CDN references (R2). Base64 path removed.

| Column | Type | Notes |
|---|---|---|
| ImageID | BIGINT PK | |
| ProductID | BIGINT FK→Products | |
| cdn_path | VARCHAR | R2 object key |
| url | VARCHAR | Public CDN URL |
| thumbnail_url | VARCHAR | Optional thumbnail URL |
| AltText | VARCHAR | |

### Reviews / ProductRatings / ProductAttributes / ProductVariants
Standard product enrichment tables.

### Inventory
Stock management per product.

| Column | Type | Notes |
|---|---|---|
| InventoryID | BIGINT PK | |
| ProductID | BIGINT FK→Products | |
| QuantityAvailable | INT | Decremented at place_order |
| ReorderLevel | INT | Low-stock threshold |

---

## Schema source

The database schema is maintained only in SQL files (no Rust migrations):

- **`backend/database/sql_dump/01_schema.sql`** — single schema file: creates database `SUDATTAS`, drops existing tables if present, and creates all tables (Orders, OrderDetails, payment_intents, coupons, idempotency_keys, webhook_events, etc.) with P1 constraints and indexes.
- **`backend/database/sql_dump/02_data.sql`** — optional seed/reference data.

### P1 Data model: constraints and indexes (stop relying on code)

**Uniques:** Products.`sku`, Products.`slug`, coupons.`code`, webhook_events (`provider`, `webhook_id`), payment_intents.`razorpay_payment_id` (nullable; multiple NULLs allowed).  
**Foreign keys:** Orders → Users, ShippingAddresses, OrderStatus, coupons (applied_coupon_id); OrderDetails → Orders, Products; payment_intents → Orders, Users; etc.  
**Not-null:** payment_intents.`status` NOT NULL DEFAULT 'pending'; other required fields as in 01_schema.  
**Indexes (high-volume):** Orders (`OrderDate`, `StatusID`, `UserID`); OrderDetails (`OrderID`); Inventory (`ProductID`); coupons (`code`, `ends_at`).

### Migrations safety

- **Forward-only:** Schema changes are additive (new constraints, new indexes, new tables). No destructive ALTERs without backfill.
- **CI:** Backend CI loads `01_schema.sql` on an empty MySQL and runs entity generation + tests; this validates that the schema loads and matches committed entities.
- After changing 01_schema, regenerate entities and commit: `cd backend/core_db_entities && ./regenerate_entities.ps1` (or the shell equivalent).

### Loading the schema

```bash
# Load schema (and optionally data) into MySQL
mysql -h HOST -P PORT -u USER -p < backend/database/sql_dump/01_schema.sql
# Optional: mysql -h HOST -P PORT -u USER -p < backend/database/sql_dump/02_data.sql
```
