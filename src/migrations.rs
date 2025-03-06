use std::{collections::HashSet, fs::DirEntry};

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
        let wow = filename.splitn(3, '_').collect::<Vec<_>>();
        let (id, name) = (wow[0], wow[1]);

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

    pub fn generate_filenames(&self) -> (String, String) {
        let id = self.stringify_id();
        let name = self
            .name
            .chars()
            .take(crate::MIGRATION_MAX_NAME_FOR_FILE)
            .collect::<String>();

        let get_filename = |up_or_down| format!("{}_{}_{}.sql", id, name, up_or_down,);

        (get_filename("up"), get_filename("down"))
    }

    pub fn generate_files(&self) {
        let (up_filename, down_filename) = self.generate_filenames();

        let up_path = format!("{}/{}", &*crate::MIGRATOR_UP_DIR, up_filename);
        let down_path = format!("{}/{}", &*crate::MIGRATOR_DOWN_DIR, down_filename);

        std::fs::write(&up_path, "").unwrap();
        log::debug!("Generated file: {}", up_path);

        std::fs::write(&down_path, "").unwrap();
        log::debug!("Generated file: {}", down_path);
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
pub enum GetMigrationHistoryError {
    IO(std::io::Error),

    InconsistentMigrations {
        up: HashSet<String>,
        down: HashSet<String>,
    },
}

impl std::fmt::Display for GetMigrationHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetMigrationHistoryError::IO(e) => write!(f, "IO error: {}", e),
            GetMigrationHistoryError::InconsistentMigrations { up, down } => {
                if !up.is_empty() {
                    writeln!(f, "The following migrations are missing down files:")?;
                    for migration in up {
                        writeln!(f, "  {}", migration)?;
                    }
                }

                if !down.is_empty() {
                    if !up.is_empty() {
                        writeln!(f)?;
                    }

                    writeln!(f, "The following migrations are missing up files:")?;
                    for migration in down {
                        writeln!(f, "  {}", migration)?;
                    }
                }

                Ok(())
            }
        }
    }
}

pub fn get_migration_history() -> Result<Vec<Migration>, GetMigrationHistoryError> {
    let up_files =
        std::fs::read_dir(&*crate::MIGRATOR_UP_DIR).map_err(GetMigrationHistoryError::IO)?;
    let down_files =
        std::fs::read_dir(&*crate::MIGRATOR_DOWN_DIR).map_err(GetMigrationHistoryError::IO)?;

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
        return Err(GetMigrationHistoryError::InconsistentMigrations {
            up: only_up_files,
            down: only_down_files,
        });
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
        Err(err) => Err(err),
    }
}
