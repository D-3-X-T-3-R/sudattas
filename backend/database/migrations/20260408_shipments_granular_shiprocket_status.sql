-- Granular shipment lifecycle + Shiprocket tracking metadata (shipment_status_id / label from API).
-- Run against existing DBs that already have `Shipments` from 01_schema.sql.

UPDATE `Shipments` SET `status` = 'pending' WHERE `status` IS NULL;

-- Idempotent on both:
-- 1) older DBs without these columns
-- 2) fresh DBs created from current 01_schema.sql (columns already exist)
SET @has_sr_id := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'Shipments'
    AND COLUMN_NAME = 'shiprocket_status_id'
);
SET @sql := IF(
  @has_sr_id = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `shiprocket_status_id` INT NULL DEFAULT NULL COMMENT ''Shiprocket shipment_status_id'' AFTER `carrier`',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @has_sr_label := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'Shipments'
    AND COLUMN_NAME = 'shiprocket_status_label'
);
SET @sql := IF(
  @has_sr_label = 0,
  'ALTER TABLE `Shipments` ADD COLUMN `shiprocket_status_label` VARCHAR(128) NULL DEFAULT NULL COMMENT ''Shiprocket status label (API or mapped)'' AFTER `shiprocket_status_id`',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

ALTER TABLE `Shipments`
  MODIFY COLUMN `status` ENUM(
    'pending',
    'awb_assigned',
    'label_generated',
    'manifest_generated',
    'pickup_scheduled',
    'picked_up',
    'in_transit',
    'out_for_delivery',
    'delivered',
    'rto_initiated',
    'rto_delivered',
    'cancelled',
    'lost',
    'delayed',
    'failed'
  ) NOT NULL DEFAULT 'pending';
