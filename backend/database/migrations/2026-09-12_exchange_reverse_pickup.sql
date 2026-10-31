-- Two-way courier logistics for exchanges: a reverse-pickup shipment (Shiprocket "Create Return
-- Order" API) booked once an exchange is approved, tracked the same way forward shipments are
-- (AWB + status + tracking events), so admin isn't just trusting the customer shipped it back.
-- Mirrors Shipments' shape for the fields that matter here; kept on ExchangeRequests directly
-- (not a row in Shipments) since a reverse pickup belongs to an exchange, not an order.

ALTER TABLE ExchangeRequests
    ADD COLUMN pickup_shiprocket_order_id VARCHAR(100) NULL DEFAULT NULL,
    ADD COLUMN pickup_shiprocket_shipment_id VARCHAR(100) NULL DEFAULT NULL,
    ADD COLUMN pickup_awb_code VARCHAR(100) NULL DEFAULT NULL,
    ADD COLUMN pickup_courier_name VARCHAR(150) NULL DEFAULT NULL,
    ADD COLUMN pickup_status VARCHAR(64) NULL DEFAULT NULL,
    ADD COLUMN pickup_scheduled_at TIMESTAMP NULL DEFAULT NULL,
    ADD COLUMN pickup_tracking_events JSON NULL DEFAULT NULL;
