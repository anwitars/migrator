use crate::Revision;
use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Parser)]
#[clap(name = "migrator")]
#[clap(version)]
#[clap(about = crate::constants::ABOUT)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(name = "migrate")]
    Migrate(Migrate),

    #[clap(name = "history")]
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
    Create { name: String },

    #[clap(name = "up")]
    Up {
        revision: Revision,

        #[clap(short, long)]
        database_url: DatabaseUrl,
    },

    #[clap(name = "down")]
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
