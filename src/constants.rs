use std::sync::LazyLock;

pub const MIGRATIONS_TABLE_NAME: &str = "__migrations__";

pub static CREATE_MIGRATIONS_TABLE_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id TEXT NOT NULL PRIMARY KEY,
            migrated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        MIGRATIONS_TABLE_NAME
    )
});

pub static CREATE_MIGRATIONS_TABLE_UPDATE_TRIGGER_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "CREATE TRIGGER IF NOT EXISTS update_migration_timestamp UPDATE OF id ON {tablename} BEGIN
            UPDATE {tablename} SET migrated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
        END",
        tablename = MIGRATIONS_TABLE_NAME,
    )
});

pub const MIGRATOR_MAIN_DIR: &str = "migrations";
pub const MIGRATOR_SQLITE_SUBDIR_BASENAME: &str = "sqlite";
pub const MIGRATOR_UP_DIR_BASENAME: &str = "up";
pub const MIGRATOR_DOWN_DIR_BASENAME: &str = "down";

pub static MIGRATOR_SQLITE_SUBDIR: LazyLock<String> =
    LazyLock::new(|| format!("{}/{}", MIGRATOR_MAIN_DIR, MIGRATOR_SQLITE_SUBDIR_BASENAME));

pub static MIGRATOR_UP_DIR: LazyLock<String> =
    LazyLock::new(|| format!("{}/{}", &*MIGRATOR_SQLITE_SUBDIR, MIGRATOR_UP_DIR_BASENAME));

pub static MIGRATOR_DOWN_DIR: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}/{}",
        &*MIGRATOR_SQLITE_SUBDIR, MIGRATOR_DOWN_DIR_BASENAME
    )
});

pub const MIGRATION_MAX_NAME_FOR_FILE: usize = 40;
