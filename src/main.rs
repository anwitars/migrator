use clap::Parser;
use migrator::AnyResult;
use migrator::cli::{Cli, Commands, MigrateCommands};
use migrator::commands::{migration_history_command, migration_migrate_down, migration_migrate_up};
use migrator::traits::ExitIfError;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate(migrate) => match migrate.command {
            MigrateCommands::Create { name } => migration_create_command(name).exit_if_error(),
            MigrateCommands::Up {
                revision,
                database_url,
            } => migration_migrate_up(revision, database_url).exit_if_error(),
            MigrateCommands::Down {
                revision,
                database_url,
            } => migration_migrate_down(revision, database_url).exit_if_error(),
        },
        Commands::History { database_url } => {
            migration_history_command(database_url).exit_if_error();
        }
    }
}

fn migration_create_command(name: String) -> AnyResult<()> {
    migrator::create_migrations_dir()?;

    let migration = migrator::Migration::new(name)?;
    log::debug!("Initialized migration: {:?}", migration);

    migration.generate_files();

    Ok(())
}
