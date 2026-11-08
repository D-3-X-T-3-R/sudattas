use core_db_entities::CoreDatabaseConnection;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use std::collections::BTreeSet;

pub const REQUIRED_TABLES: &[&str] = &[
    "RefundAttempts",
    "ReturnRequests",
    "ReturnRequestItems",
    "OrderInventoryRestores",
    "OrderInventoryRestoreItems",
    "Invoices",
    "ExchangeRequests",
    "SchemaMigrations",
];

fn normalize_table_name(name: &str) -> String {
    name.trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn camel_to_snake(name: &str) -> String {
    let mut output = String::with_capacity(name.len() + 4);
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch.to_ascii_lowercase());
        }
    }
    output
}

fn table_name_candidates(required_name: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    let trimmed = required_name.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    names.insert(trimmed.to_string());
    names.insert(trimmed.to_ascii_lowercase());
    names.insert(camel_to_snake(trimmed));
    names.into_iter().collect()
}

pub fn validate_required_tables_from_found<I, S>(found_tables: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let found: BTreeSet<String> = found_tables
        .into_iter()
        .map(|name| normalize_table_name(name.as_ref()))
        .filter(|name| !name.is_empty())
        .collect();

    let missing: Vec<&str> = REQUIRED_TABLES
        .iter()
        .copied()
        .filter(|table| !found.contains(&normalize_table_name(table)))
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Missing required database table(s): {}. Run migrations before startup.",
            missing.join(", ")
        ))
    }
}

fn is_missing_table_error(error_message: &str) -> bool {
    let lower = error_message.to_ascii_lowercase();
    lower.contains("1146")
        || lower.contains("unknown table")
        || lower.contains("doesn't exist")
        || lower.contains("does not exist")
}

async fn table_exists(db: &CoreDatabaseConnection, table_name: &str) -> Result<bool, String> {
    let sql = format!("SELECT 1 FROM `{}` LIMIT 1", table_name.replace('`', "``"));
    match db
        .query_all(Statement::from_string(DbBackend::MySql, sql))
        .await
    {
        Ok(_) => Ok(true),
        Err(err) => {
            let message = err.to_string();
            if is_missing_table_error(&message) {
                Ok(false)
            } else {
                Err(format!(
                    "Required table validation query failed for '{table_name}': {message}"
                ))
            }
        }
    }
}

pub async fn validate_required_schema(db: &CoreDatabaseConnection) -> Result<(), String> {
    let mut missing = Vec::new();
    for required in REQUIRED_TABLES.iter().copied() {
        let mut found = false;
        for candidate in table_name_candidates(required) {
            if table_exists(db, &candidate).await? {
                found = true;
                break;
            }
        }
        if !found {
            missing.push(required);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Missing required database table(s): {}. Run migrations before startup.",
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_required_tables_accepts_full_schema() {
        let found = REQUIRED_TABLES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        assert!(validate_required_tables_from_found(found).is_ok());
    }

    #[test]
    fn validate_required_tables_accepts_snake_case_forms() {
        let found = vec![
            "refund_attempts".to_string(),
            "return_requests".to_string(),
            "return_request_items".to_string(),
            "order_inventory_restores".to_string(),
            "order_inventory_restore_items".to_string(),
            "invoices".to_string(),
            "exchange_requests".to_string(),
            "schema_migrations".to_string(),
        ];
        assert!(validate_required_tables_from_found(found).is_ok());
    }

    #[test]
    fn validate_required_tables_reports_missing_tables() {
        let found = vec![
            "RefundAttempts".to_string(),
            "ReturnRequests".to_string(),
            "ReturnRequestItems".to_string(),
            "OrderInventoryRestores".to_string(),
        ];
        let err = validate_required_tables_from_found(found).expect_err("expected missing table");
        assert!(err.contains("OrderInventoryRestoreItems"));
        assert!(err.contains("Invoices"));
        assert!(err.contains("SchemaMigrations"));
    }

    #[test]
    fn table_name_candidates_include_snake_case() {
        let names = table_name_candidates("OrderInventoryRestoreItems");
        assert!(names.contains(&"OrderInventoryRestoreItems".to_string()));
        assert!(names.contains(&"order_inventory_restore_items".to_string()));
    }
}
