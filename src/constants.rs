use constcat::concat;

macro_rules! join_dirs {
    ($($dir:expr),*) => {
        concat!($($dir, "/"),*)
    };
}

pub const MIGRATIONS_TABLE_NAME: &str = "__migrations__";

pub const CREATE_MIGRATIONS_TABLE_SQL: &str = concat!(
    "CREATE TABLE IF NOT EXISTS ",
    MIGRATIONS_TABLE_NAME,
    " (
        migrated_at TIMESTAMP NOT NULL PRIMARY KEY DEFAULT CURRENT_TIMESTAMP,
        id TEXT NOT NULL
    )"
);

pub const CREATE_MIGRATIONS_TABLE_UPDATE_TRIGGER_SQL: &str = concat!(
    "CREATE TRIGGER IF NOT EXISTS update_migration_timestamp UPDATE OF id ON ",
    MIGRATIONS_TABLE_NAME,
    " BEGIN
        UPDATE ",
    MIGRATIONS_TABLE_NAME,
    " SET migrated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END"
);

pub const MIGRATOR_MAIN_DIR: &str = "migrations";
pub const MIGRATOR_SQLITE_SUBDIR_BASENAME: &str = "sqlite";
pub const MIGRATOR_UP_DIR_BASENAME: &str = "up";
pub const MIGRATOR_DOWN_DIR_BASENAME: &str = "down";

pub const MIGRATOR_SQLITE_SUBDIR: &str =
    join_dirs!(MIGRATOR_MAIN_DIR, MIGRATOR_SQLITE_SUBDIR_BASENAME);
pub const MIGRATOR_UP_DIR: &str = join_dirs!(MIGRATOR_SQLITE_SUBDIR, MIGRATOR_UP_DIR_BASENAME);
pub const MIGRATOR_DOWN_DIR: &str = join_dirs!(MIGRATOR_SQLITE_SUBDIR, MIGRATOR_DOWN_DIR_BASENAME);

pub const MIGRATION_MAX_NAME_FOR_FILE: usize = 40;
