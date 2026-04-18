-- Immutable public order reference for Shiprocket channel order_id and customer-facing display.
-- Format: SUD-{YYYYMMDD}-{RANDOM_SUFFIX} (suffix: uppercase A–Z and 0–9).
-- Idempotent: safe when base schema / backup already has PublicOrderRef and unique index.

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE() AND table_name = 'Orders' AND column_name = 'PublicOrderRef'
);
SET @add_col_sql := IF(
  @col_exists = 0,
  'ALTER TABLE `Orders` ADD COLUMN `PublicOrderRef` VARCHAR(48) NULL AFTER `order_number`',
  'SELECT 1'
);
PREPARE stmt FROM @add_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

UPDATE `Orders`
SET `PublicOrderRef` = CONCAT(
  'SUD-',
  DATE_FORMAT(`OrderDate`, '%Y%m%d'),
  '-',
  UPPER(SUBSTRING(REPLACE(UUID(), '-', ''), 1, 10))
)
WHERE `PublicOrderRef` IS NULL;

SET @col_exists := (
  SELECT COUNT(1) FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND column_name = 'PublicOrderRef'
    AND is_nullable = 'YES'
);
SET @mod_sql := IF(
  @col_exists > 0,
  'ALTER TABLE `Orders` MODIFY COLUMN `PublicOrderRef` VARCHAR(48) NOT NULL',
  'SELECT 1'
);
PREPARE stmt FROM @mod_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @idx_exists := (
  SELECT COUNT(1) FROM information_schema.statistics
  WHERE table_schema = DATABASE() AND table_name = 'Orders' AND index_name = 'uq_orders_public_order_ref'
);
SET @add_idx_sql := IF(
  @idx_exists = 0,
  'CREATE UNIQUE INDEX `uq_orders_public_order_ref` ON `Orders` (`PublicOrderRef`)',
  'SELECT 1'
);
PREPARE stmt FROM @add_idx_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
