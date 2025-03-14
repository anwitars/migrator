use crate::Revision;
use clap::{Parser, Subcommand};
use constcat::concat;
use std::str::FromStr;

const ABOUT: &str = "A simple database migration tool";
const MIGRATE_DESC: &str = "Commands related to create, apply and revert migrations";
const HISTORY_DESC: &str = "Show the history of the migrations in the <MIGRATIONS_DIR> directory";

const MIGRATE_CREATE_DESC: &str = "Create a new migration with <name> and current date and time in the following format: <YYYYMMDDHHMMSS>_<name>.sql";
const MIGRATE_UP_DESC_SHORT: &str = "Apply the migration with the given <revision> to the database";
const MIGRATE_DOWN_DESC_SHORT: &str =
    "Revert the migration with the given <revision> from the database";

const MIGRATE_UP_DESC_LONG: &str = concat!(
    "Apply the migration with the given <revision> to the database.

",
    REVISION_HELP
);

const MIGRATE_DOWN_DESC_LONG: &str = concat!(
    "Revert the migration with the given <revision> from the database.

",
    REVISION_HELP
);

const REVISION_HELP: &str = "The <revision> can be the following:
- An absolute revision ID (e.g. 20210101103015)
- A relative revision offset in respect to the latest (head) revision:
  - head - the latest revision
  - head:0 - the same as 'head'
  - head:1 - the previous revision
- A relative revision offset in respect to the current migration state:
  - current - the current migration state
  - current:0 - the same as 'current'
  - current:1 - the next revision
  - current:-1 - the previous revision";

#[derive(Parser)]
#[clap(name = "migrator")]
#[clap(version)]
#[clap(about = ABOUT)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(name = "migrate")]
    #[clap(about = MIGRATE_DESC)]
    Migrate(Migrate),

    #[clap(name = "history")]
    #[clap(about = HISTORY_DESC)]
    History {
        /// If provided, the migration state of the database will be marked as current
        #[clap(short, long)]
        database_url: Option<DatabaseUrl>,
    },
}

#[derive(Parser)]
pub struct Migrate {
    #[clap(subcommand)]
    pub command: MigrateCommands,
}

#[derive(Subcommand)]
pub enum MigrateCommands {
    #[clap(name = "create")]
    #[clap(about = MIGRATE_CREATE_DESC)]
    Create { name: String },

    #[clap(name = "up")]
    #[clap(about = MIGRATE_UP_DESC_SHORT, long_about = MIGRATE_UP_DESC_LONG)]
    Up {
        revision: Revision,

        #[clap(short, long)]
        database_url: DatabaseUrl,
    },

    #[clap(name = "down")]
    #[clap(about = MIGRATE_DOWN_DESC_SHORT, long_about = MIGRATE_DOWN_DESC_LONG)]
    Down {
        revision: Revision,

        #[clap(short, long)]
        database_url: DatabaseUrl,
    },
}

#[derive(Clone)]
pub enum DatabaseUrl {
    Memory,
    File(String),
}

impl DatabaseUrl {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseUrl::Memory => ":memory:",
            DatabaseUrl::File(s) => s.as_str(),
        }
    }

    pub fn open_connection(&self) -> rusqlite::Result<rusqlite::Connection> {
        match self {
            DatabaseUrl::Memory => rusqlite::Connection::open_in_memory(),
            DatabaseUrl::File(s) => rusqlite::Connection::open(s),
        }
    }
}

impl FromStr for DatabaseUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "memory" {
            Ok(DatabaseUrl::Memory)
        } else {
            Ok(DatabaseUrl::File(s.to_string()))
        }
    }
}
