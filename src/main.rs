use clap::{Parser, Subcommand};
use rusqlite::Connection;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(name = "migrate")]
    Migrate(Migrate),
}

#[derive(Parser)]
struct Migrate {
    #[clap(subcommand)]
    command: MigrateCommands,
}

#[derive(Subcommand)]
enum MigrateCommands {
    #[clap(name = "create")]
    Create { name: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate(migrate) => match migrate.command {
            MigrateCommands::Create { name } => migration_create_command(name),
        },
    }
}

fn migration_create_command(name: String) {
    migrator::create_migrations_dir();

    let migration = migrator::Migration::new(name);
    println!("{:?}", migration);
    println!("{:?}", migration.created_at());
    println!("{:?}", migration.stringify_id());

    let (up_filename, down_filename) = migration.generate_filenames();
    println!("{:?}", up_filename);
    println!("{:?}", down_filename);
}
