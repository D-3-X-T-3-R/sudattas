use sea_orm::Statement;
use sea_orm_migration::prelude::*;

/// Phase 4: Order snapshot and coupon usage.
/// Adds order-level and line-level snapshot columns, and coupon snapshot on Orders.
/// OrderDetails: unit_price_minor, discount_minor, tax_minor, sku, title, line_attrs.
/// Orders: subtotal_minor, shipping_minor, tax_total_minor, discount_total_minor,
/// grand_total_minor, applied_coupon_id, applied_coupon_code, applied_discount_paise.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250103_000001_order_snapshot_and_coupon"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // OrderDetails: line-level snapshot
        db.execute(Statement::from_string(
            backend,
            [
                "ALTER TABLE OrderDetails",
                "ADD COLUMN unit_price_minor INT NULL DEFAULT NULL,",
                "ADD COLUMN discount_minor INT NULL DEFAULT NULL,",
                "ADD COLUMN tax_minor INT NULL DEFAULT NULL,",
                "ADD COLUMN sku VARCHAR(255) NULL DEFAULT NULL,",
                "ADD COLUMN title VARCHAR(512) NULL DEFAULT NULL,",
                "ADD COLUMN line_attrs JSON NULL DEFAULT NULL",
            ]
            .join(" "),
        ))
        .await?;

        // Orders: order-level snapshot + coupon snapshot
        db.execute(Statement::from_string(
            backend,
            [
                "ALTER TABLE Orders",
                "ADD COLUMN subtotal_minor BIGINT NULL DEFAULT NULL,",
                "ADD COLUMN shipping_minor BIGINT NULL DEFAULT 0,",
                "ADD COLUMN tax_total_minor BIGINT NULL DEFAULT 0,",
                "ADD COLUMN discount_total_minor BIGINT NULL DEFAULT 0,",
                "ADD COLUMN grand_total_minor BIGINT NULL DEFAULT NULL,",
                "ADD COLUMN applied_coupon_id BIGINT NULL DEFAULT NULL,",
                "ADD COLUMN applied_coupon_code VARCHAR(64) NULL DEFAULT NULL,",
                "ADD COLUMN applied_discount_paise INT NULL DEFAULT NULL",
            ]
            .join(" "),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        db.execute(Statement::from_string(
            backend,
            [
                "ALTER TABLE OrderDetails",
                "DROP COLUMN unit_price_minor,",
                "DROP COLUMN discount_minor,",
                "DROP COLUMN tax_minor,",
                "DROP COLUMN sku,",
                "DROP COLUMN title,",
                "DROP COLUMN line_attrs",
            ]
            .join(" "),
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            [
                "ALTER TABLE Orders",
                "DROP COLUMN subtotal_minor,",
                "DROP COLUMN shipping_minor,",
                "DROP COLUMN tax_total_minor,",
                "DROP COLUMN discount_total_minor,",
                "DROP COLUMN grand_total_minor,",
                "DROP COLUMN applied_coupon_id,",
                "DROP COLUMN applied_coupon_code,",
                "DROP COLUMN applied_discount_paise",
            ]
            .join(" "),
        ))
        .await?;

        Ok(())
    }
}
