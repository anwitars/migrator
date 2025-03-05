use rusqlite::Connection;

pub fn create_migration_table(conn: &Connection) {
    conn.execute(&*crate::CREATE_MIGRATIONS_TABLE_SQL, [])
        .unwrap();
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
    std::fs::create_dir_all(format!(
        "{}/{}/{}",
        crate::MIGRATOR_MAIN_DIR,
        crate::MIGRATOR_SQLITE_SUBDIR,
        crate::MIGRATOR_UP_DIR,
    ))
    .unwrap();

    std::fs::create_dir_all(format!(
        "{}/{}/{}",
        crate::MIGRATOR_MAIN_DIR,
        crate::MIGRATOR_SQLITE_SUBDIR,
        crate::MIGRATOR_DOWN_DIR,
    ))
    .unwrap();
}
