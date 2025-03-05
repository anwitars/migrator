use std::sync::LazyLock;

use rusqlite::Connection;

const MIGRATIONS_TABLE_NAME: &str = "__migrations__";

static CREATE_MIGRATIONS_TABLE_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        MIGRATIONS_TABLE_NAME
    )
});

fn create_migration_table(conn: &Connection) {
    conn.execute(&*CREATE_MIGRATIONS_TABLE_SQL, []).unwrap();
}

fn table_exists<S: AsRef<str>>(conn: &Connection, table_name: S) -> bool {
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

fn main() {
    let conn = Connection::open_in_memory().unwrap();
    create_migration_table(&conn);

    let table_exists = table_exists(&conn, MIGRATIONS_TABLE_NAME);

    println!("Table exists: {}", table_exists);
    assert!(table_exists);
}
