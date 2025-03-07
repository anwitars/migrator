use crate::{
    AnyResult, Revision, cli::DatabaseUrl, create_migration_table, get_current_migration,
    get_migration_history,
};

pub fn migration_migrate_up(revision: Revision, database_url: DatabaseUrl) -> AnyResult<()> {
    log::debug!("Target revision: {:?}", revision);

    let conn = database_url.open_connection()?;
    log::debug!("Opened connection to database");

    create_migration_table(&conn)?;
    log::debug!("Migration table created if it didn't exist");

    let all_migrations = get_migration_history()?;
    let current =
        get_current_migration(&conn)?.map(|bytes| String::from_utf8_lossy(&bytes).to_string());
    log::debug!("Current migration: {:?}", current);

    let revisions_to_apply = revision.revisions_to_apply(&all_migrations, current)?;
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

        log::debug!("Applying migration: {:?}", migration);
        migration.up(&conn)?;
    }
    log::debug!("All migrations applied");

    let last_id = revisions_to_apply.last().unwrap();
    log::debug!("Setting last migration id to: {}", last_id);

    conn.execute(
        &format!(
            "INSERT INTO {} (id) VALUES (?)",
            &*crate::MIGRATIONS_TABLE_NAME
        ),
        [last_id],
    )?;
    log::debug!("Migration entry added to the database");

    Ok(())
}
