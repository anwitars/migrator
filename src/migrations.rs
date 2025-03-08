use std::{collections::HashSet, fs::DirEntry};

use crate::AnyResult;

/// Defines a migration. A migration is a set of SQL queries that are executed in order to update
/// the database schema.
#[derive(Debug)]
pub struct Migration {
    /// The unique identifier of the migration. Defined by the current datetime in the format
    /// `YYYYMMDDHHMMSS`.
    pub id: [u8; 14],

    /// The name of the migration. This is the name that will be displayed in the migration table.
    name: String,
}

impl Migration {
    pub fn new(name: impl AsRef<str>) -> Self {
        let id = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let id = id.as_bytes().try_into().unwrap();

        Self {
            id,
            name: name.as_ref().to_string().replace(" ", "_"),
        }
    }

    pub fn from_filename(filename: impl AsRef<str>) -> Self {
        let filename = filename.as_ref();
        let wow = filename.splitn(2, '_').collect::<Vec<_>>();
        let (id, name) = (wow[0], wow[1].replace(".sql", ""));

        Self {
            id: id.as_bytes().try_into().unwrap(),
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        let id = std::str::from_utf8(&self.id).unwrap();
        chrono::NaiveDateTime::parse_from_str(id, "%Y%m%d%H%M%S").unwrap()
    }

    pub fn stringify_id(&self) -> String {
        std::str::from_utf8(&self.id).unwrap().to_string()
    }

    pub fn generate_filename(&self) -> String {
        let id = self.stringify_id();
        let name = self
            .name
            .chars()
            .take(crate::MIGRATION_MAX_NAME_FOR_FILE)
            .collect::<String>();

        format!("{}_{}.sql", id, name)
    }

    pub fn generate_files(&self) {
        let filename = self.generate_filename();

        let up_path = format!("{}/{}", &*crate::MIGRATOR_UP_DIR, &filename);
        let down_path = format!("{}/{}", &*crate::MIGRATOR_DOWN_DIR, &filename);

        std::fs::write(&up_path, "").unwrap();
        log::debug!("Generated file: {}", up_path);

        std::fs::write(&down_path, "").unwrap();
        log::debug!("Generated file: {}", down_path);
    }

    fn execute_file(&self, conn: &rusqlite::Connection, filepath: &str) -> rusqlite::Result<()> {
        log::debug!("Executing file: {}", filepath);
        let sql = std::fs::read_to_string(&filepath).unwrap();
        conn.execute_batch(&sql)?;

        Ok(())
    }

    pub fn up(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        self.execute_file(
            conn,
            &format!("{}/{}", &*crate::MIGRATOR_UP_DIR, self.generate_filename()),
        )
    }

    pub fn down(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        self.execute_file(
            conn,
            &format!(
                "{}/{}",
                &*crate::MIGRATOR_DOWN_DIR,
                self.generate_filename()
            ),
        )
    }
}

impl PartialEq for Migration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Migration {}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug)]
pub struct InconsistentMigrationsError {
    up: HashSet<String>,
    down: HashSet<String>,
}

impl std::fmt::Display for InconsistentMigrationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.up.is_empty() {
            writeln!(f, "The following migrations are missing down files:")?;
            for migration in self.up.iter() {
                writeln!(f, "  {}", migration)?;
            }
        }

        if !self.down.is_empty() {
            if !self.up.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "The following migrations are missing up files:")?;
            for migration in self.down.iter() {
                writeln!(f, "  {}", migration)?;
            }
        }

        Ok(())
    }
}

pub fn get_migration_history() -> AnyResult<Vec<Migration>> {
    let up_files = std::fs::read_dir(&*crate::MIGRATOR_UP_DIR)?;
    let down_files = std::fs::read_dir(&*crate::MIGRATOR_DOWN_DIR)?;

    let filename_mapper = |entry: std::io::Result<DirEntry>| -> String {
        entry
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .replace("_up.sql", "")
            .replace("_down.sql", "")
    };

    let up_files: HashSet<String> = up_files.map(filename_mapper).collect();
    let down_files: HashSet<String> = down_files.map(filename_mapper).collect();

    let only_up_files: HashSet<String> = up_files.difference(&down_files).cloned().collect();
    let only_down_files: HashSet<String> = down_files.difference(&up_files).cloned().collect();

    if !only_up_files.is_empty() || !only_down_files.is_empty() {
        return Err(InconsistentMigrationsError {
            up: only_up_files,
            down: only_down_files,
        })?;
    }

    let mut migrations = up_files
        .iter()
        .map(|filename| {
            let wow = filename.splitn(2, '_').collect::<Vec<_>>();
            let (id, name) = (wow[0], wow[1]);

            Migration {
                id: id.as_bytes().try_into().unwrap(),
                name: name.to_string(),
            }
        })
        .collect::<Vec<_>>();

    migrations.sort();

    Ok(migrations)
}

pub fn get_current_migration(conn: &rusqlite::Connection) -> rusqlite::Result<Option<[u8; 14]>> {
    match conn.query_row(
        &format!(
            "SELECT id FROM {} ORDER BY migrated_at DESC LIMIT 1",
            &*crate::MIGRATIONS_TABLE_NAME
        ),
        [],
        |row| {
            let id: String = row.get(0)?;
            Ok(Some(id.as_bytes().try_into().unwrap()))
        },
    ) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::SqliteFailure(e, msg)) => {
            if let Some(ref m) = msg {
                if m.contains("no such table") {
                    Ok(None)
                } else {
                    Err(rusqlite::Error::SqliteFailure(e, msg))
                }
            } else {
                Err(rusqlite::Error::SqliteFailure(e, msg))
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}
