-- Invoices are documented as immutable order/tax records, but the original migration
-- (2026-04-26_order_invoices.sql) declared fk_invoices_order/fk_invoices_user as ON DELETE
-- CASCADE, so deleting an Order or User silently destroyed its Invoice. Databases that already
-- ran that migration have the CASCADE constraints in place (CREATE TABLE IF NOT EXISTS means the
-- corrected header in that file only helps fresh installs); this migration switches existing
-- Invoices FKs to RESTRICT so an order/user with an invoice can't be deleted out from under it.

SET @fk_invoices_order_is_cascade := (
    SELECT COUNT(*)
    FROM information_schema.referential_constraints
    WHERE constraint_schema = DATABASE()
      AND table_name = 'Invoices'
      AND constraint_name = 'fk_invoices_order'
      AND delete_rule = 'CASCADE'
);
SET @drop_fk_invoices_order_sql := IF(
    @fk_invoices_order_is_cascade > 0,
    'ALTER TABLE `Invoices` DROP FOREIGN KEY `fk_invoices_order`',
    'SELECT 1'
);
PREPARE drop_fk_invoices_order_stmt FROM @drop_fk_invoices_order_sql;
EXECUTE drop_fk_invoices_order_stmt;
DEALLOCATE PREPARE drop_fk_invoices_order_stmt;

SET @add_fk_invoices_order_sql := IF(
    @fk_invoices_order_is_cascade > 0,
    'ALTER TABLE `Invoices`
       ADD CONSTRAINT `fk_invoices_order`
         FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`)
         ON DELETE RESTRICT',
    'SELECT 1'
);
PREPARE add_fk_invoices_order_stmt FROM @add_fk_invoices_order_sql;
EXECUTE add_fk_invoices_order_stmt;
DEALLOCATE PREPARE add_fk_invoices_order_stmt;

SET @fk_invoices_user_is_cascade := (
    SELECT COUNT(*)
    FROM information_schema.referential_constraints
    WHERE constraint_schema = DATABASE()
      AND table_name = 'Invoices'
      AND constraint_name = 'fk_invoices_user'
      AND delete_rule = 'CASCADE'
);
SET @drop_fk_invoices_user_sql := IF(
    @fk_invoices_user_is_cascade > 0,
    'ALTER TABLE `Invoices` DROP FOREIGN KEY `fk_invoices_user`',
    'SELECT 1'
);
PREPARE drop_fk_invoices_user_stmt FROM @drop_fk_invoices_user_sql;
EXECUTE drop_fk_invoices_user_stmt;
DEALLOCATE PREPARE drop_fk_invoices_user_stmt;

SET @add_fk_invoices_user_sql := IF(
    @fk_invoices_user_is_cascade > 0,
    'ALTER TABLE `Invoices`
       ADD CONSTRAINT `fk_invoices_user`
         FOREIGN KEY (`user_id`) REFERENCES `Users` (`UserID`)
         ON DELETE RESTRICT',
    'SELECT 1'
);
PREPARE add_fk_invoices_user_stmt FROM @add_fk_invoices_user_sql;
EXECUTE add_fk_invoices_user_stmt;
DEALLOCATE PREPARE add_fk_invoices_user_stmt;
