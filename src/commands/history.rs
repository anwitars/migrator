use crate::{AnyResult, cli::DatabaseUrl};

pub fn migration_history_command(database_url: Option<DatabaseUrl>) -> AnyResult<()> {
    let current = if let Some(db_url) = database_url {
        let conn = db_url.open_connection()?;
        crate::get_current_migration_id(&conn)?
    } else {
        None
    };

    let mut history = crate::get_migration_history()?;
    history.reverse();

    for migration in history {
        let mut text = format!("{} {}", migration.stringify_id(), migration.name());

        if let Some(current) = &current {
            if migration.id == *current {
                text.push_str(" (current)");
            }
        }

        println!("{}", text);
    }

    Ok(())
}
