use rusqlite::Connection;

pub fn create_migration_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(&*crate::CREATE_MIGRATIONS_TABLE_SQL, [])?;
    conn.execute(&*crate::CREATE_MIGRATIONS_TABLE_UPDATE_TRIGGER_SQL, [])?;

    Ok(())
}

pub fn table_exists<S: AsRef<str>>(conn: &Connection, table_name: S) -> bool {
    let table_name = table_name.as_ref();

    let mut stmt = conn
        .prepare(
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                table_name
            )
            .as_str(),
        )
        .unwrap();

    let mut rows = stmt.query([]).unwrap();

    rows.next().unwrap().is_some()
}

pub fn create_migrations_dir() {
    std::fs::create_dir_all(&*crate::MIGRATOR_UP_DIR).unwrap();
    std::fs::create_dir_all(&*crate::MIGRATOR_DOWN_DIR).unwrap();
    log::debug!("Created migrations directory");
}
