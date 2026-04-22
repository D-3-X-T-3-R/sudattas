-- Resolve schema drift: remove obsolete OrderDetails.status column.
-- Canonical line-level state column is OrderDetails.item_status.

SET @has_item_status := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'item_status'
);

SET @ensure_item_status_sql := IF(
    @has_item_status = 0,
    'ALTER TABLE `OrderDetails`
       ADD COLUMN `item_status` VARCHAR(16) NOT NULL DEFAULT ''active'' AFTER `line_attrs`',
    'SELECT 1'
);
PREPARE ensure_item_status_stmt FROM @ensure_item_status_sql;
EXECUTE ensure_item_status_stmt;
DEALLOCATE PREPARE ensure_item_status_stmt;

SET @has_status := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'status'
);

SET @backfill_item_status_sql := IF(
    @has_status = 1,
    'UPDATE `OrderDetails`
       SET `item_status` = COALESCE(NULLIF(`item_status`, ''''), `status`, ''active'')
     WHERE `item_status` IS NULL OR `item_status` = ''''',
    'UPDATE `OrderDetails`
       SET `item_status` = ''active''
     WHERE `item_status` IS NULL OR `item_status` = '''''
);
PREPARE backfill_item_status_stmt FROM @backfill_item_status_sql;
EXECUTE backfill_item_status_stmt;
DEALLOCATE PREPARE backfill_item_status_stmt;

SET @has_item_status_idx := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND index_name = 'idx_order_details_order_item_status'
);
SET @ensure_item_status_idx_sql := IF(
    @has_item_status_idx = 0,
    'ALTER TABLE `OrderDetails`
       ADD INDEX `idx_order_details_order_item_status` (`OrderID`, `item_status`)',
    'SELECT 1'
);
PREPARE ensure_item_status_idx_stmt FROM @ensure_item_status_idx_sql;
EXECUTE ensure_item_status_idx_stmt;
DEALLOCATE PREPARE ensure_item_status_idx_stmt;

SET @has_status_idx := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND index_name = 'idx_order_details_order_status'
);
SET @drop_status_idx_sql := IF(
    @has_status_idx > 0,
    'ALTER TABLE `OrderDetails` DROP INDEX `idx_order_details_order_status`',
    'SELECT 1'
);
PREPARE drop_status_idx_stmt FROM @drop_status_idx_sql;
EXECUTE drop_status_idx_stmt;
DEALLOCATE PREPARE drop_status_idx_stmt;

SET @has_status := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'status'
);
SET @drop_status_col_sql := IF(
    @has_status = 1,
    'ALTER TABLE `OrderDetails` DROP COLUMN `status`',
    'SELECT 1'
);
PREPARE drop_status_col_stmt FROM @drop_status_col_sql;
EXECUTE drop_status_col_stmt;
DEALLOCATE PREPARE drop_status_col_stmt;
