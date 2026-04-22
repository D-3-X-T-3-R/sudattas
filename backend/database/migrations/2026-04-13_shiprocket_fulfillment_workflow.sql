-- Shiprocket fulfillment columns on Shipments + cancel_pending_logistics status.
-- Idempotent: safe when restoring from dumps that already include these columns/indexes
-- but schema_migrations does not yet record this file.

-- --- Columns (each optional) ---

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'shiprocket_external_order_id'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `shiprocket_external_order_id` VARCHAR(100) NULL AFTER `shiprocket_order_id`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'selected_courier_id'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `selected_courier_id` BIGINT NULL AFTER `carrier`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'selected_courier_name'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `selected_courier_name` VARCHAR(150) NULL AFTER `selected_courier_id`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'quoted_shipping_cost'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `quoted_shipping_cost` BIGINT NULL AFTER `selected_courier_name`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'quoted_shipping_quote_payload'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `quoted_shipping_quote_payload` JSON NULL AFTER `quoted_shipping_cost`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'pickup_scheduled_for'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `pickup_scheduled_for` TIMESTAMP NULL AFTER `delivered_at`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'logistics_status'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `logistics_status` VARCHAR(64) NULL AFTER `pickup_scheduled_for`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'can_customer_cancel'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `can_customer_cancel` TINYINT(1) NOT NULL DEFAULT 1 AFTER `logistics_status`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'razorpay_refund_id'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `razorpay_refund_id` VARCHAR(100) NULL AFTER `can_customer_cancel`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'refund_status'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `refund_status` VARCHAR(32) NULL AFTER `razorpay_refund_id`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND column_name = 'refund_initiated_at'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `refund_initiated_at` TIMESTAMP NULL AFTER `refund_status`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- --- Indexes ---

SET @idx_exists := (
  SELECT COUNT(1) FROM information_schema.statistics
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND index_name = 'idx_shipments_external_order'
);
SET @add_idx_sql := IF(
  @idx_exists = 0,
  'CREATE INDEX `idx_shipments_external_order` ON `Shipments` (`shiprocket_external_order_id`)',
  'SELECT 1'
);
PREPARE stmt FROM @add_idx_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @idx_exists := (
  SELECT COUNT(1) FROM information_schema.statistics
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND index_name = 'idx_shipments_cancelable'
);
SET @add_idx_sql := IF(
  @idx_exists = 0,
  'CREATE INDEX `idx_shipments_cancelable` ON `Shipments` (`can_customer_cancel`, `logistics_status`)',
  'SELECT 1'
);
PREPARE stmt FROM @add_idx_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @idx_exists := (
  SELECT COUNT(1) FROM information_schema.statistics
  WHERE table_schema = DATABASE() AND table_name = 'Shipments' AND index_name = 'uq_shipments_refund_id'
);
SET @add_idx_sql := IF(
  @idx_exists = 0,
  'CREATE UNIQUE INDEX `uq_shipments_refund_id` ON `Shipments` (`razorpay_refund_id`)',
  'SELECT 1'
);
PREPARE stmt FROM @add_idx_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

INSERT INTO `OrderStatus` (`StatusName`)
VALUES ('cancel_pending_logistics')
ON DUPLICATE KEY UPDATE `StatusName` = VALUES(`StatusName`);
