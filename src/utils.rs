use rusqlite::Connection;

pub fn create_migration_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(&*crate::CREATE_MIGRATIONS_TABLE_SQL, [])?;
    conn.execute(&*crate::CREATE_MIGRATIONS_TABLE_UPDATE_TRIGGER_SQL, [])?;

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
    std::fs::create_dir_all(&*crate::MIGRATOR_UP_DIR)?;
    std::fs::create_dir_all(&*crate::MIGRATOR_DOWN_DIR)?;
    log::debug!("Created migrations directory");
    Ok(())
}
