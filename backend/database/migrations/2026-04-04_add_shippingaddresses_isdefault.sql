-- Add ShippingAddresses.IsDefault as an incremental migration.
-- Base schema (01_schema.sql) intentionally omits this column.

SET @col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'ShippingAddresses'
    AND column_name = 'IsDefault'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `ShippingAddresses` ADD COLUMN `IsDefault` TINYINT(1) NOT NULL DEFAULT 0 AFTER `UserID`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @idx_exists := (
  SELECT COUNT(1)
  FROM information_schema.statistics
  WHERE table_schema = DATABASE()
    AND table_name = 'ShippingAddresses'
    AND index_name = 'idx_shipping_user_default'
);
SET @add_idx_sql := IF(
  @idx_exists = 0,
  'ALTER TABLE `ShippingAddresses` ADD INDEX `idx_shipping_user_default` (`UserID`, `IsDefault`)',
  'SELECT 1'
);
PREPARE stmt FROM @add_idx_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
