# Migrator

This simple tool allows you to create various migrations for your database. The crate is created for educational purposes and is not recommended for use in production.

## Limitations

Currently, it can be used only with SQLite databases, and its features are limited to:

- Create a migration
- Apply migrations
- Revert migrations
- Display the history of migrations

## Usage

### Creating a migration

To create a migration, you need to run the following command:

```bash
migrator migration create <name>
```

Where `<name>` is the name of the migration. The name will not be used by the migrator, it is for better readability.
The names have a maximum length of 40 characters. If the user gives a name longer than the specified length, the migrator will truncate it.
2 migration files will be created in the `migrations` directory: one under the `sqlite/up` directory and the other under the `sqlite/down` directory, and both share the same name.
The name of the migration/revision will be the following format: `YYYYMMDDHHMMSS_<name>`, where `YYYYMMDDHHMMSS` is the current date and time.
The first half (the date and time) is the id of the migrations, and is used to order them correctly.

### Applying migrations

To apply migrations, you need to run the following command:

```bash
migrator migration up --database-url <url> <revision>
```

Where `<url>` is the path to the SQLite database, and `<revision>` can be the following:

- `head`: The latest migration
- `head:<n>`: The `n`-th latest migration
  - `head:0`: is the same as `head`
  - `head:1`: is the previous migration before the latest migration
- `current`: The current migration (is not really useful in its own, and is recommended to use the version below)
- `current:<n>`: The `n`-th migration from the current migration
  - `current:0` is the same as `current`
  - `current:1` is the next migration after the current migration
  - `current:-1` is the previous migration before the current migration
- `<id>`: The specific migration

### Reverting migrations

To revert migrations, you need to run the following command:

```bash
migrator migration down --database-url <url> <revision>
```

Where the arguments are the same as the `up` command.

### Displaying the history of migrations

The history of migrations are the ones that are in the `migrations` directory. To display the history of migrations, you need to run the following command:

```bash
migrator migration history
```

And if you also want to display the current migration among the history, you can run the following command:

```bash
migrator migration history --database-url <url>
```

## Future

Since this package is created only for educational purposes, it will not be maintained actively, and will only be used in my future projects (if needed at all).
