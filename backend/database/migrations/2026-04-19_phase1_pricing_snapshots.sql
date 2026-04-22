-- Phase 1 pricing/refund foundation:
-- persist frozen pricing snapshot columns used for deterministic refund math.

SET @orders_items_before_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'items_total_minor_before_discount'
);
SET @orders_items_before_sql := IF(
    @orders_items_before_exists = 0,
    'ALTER TABLE Orders ADD COLUMN items_total_minor_before_discount BIGINT NULL DEFAULT NULL AFTER subtotal_minor',
    'SELECT 1'
);
PREPARE orders_items_before_stmt FROM @orders_items_before_sql;
EXECUTE orders_items_before_stmt;
DEALLOCATE PREPARE orders_items_before_stmt;

SET @orders_items_after_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'items_total_minor_after_discount'
);
SET @orders_items_after_sql := IF(
    @orders_items_after_exists = 0,
    'ALTER TABLE Orders ADD COLUMN items_total_minor_after_discount BIGINT NULL DEFAULT NULL AFTER discount_total_minor',
    'SELECT 1'
);
PREPARE orders_items_after_stmt FROM @orders_items_after_sql;
EXECUTE orders_items_after_stmt;
DEALLOCATE PREPARE orders_items_after_stmt;

SET @orders_shipping_charge_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'shipping_charge_minor'
);
SET @orders_shipping_charge_sql := IF(
    @orders_shipping_charge_exists = 0,
    'ALTER TABLE Orders ADD COLUMN shipping_charge_minor BIGINT NULL DEFAULT NULL AFTER shipping_minor',
    'SELECT 1'
);
PREPARE orders_shipping_charge_stmt FROM @orders_shipping_charge_sql;
EXECUTE orders_shipping_charge_stmt;
DEALLOCATE PREPARE orders_shipping_charge_stmt;

UPDATE Orders
SET items_total_minor_before_discount = COALESCE(
        items_total_minor_before_discount,
        subtotal_minor
    ),
    shipping_charge_minor = COALESCE(
        shipping_charge_minor,
        COALESCE(shipping_minor, 0)
    ),
    items_total_minor_after_discount = COALESCE(
        items_total_minor_after_discount,
        GREATEST(grand_total_minor - COALESCE(shipping_minor, 0), 0)
    );
