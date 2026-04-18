-- Enforce exactly-once coupon redemption per paid order and one active payable
-- intent per order without blocking historical failed/processed intents.

SET @has_coupon_order_unique := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'CouponRedemptions'
    AND INDEX_NAME = 'uq_coupon_redemption_coupon_order'
);
SET @sql := IF(
  @has_coupon_order_unique = 0,
  'ALTER TABLE `CouponRedemptions`
     ADD CONSTRAINT `uq_coupon_redemption_coupon_order`
     UNIQUE (`coupon_id`, `order_id`)',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @has_active_order_col := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'PaymentIntents'
    AND COLUMN_NAME = 'active_order_id'
);
SET @sql := IF(
  @has_active_order_col = 0,
  'ALTER TABLE `PaymentIntents`
     ADD COLUMN `active_order_id` BIGINT
     GENERATED ALWAYS AS (
       CASE
         WHEN `order_id` IS NOT NULL
          AND `status` IN (''pending'', ''client_verified'', ''needs_review'')
         THEN `order_id`
         ELSE NULL
       END
     ) STORED',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @has_active_order_unique := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'PaymentIntents'
    AND INDEX_NAME = 'uq_payment_intents_active_order'
);
SET @sql := IF(
  @has_active_order_unique = 0,
  'ALTER TABLE `PaymentIntents`
     ADD CONSTRAINT `uq_payment_intents_active_order`
     UNIQUE (`active_order_id`)',
  'SELECT 1'
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
