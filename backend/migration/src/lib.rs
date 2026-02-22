pub use sea_orm_migration::prelude::*;

mod m20240101_000001_baseline;
mod m20250101_000001_new_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_baseline::Migration),
            Box::new(m20250101_000001_new_tables::Migration),
        ]
    }
}
