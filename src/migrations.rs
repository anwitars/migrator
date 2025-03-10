use std::{array::TryFromSliceError, collections::HashSet, fs::DirEntry};

use rusqlite::Transaction;

use crate::AnyResult;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct MigrationId([u8; 14]);

impl MigrationId {
    pub fn new(id: [u8; 14]) -> Self {
        Self(id)
    }

    pub fn as_bytes(&self) -> &[u8; 14] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

impl std::fmt::Debug for MigrationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("MigrationId({})", self.as_str()))
    }
}

impl ToString for MigrationId {
    fn to_string(&self) -> String {
        std::str::from_utf8(&self.0).unwrap().to_string()
    }
}

impl TryFrom<&str> for MigrationId {
    type Error = TryFromSliceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(value.as_bytes().try_into()?))
    }
}

impl TryFrom<&[u8]> for MigrationId {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

/// Defines a migration. A migration is a set of SQL queries that are executed in order to update
/// the database schema.
#[derive(Debug)]
pub struct Migration {
    /// The unique identifier of the migration. Defined by the current datetime in the format
    /// `YYYYMMDDHHMMSS`.
    pub id: MigrationId,

    /// The name of the migration. This is the name that will be displayed in the migration table.
    name: String,
}

impl Migration {
    pub fn new(name: impl AsRef<str>) -> Result<Self, TryFromSliceError> {
        let id = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let id = id.as_bytes().try_into()?;

        Ok(Self {
            id,
            name: name.as_ref().to_string().replace(" ", "_"),
        })
    }

    pub fn from_filename(filename: impl AsRef<str>) -> Result<Self, TryFromSliceError> {
        let filename = filename.as_ref();
        let wow = filename.splitn(2, '_').collect::<Vec<_>>();
        let (id, name) = (wow[0], wow[1].replace(".sql", ""));

        Ok(Self {
            id: id.as_bytes().try_into()?,
            name: name.to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::parse_from_str(self.id.as_str(), "%Y%m%d%H%M%S").unwrap()
    }

    pub fn stringify_id(&self) -> String {
        self.id.to_string()
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

    fn execute_file(&self, conn: &rusqlite::Connection, filepath: &str) -> AnyResult<()> {
        log::debug!("Executing file: {}", filepath);
        let sql = std::fs::read_to_string(&filepath)?;
        conn.execute_batch(&sql)?;

        Ok(())
    }

    pub fn up(&self, transaction: &Transaction<'_>) -> AnyResult<()> {
        self.execute_file(
            transaction,
            &format!("{}/{}", &*crate::MIGRATOR_UP_DIR, self.generate_filename()),
        )
    }

    pub fn down(&self, conn: &rusqlite::Connection) -> AnyResult<()> {
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
            .replace(".sql", "")
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

pub fn get_current_migration_id(transaction: &Transaction<'_>) -> AnyResult<Option<MigrationId>> {
    match transaction.query_row(
        &format!(
            "SELECT id FROM {} ORDER BY migrated_at DESC LIMIT 1",
            &*crate::MIGRATIONS_TABLE_NAME
        ),
        [],
        |row| {
            let id: String = row.get(0)?;
            let id =
                MigrationId::try_from(id.as_str()).map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(Some(id))
        },
    ) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::SqliteFailure(e, msg)) => {
            if let Some(ref m) = msg {
                if m.contains("no such table") {
                    return Ok(None);
                }
            }
            Err(rusqlite::Error::SqliteFailure(e, msg))?
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err)?,
    }
}
