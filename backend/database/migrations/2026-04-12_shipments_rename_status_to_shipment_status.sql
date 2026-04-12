-- Ensure Shipments uses a dedicated enum column name so SeaORM generates ShipmentStatus.
-- Safe for preserve-data mode and idempotent for repeated runs.

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

