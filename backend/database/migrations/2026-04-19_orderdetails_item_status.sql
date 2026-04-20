-- Resolve SeaORM enum-name collision on OrderDetails.status by renaming to item_status.
-- This keeps line-item cancel states as plain string values: active | cancelled.

SET @has_item_status := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'item_status'
);

SET @has_status := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'status'
);

SET @status_to_item_status_sql := IF(
    @has_item_status = 0 AND @has_status = 1,
    'ALTER TABLE OrderDetails CHANGE COLUMN status item_status VARCHAR(16) NOT NULL DEFAULT ''active''',
    IF(
        @has_item_status = 0 AND @has_status = 0,
        'ALTER TABLE OrderDetails ADD COLUMN item_status VARCHAR(16) NOT NULL DEFAULT ''active'' AFTER line_attrs',
        'SELECT 1'
    )
);
PREPARE status_to_item_status_stmt FROM @status_to_item_status_sql;
EXECUTE status_to_item_status_stmt;
DEALLOCATE PREPARE status_to_item_status_stmt;

UPDATE OrderDetails
SET item_status = 'active'
WHERE item_status IS NULL OR item_status = '';

SET @idx_exists := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND index_name = 'idx_order_details_order_item_status'
);
SET @idx_sql := IF(
    @idx_exists = 0,
    'ALTER TABLE OrderDetails ADD INDEX idx_order_details_order_item_status (OrderID, item_status)',
    'SELECT 1'
);
PREPARE idx_stmt FROM @idx_sql;
EXECUTE idx_stmt;
DEALLOCATE PREPARE idx_stmt;
