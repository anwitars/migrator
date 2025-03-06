use clap::Parser;
use migrator::cli::{Cli, Commands, MigrateCommands};
use migrator::commands::migration_history_command;
use migrator::traits::ExitIfError;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate(migrate) => match migrate.command {
            MigrateCommands::Create { name } => migration_create_command(name),
        },
        Commands::History { database_url } => {
            migration_history_command(database_url).exit_if_error();
        }
    }
}

fn migration_create_command(name: String) {
    migrator::create_migrations_dir();

    let migration = migrator::Migration::new(name);
    log::debug!("Initialized migration: {:?}", migration);

    migration.generate_files();
}
