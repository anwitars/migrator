use rusqlite::{Connection, Transaction};

use crate::{AnyResult, cli::DatabaseUrl};

pub fn create_migration_table(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    transaction.execute(crate::CREATE_MIGRATIONS_TABLE_SQL, [])?;
    transaction.execute(crate::CREATE_MIGRATIONS_TABLE_UPDATE_TRIGGER_SQL, [])?;

    Ok(())
}

pub fn table_exists<S: AsRef<str>>(conn: &Connection, table_name: S) -> rusqlite::Result<bool> {
    let table_name = table_name.as_ref();

    let mut stmt = conn.prepare(
        format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
            table_name
        )
        .as_str(),
    )?;

    let mut rows = stmt.query([])?;

    Ok(rows.next().ok().flatten().is_some())
}

pub fn create_migrations_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(crate::MIGRATOR_UP_DIR)?;
    std::fs::create_dir_all(crate::MIGRATOR_DOWN_DIR)?;
    log::debug!("Created migrations directory");
    Ok(())
}

pub fn run_with_transaction<T, F>(db_url: DatabaseUrl, callback: F) -> AnyResult<T>
where
    F: FnOnce(&Transaction<'_>) -> AnyResult<T>,
{
    let mut conn = db_url.open_connection()?;
    let transaction = conn.transaction()?;
    let result = callback(&transaction);

    match result {
        Ok(value) => {
            transaction.commit()?;
            Ok(value)
        }
        Err(err) => {
            log::debug!("Rolling back transaction due to error: {}", err.to_string());
            transaction.rollback()?;
            Err(err)
        }
    }
}
