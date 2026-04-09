-- Add optional phone number per shipping address.
-- Idempotent: only add column if it doesn't already exist.

SET @has_col := (
  SELECT COUNT(*)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'ShippingAddresses'
    AND column_name = 'PhoneNumber'
);

SET @sql := IF(
  @has_col = 0,
  'ALTER TABLE `ShippingAddresses` ADD COLUMN `PhoneNumber` VARCHAR(32) NULL AFTER `RecipientName`',
  'SELECT 1'
);

PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
