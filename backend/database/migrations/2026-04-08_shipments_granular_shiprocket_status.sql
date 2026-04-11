-- Granular shipment lifecycle + Shiprocket tracking metadata (shipment_status_id / label from API).
-- Supports both legacy `status` and canonical `shipment_status` column names.

SET @has_status := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'Shipments'
    AND COLUMN_NAME = 'status'
);
SET @has_shipment_status := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'Shipments'
    AND COLUMN_NAME = 'shipment_status'
);
SET @sql := IF(
  @has_status = 1 AND @has_shipment_status = 0,
  'ALTER TABLE `Shipments` CHANGE COLUMN `status` `shipment_status` ENUM(
    ''pending'',
    ''awb_assigned'',
    ''label_generated'',
    ''manifest_generated'',
    ''pickup_scheduled'',
    ''picked_up'',
    ''in_transit'',
    ''out_for_delivery'',
    ''delivered'',
    ''rto_initiated'',
    ''rto_delivered'',
    ''cancelled'',
    ''lost'',
    ''delayed'',
    ''failed''
  ) NOT NULL DEFAULT ''pending''',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

UPDATE `Shipments`
SET `shipment_status` = 'pending'
WHERE `shipment_status` IS NULL;

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

SET @has_shipment_status := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'Shipments'
    AND COLUMN_NAME = 'shipment_status'
);
SET @sql := IF(
  @has_shipment_status = 1,
  'ALTER TABLE `Shipments`
    MODIFY COLUMN `shipment_status` ENUM(
      ''pending'',
      ''awb_assigned'',
      ''label_generated'',
      ''manifest_generated'',
      ''pickup_scheduled'',
      ''picked_up'',
      ''in_transit'',
      ''out_for_delivery'',
      ''delivered'',
      ''rto_initiated'',
      ''rto_delivered'',
      ''cancelled'',
      ''lost'',
      ''delayed'',
      ''failed''
    ) NOT NULL DEFAULT ''pending''',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
