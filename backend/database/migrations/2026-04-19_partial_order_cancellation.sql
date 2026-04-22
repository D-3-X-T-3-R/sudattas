-- Partial order cancellation + delayed shipment lifecycle.
-- Forward-safe migration: keep existing full-cancel saga compatible while adding hybrid fulfilment.

SET @od_line_total_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'line_total_minor'
);
SET @od_line_total_sql := IF(
    @od_line_total_exists = 0,
    'ALTER TABLE OrderDetails ADD COLUMN line_total_minor BIGINT NOT NULL DEFAULT 0 AFTER Price',
    'SELECT 1'
);
PREPARE od_line_total_stmt FROM @od_line_total_sql;
EXECUTE od_line_total_stmt;
DEALLOCATE PREPARE od_line_total_stmt;

SET @od_status_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'status'
);
SET @od_status_type := (
    SELECT DATA_TYPE
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'status'
    LIMIT 1
);
SET @od_status_sql := IF(
    @od_status_exists = 0,
    'ALTER TABLE OrderDetails ADD COLUMN status VARCHAR(16) NOT NULL DEFAULT ''active'' AFTER line_attrs',
    IF(
        @od_status_type <> 'varchar',
        'ALTER TABLE OrderDetails MODIFY COLUMN status VARCHAR(16) NOT NULL DEFAULT ''active''',
        'SELECT 1'
    )
);
PREPARE od_status_stmt FROM @od_status_sql;
EXECUTE od_status_stmt;
DEALLOCATE PREPARE od_status_stmt;

SET @od_cancelled_at_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND column_name = 'cancelled_at'
);
SET @od_cancelled_at_sql := IF(
    @od_cancelled_at_exists = 0,
    'ALTER TABLE OrderDetails ADD COLUMN cancelled_at TIMESTAMP NULL DEFAULT NULL AFTER status',
    'SELECT 1'
);
PREPARE od_cancelled_at_stmt FROM @od_cancelled_at_sql;
EXECUTE od_cancelled_at_stmt;
DEALLOCATE PREPARE od_cancelled_at_stmt;

SET @idx_exists := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'OrderDetails'
      AND index_name = 'idx_order_details_order_status'
);
SET @idx_sql := IF(
    @idx_exists = 0,
    'ALTER TABLE OrderDetails ADD INDEX idx_order_details_order_status (OrderID, status)',
    'SELECT 1'
);
PREPARE idx_stmt FROM @idx_sql;
EXECUTE idx_stmt;
DEALLOCATE PREPARE idx_stmt;

UPDATE OrderDetails
SET line_total_minor = CASE
    WHEN COALESCE(line_total_minor, 0) = 0
        THEN COALESCE(unit_price_minor, 0) * COALESCE(Quantity, 0)
    ELSE line_total_minor
END;

SET @orders_created_at_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'created_at'
);
SET @orders_created_at_sql := IF(
    @orders_created_at_exists = 0,
    'ALTER TABLE Orders ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP AFTER OrderDate',
    'SELECT 1'
);
PREPARE orders_created_at_stmt FROM @orders_created_at_sql;
EXECUTE orders_created_at_stmt;
DEALLOCATE PREPARE orders_created_at_stmt;

SET @orders_refund_settlement_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'refund_settlement_status'
);
SET @orders_refund_settlement_sql := IF(
    @orders_refund_settlement_exists = 0,
    'ALTER TABLE Orders ADD COLUMN refund_settlement_status VARCHAR(32) NULL DEFAULT NULL COMMENT ''refund_pending|refund_processed|refund_failed|refund_not_applicable''',
    'SELECT 1'
);
PREPARE orders_refund_settlement_stmt FROM @orders_refund_settlement_sql;
EXECUTE orders_refund_settlement_stmt;
DEALLOCATE PREPARE orders_refund_settlement_stmt;

SET @orders_fulfillment_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'fulfillment_status'
);
SET @orders_fulfillment_sql := IF(
    @orders_fulfillment_exists = 0,
    'ALTER TABLE Orders ADD COLUMN fulfillment_status ENUM(''not_created'',''booked'',''pickup_completed'',''in_transit'',''delivered'',''rto'') NOT NULL DEFAULT ''not_created'' AFTER refund_settlement_status',
    'SELECT 1'
);
PREPARE orders_fulfillment_stmt FROM @orders_fulfillment_sql;
EXECUTE orders_fulfillment_stmt;
DEALLOCATE PREPARE orders_fulfillment_stmt;

UPDATE Orders
SET created_at = COALESCE(created_at, OrderDate);

CREATE TABLE IF NOT EXISTS OrderInventoryRestoreItems (
    order_id BIGINT NOT NULL,
    order_detail_id BIGINT NOT NULL,
    restored_quantity BIGINT NOT NULL,
    restored_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (order_id, order_detail_id),
    CONSTRAINT fk_restore_items_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID),
    CONSTRAINT fk_restore_items_order_detail
        FOREIGN KEY (order_detail_id) REFERENCES OrderDetails (OrderDetailID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO OrderStatus (StatusName)
VALUES
    ('active_sale'),
    ('partially_cancelled'),
    ('cancelled')
ON DUPLICATE KEY UPDATE StatusName = VALUES(StatusName);
