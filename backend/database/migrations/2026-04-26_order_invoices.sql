-- Immutable order invoices (PDF) with secure storage metadata.

CREATE TABLE IF NOT EXISTS Invoices (
    invoice_id BIGINT NOT NULL AUTO_INCREMENT,
    invoice_number VARCHAR(64) NOT NULL,
    order_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    generated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    storage_path VARCHAR(255) NOT NULL,
    pdf_blob LONGBLOB NOT NULL,
    snapshot_json JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (invoice_id),
    UNIQUE KEY uq_invoices_invoice_number (invoice_number),
    UNIQUE KEY uq_invoices_order_id (order_id),
    KEY idx_invoices_user_id (user_id),
    KEY idx_invoices_generated_at (generated_at),
    CONSTRAINT fk_invoices_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID)
        ON DELETE CASCADE,
    CONSTRAINT fk_invoices_user
        FOREIGN KEY (user_id) REFERENCES Users (UserID)
        ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

SET @orders_invoice_id_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND column_name = 'invoice_id'
);
SET @orders_add_invoice_id_col_sql := IF(
  @orders_invoice_id_col_exists = 0,
  'ALTER TABLE `Orders` ADD COLUMN `invoice_id` BIGINT NULL',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_id_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_number_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND column_name = 'invoice_number'
);
SET @orders_add_invoice_number_col_sql := IF(
  @orders_invoice_number_col_exists = 0,
  'ALTER TABLE `Orders` ADD COLUMN `invoice_number` VARCHAR(64) NULL',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_number_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_generated_at_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND column_name = 'invoice_generated_at'
);
SET @orders_add_invoice_generated_at_col_sql := IF(
  @orders_invoice_generated_at_col_exists = 0,
  'ALTER TABLE `Orders` ADD COLUMN `invoice_generated_at` TIMESTAMP NULL DEFAULT NULL',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_generated_at_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_storage_path_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND column_name = 'invoice_storage_path'
);
SET @orders_add_invoice_storage_path_col_sql := IF(
  @orders_invoice_storage_path_col_exists = 0,
  'ALTER TABLE `Orders` ADD COLUMN `invoice_storage_path` VARCHAR(255) NULL',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_storage_path_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_id_uq_exists := (
  SELECT COUNT(1)
  FROM information_schema.statistics
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND index_name = 'uq_orders_invoice_id'
);
SET @orders_add_invoice_id_uq_sql := IF(
  @orders_invoice_id_uq_exists = 0,
  'ALTER TABLE `Orders` ADD UNIQUE KEY `uq_orders_invoice_id` (`invoice_id`)',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_id_uq_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_number_uq_exists := (
  SELECT COUNT(1)
  FROM information_schema.statistics
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND index_name = 'uq_orders_invoice_number'
);
SET @orders_add_invoice_number_uq_sql := IF(
  @orders_invoice_number_uq_exists = 0,
  'ALTER TABLE `Orders` ADD UNIQUE KEY `uq_orders_invoice_number` (`invoice_number`)',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_number_uq_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @orders_invoice_fk_exists := (
  SELECT COUNT(1)
  FROM information_schema.table_constraints
  WHERE table_schema = DATABASE()
    AND table_name = 'Orders'
    AND constraint_type = 'FOREIGN KEY'
    AND constraint_name = 'fk_orders_invoice'
);
SET @orders_add_invoice_fk_sql := IF(
  @orders_invoice_fk_exists = 0,
  'ALTER TABLE `Orders` ADD CONSTRAINT `fk_orders_invoice` FOREIGN KEY (`invoice_id`) REFERENCES `Invoices` (`invoice_id`) ON DELETE SET NULL',
  'SELECT 1'
);
PREPARE stmt FROM @orders_add_invoice_fk_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
