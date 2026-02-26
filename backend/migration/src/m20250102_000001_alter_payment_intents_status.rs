use sea_orm::Statement;
use sea_orm_migration::prelude::*;

/// Aligns `payment_intents.status` with code when the table was created from
/// 01_schema.sql (enum 'created','attempted','paid','failed') instead of the
/// migration (enum 'pending','processed','failed').
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250102_000001_alter_payment_intents_status"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE payment_intents MODIFY COLUMN status ENUM('pending','processed','failed') NULL DEFAULT NULL".to_string(),
        ))
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE payment_intents MODIFY COLUMN status ENUM('created','attempted','paid','failed') NULL DEFAULT NULL".to_string(),
        ))
        .await?;
        Ok(())
    }
}
