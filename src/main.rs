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
    }
}

fn migration_create_command(name: String) {
    migrator::create_migrations_dir();

    let migration = migrator::Migration::new(name);
    log::debug!("Initialized migration: {:?}", migration);

    migration.generate_files();
}
