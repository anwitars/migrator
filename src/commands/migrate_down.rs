use rusqlite::Transaction;

use crate::{
    AnyResult, Revision, create_migration_table, get_migration_history,
    migrations::get_current_migration_id,
};

pub fn migration_migrate_down(target: Revision, transaction: &Transaction<'_>) -> AnyResult<()> {
    log::debug!("Target revision: {:?}", target);

    create_migration_table(&transaction)?;
    log::debug!("Migration table created if it didn't exist");

    let all_migrations = get_migration_history()?;
    let current = get_current_migration_id(&transaction)?;
    log::debug!("Current migration: {:?}", current);

    let revisions_to_revert = target.revisions_to_revert(&all_migrations, current.as_ref())?;
    log::debug!("Revisions to revert: {:?}", revisions_to_revert);

    if revisions_to_revert.is_empty() {
        println!("Already reverted to the target revision");
        return Ok(());
    }

    for migration_id in revisions_to_revert.iter() {
        let migration = all_migrations
            .iter()
            .find(|m| &m.stringify_id() == migration_id)
            .unwrap();

        println!("Reverting migration: {}", migration.stringify_id());
        migration.down(&transaction)?;
    }
    log::debug!("All migrations applied");

    let last_id = target.resolve_revision_id(&all_migrations)?;

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
