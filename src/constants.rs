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

pub const MIGRATOR_MAIN_DIR: &str = "migrations";
pub const MIGRATOR_SQLITE_SUBDIR: &str = "sqlite";
pub const MIGRATOR_UP_DIR: &str = "up";
pub const MIGRATOR_DOWN_DIR: &str = "down";

pub const MIGRATION_MAX_NAME_FOR_FILE: usize = 40;
