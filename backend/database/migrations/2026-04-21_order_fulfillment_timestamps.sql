-- 3-timestamp fulfillment model for launch scope:
-- cancel_window_ends_at, earliest_booking_at, pickup_target_at (+ audit fields for admin pickup target changes).

SET @cancel_window_ends_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'cancel_window_ends_at'
);
SET @cancel_window_ends_sql := IF(
    @cancel_window_ends_exists = 0,
    'ALTER TABLE Orders ADD COLUMN cancel_window_ends_at TIMESTAMP NULL DEFAULT NULL AFTER created_at',
    'SELECT 1'
);
PREPARE cancel_window_ends_stmt FROM @cancel_window_ends_sql;
EXECUTE cancel_window_ends_stmt;
DEALLOCATE PREPARE cancel_window_ends_stmt;

SET @earliest_booking_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'earliest_booking_at'
);
SET @earliest_booking_sql := IF(
    @earliest_booking_exists = 0,
    'ALTER TABLE Orders ADD COLUMN earliest_booking_at TIMESTAMP NULL DEFAULT NULL AFTER cancel_window_ends_at',
    'SELECT 1'
);
PREPARE earliest_booking_stmt FROM @earliest_booking_sql;
EXECUTE earliest_booking_stmt;
DEALLOCATE PREPARE earliest_booking_stmt;

SET @pickup_target_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'pickup_target_at'
);
SET @pickup_target_sql := IF(
    @pickup_target_exists = 0,
    'ALTER TABLE Orders ADD COLUMN pickup_target_at TIMESTAMP NULL DEFAULT NULL AFTER earliest_booking_at',
    'SELECT 1'
);
PREPARE pickup_target_stmt FROM @pickup_target_sql;
EXECUTE pickup_target_stmt;
DEALLOCATE PREPARE pickup_target_stmt;

SET @pickup_target_reason_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'pickup_target_reason'
);
SET @pickup_target_reason_sql := IF(
    @pickup_target_reason_exists = 0,
    'ALTER TABLE Orders ADD COLUMN pickup_target_reason VARCHAR(255) NULL DEFAULT NULL AFTER pickup_target_at',
    'SELECT 1'
);
PREPARE pickup_target_reason_stmt FROM @pickup_target_reason_sql;
EXECUTE pickup_target_reason_stmt;
DEALLOCATE PREPARE pickup_target_reason_stmt;

SET @pickup_target_set_by_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'pickup_target_set_by'
);
SET @pickup_target_set_by_sql := IF(
    @pickup_target_set_by_exists = 0,
    'ALTER TABLE Orders ADD COLUMN pickup_target_set_by VARCHAR(128) NULL DEFAULT NULL AFTER pickup_target_reason',
    'SELECT 1'
);
PREPARE pickup_target_set_by_stmt FROM @pickup_target_set_by_sql;
EXECUTE pickup_target_set_by_stmt;
DEALLOCATE PREPARE pickup_target_set_by_stmt;

SET @pickup_target_updated_at_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'pickup_target_updated_at'
);
SET @pickup_target_updated_at_sql := IF(
    @pickup_target_updated_at_exists = 0,
    'ALTER TABLE Orders ADD COLUMN pickup_target_updated_at TIMESTAMP NULL DEFAULT NULL AFTER pickup_target_set_by',
    'SELECT 1'
);
PREPARE pickup_target_updated_at_stmt FROM @pickup_target_updated_at_sql;
EXECUTE pickup_target_updated_at_stmt;
DEALLOCATE PREPARE pickup_target_updated_at_stmt;

SET @idx_orders_earliest_booking_exists := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND index_name = 'idx_orders_fulfillment_earliest_booking'
);
SET @idx_orders_earliest_booking_sql := IF(
    @idx_orders_earliest_booking_exists = 0,
    'ALTER TABLE Orders ADD INDEX idx_orders_fulfillment_earliest_booking (fulfillment_status, earliest_booking_at)',
    'SELECT 1'
);
PREPARE idx_orders_earliest_booking_stmt FROM @idx_orders_earliest_booking_sql;
EXECUTE idx_orders_earliest_booking_stmt;
DEALLOCATE PREPARE idx_orders_earliest_booking_stmt;

SET @idx_orders_cancel_window_exists := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND index_name = 'idx_orders_cancel_window_ends_at'
);
SET @idx_orders_cancel_window_sql := IF(
    @idx_orders_cancel_window_exists = 0,
    'ALTER TABLE Orders ADD INDEX idx_orders_cancel_window_ends_at (cancel_window_ends_at)',
    'SELECT 1'
);
PREPARE idx_orders_cancel_window_stmt FROM @idx_orders_cancel_window_sql;
EXECUTE idx_orders_cancel_window_stmt;
DEALLOCATE PREPARE idx_orders_cancel_window_stmt;

SET @cancel_window_hours := 12;
SET @pickup_delay_hours := 48;

UPDATE Orders
SET cancel_window_ends_at = COALESCE(
        cancel_window_ends_at,
        DATE_ADD(created_at, INTERVAL @cancel_window_hours HOUR)
    ),
    earliest_booking_at = COALESCE(
        earliest_booking_at,
        COALESCE(cancel_window_ends_at, DATE_ADD(created_at, INTERVAL @cancel_window_hours HOUR))
    ),
    pickup_target_at = COALESCE(
        pickup_target_at,
        DATE_ADD(created_at, INTERVAL @pickup_delay_hours HOUR)
    ),
    pickup_target_updated_at = COALESCE(pickup_target_updated_at, UTC_TIMESTAMP());
