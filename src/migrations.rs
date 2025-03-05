/// Defines a migration. A migration is a set of SQL queries that are executed in order to update
/// the database schema.
#[derive(Debug)]
pub struct Migration {
    /// The unique identifier of the migration. Defined by the current datetime in the format
    /// `YYYYMMDDHHMMSS`.
    id: [u8; 14],

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
