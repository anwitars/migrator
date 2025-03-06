use clap::Parser;
use migrator::cli::{Cli, Commands, DatabaseUrl, MigrateCommands};

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate(migrate) => match migrate.command {
            MigrateCommands::Create { name } => migration_create_command(name),
        },
        Commands::History { database_url } => {
            result_app_exit(migration_history_command(database_url))
        }
    }
}

fn migration_create_command(name: String) {
    migrator::create_migrations_dir();

    let migration = migrator::Migration::new(name);
    log::debug!("Initialized migration: {:?}", migration);

    migration.generate_files();
}

fn migration_history_command(database_url: Option<DatabaseUrl>) -> Result<(), ()> {
    let current = if let Some(db_url) = database_url {
        let conn = db_url.open_connection().unwrap();
        migrator::get_current_migration(&conn)
            .unwrap()
            .map(|m| String::from_utf8_lossy(&m).to_string())
    } else {
        None
    };

    let mut history = match migrator::get_migration_history() {
        Ok(history) => history,
        Err(_) => return Err(()),
    };
    history.reverse();

    for migration in history {
        let mut text = format!("{} {}", migration.stringify_id(), migration.name());

        if let Some(current) = &current {
            if &migration.stringify_id() == current {
                text.push_str(" (current)");
            }
        }

        println!("{}", text);
    }

    Ok(())
}

fn result_app_exit(result: Result<(), ()>) {
    if let Err(_) = result {
        std::process::exit(1);
    }
}
