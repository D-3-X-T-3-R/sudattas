-- "Remove address" was a hard delete_by_id with no order-history guard, so it always failed
-- with a raw FK constraint error (1451) for any address that had ever been used on a real
-- order — Orders.ShippingAddressID references ShippingAddresses with NO ACTION on delete, and
-- that column is NOT NULL, so the FK can't just be nulled out on old orders either. Soft-delete
-- instead, mirroring the same "never hard-delete something order history points at" precedent
-- already used for user accounts (decision #3) and products (archive vs permanently-delete).

ALTER TABLE ShippingAddresses
    ADD COLUMN is_deleted TINYINT(1) NOT NULL DEFAULT 0;
