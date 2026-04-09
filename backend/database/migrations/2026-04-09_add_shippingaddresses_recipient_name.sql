-- Add optional recipient name per shipping address.
-- Idempotent: only add column if it doesn't already exist.

SET @has_col := (
  SELECT COUNT(*)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'ShippingAddresses'
    AND column_name = 'RecipientName'
);

SET @sql := IF(
  @has_col = 0,
  'ALTER TABLE `ShippingAddresses` ADD COLUMN `RecipientName` VARCHAR(255) NULL AFTER `ApartmentNoOrName`',
  'SELECT 1'
);

PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
