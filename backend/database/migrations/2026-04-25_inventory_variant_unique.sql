-- Enforce one inventory row per variant.
-- This migration intentionally fails if NULL or duplicate VariantID rows exist.

-- Guard 1: fail when VariantID is NULL.
CREATE TEMPORARY TABLE _inventory_variant_not_null_guard (
    variant_id BIGINT NOT NULL PRIMARY KEY
) ENGINE=InnoDB;

INSERT INTO _inventory_variant_not_null_guard (variant_id)
SELECT VariantID
FROM Inventory;

DROP TEMPORARY TABLE _inventory_variant_not_null_guard;

-- Guard 2: fail when duplicate VariantID rows exist.
-- The PRIMARY KEY on the temporary table guarantees duplicate insert failure.
CREATE TEMPORARY TABLE _inventory_variant_unique_guard (
    variant_id BIGINT NOT NULL PRIMARY KEY
) ENGINE=InnoDB;

INSERT INTO _inventory_variant_unique_guard (variant_id)
SELECT VariantID
FROM Inventory;

DROP TEMPORARY TABLE _inventory_variant_unique_guard;

SET @inventory_variant_unique_exists := (
    SELECT COUNT(*)
    FROM information_schema.statistics
    WHERE table_schema = DATABASE()
      AND table_name = 'Inventory'
      AND index_name = 'uq_inventory_variant_id'
);

SET @add_inventory_variant_unique_sql := IF(
    @inventory_variant_unique_exists = 0,
    'ALTER TABLE Inventory ADD CONSTRAINT uq_inventory_variant_id UNIQUE (VariantID)',
    'SELECT 1'
);

PREPARE add_inventory_variant_unique_stmt FROM @add_inventory_variant_unique_sql;
EXECUTE add_inventory_variant_unique_stmt;
DEALLOCATE PREPARE add_inventory_variant_unique_stmt;
