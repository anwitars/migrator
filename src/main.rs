use rusqlite::Connection;

fn main() {
    let conn = Connection::open_in_memory().unwrap();
    migrator::create_migration_table(&conn);

    let table_exists = migrator::table_exists(&conn, migrator::MIGRATIONS_TABLE_NAME);

    println!("Table exists: {}", table_exists);
    assert!(table_exists);
}
