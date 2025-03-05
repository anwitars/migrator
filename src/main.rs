use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(name = "migrate")]
    Migrate(Migrate),

    #[clap(name = "history")]
    History,
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
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate(migrate) => match migrate.command {
            MigrateCommands::Create { name } => migration_create_command(name),
        },
        Commands::History => result_app_exit(migration_history_command()),
    }
}

fn migration_create_command(name: String) {
    migrator::create_migrations_dir();

    let migration = migrator::Migration::new(name);
    log::debug!("Initialized migration: {:?}", migration);

    migration.generate_files();
}

fn migration_history_command() -> Result<(), ()> {
    let mut history = match migrator::get_migration_history() {
        Ok(history) => history,
        Err(_) => return Err(()),
    };
    history.reverse();

    for migration in history {
        println!("{} {}", migration.stringify_id(), migration.name());
    }

    Ok(())
}

fn result_app_exit(result: Result<(), ()>) {
    if let Err(_) = result {
        std::process::exit(1);
    }
}
