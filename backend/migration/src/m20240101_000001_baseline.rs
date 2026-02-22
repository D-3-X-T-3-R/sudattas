use sea_orm_migration::prelude::*;

/// Baseline migration — marks the original schema as the starting point.
///
/// The original 40+ tables (Users, Products, Orders, Cart, etc.) were created
/// from the initial SQL dump and are NOT re-created here.  This migration acts
/// as a "version zero" so that `sea_orm_migration` knows the database is at a
/// known state before subsequent migrations run.
///
/// To apply the original schema on a fresh database, run the SQL dump first:
///   mysql -u <user> -p <db> < schema/initial_dump.sql
/// Then run migrations:
///   cargo run --bin migrate -- up
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000001_baseline"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // No-op: original schema already exists in the database.
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
