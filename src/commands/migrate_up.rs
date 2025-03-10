use rusqlite::Transaction;

use crate::{
    AnyResult, Revision, create_migration_table, get_migration_history,
    migrations::get_current_migration_id,
};

pub fn migration_migrate_up(revision: Revision, transaction: &Transaction<'_>) -> AnyResult<()> {
    log::debug!("Target revision: {:?}", revision);

    create_migration_table(&transaction)?;
    log::debug!("Migration table created if it didn't exist");

    let all_migrations = get_migration_history()?;
    let current = get_current_migration_id(&transaction)?;
    log::debug!("Current migration: {:?}", current);

    let revisions_to_apply = revision.revisions_to_apply(&all_migrations, current.as_ref())?;
    log::debug!("Revisions to apply: {:?}", revisions_to_apply);

    if revisions_to_apply.is_empty() {
        println!("Already up to date");
        return Ok(());
    }

    for migration_id in revisions_to_apply.iter() {
        let migration = all_migrations
            .iter()
            .find(|m| &m.stringify_id() == migration_id)
            .unwrap();

        println!("Applying migration: {}", migration.stringify_id());
        migration.up(&transaction)?;
    }
    log::debug!("All migrations applied");

    let last_id = revisions_to_apply.last().unwrap();
    log::debug!("Setting last migration id to: {}", last_id);

    transaction.execute(
        &format!(
            "INSERT INTO {} (id) VALUES (?)",
            crate::MIGRATIONS_TABLE_NAME
        ),
        [last_id],
    )?;
    log::debug!("Migration entry added to the database");

    Ok(())
}
