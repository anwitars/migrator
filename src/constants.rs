use std::sync::LazyLock;

pub const MIGRATIONS_TABLE_NAME: &str = "__migrations__";

pub static CREATE_MIGRATIONS_TABLE_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        MIGRATIONS_TABLE_NAME
    )
});
