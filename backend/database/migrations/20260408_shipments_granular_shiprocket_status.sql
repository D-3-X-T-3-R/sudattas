-- Granular shipment lifecycle + Shiprocket tracking metadata (shipment_status_id / label from API).
-- Run against existing DBs that already have `Shipments` from 01_schema.sql.

UPDATE `Shipments` SET `status` = 'pending' WHERE `status` IS NULL;

ALTER TABLE `Shipments`
  ADD COLUMN `shiprocket_status_id` INT NULL DEFAULT NULL COMMENT 'Shiprocket shipment_status_id' AFTER `carrier`,
  ADD COLUMN `shiprocket_status_label` VARCHAR(128) NULL DEFAULT NULL COMMENT 'Shiprocket status label (API or mapped)' AFTER `shiprocket_status_id`;

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
