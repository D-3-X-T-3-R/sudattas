pub use sea_orm_migration::prelude::*;

mod m20240101_000001_baseline_and_new_tables;
mod m20250102_000001_alter_payment_intents_status;
mod m20250103_000001_order_snapshot_and_coupon;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_baseline_and_new_tables::Migration),
            Box::new(m20250102_000001_alter_payment_intents_status::Migration),
            Box::new(m20250103_000001_order_snapshot_and_coupon::Migration),
        ]
    }
}
