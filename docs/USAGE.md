# Using this repository

This doc explains how the pieces of `tax_ctrl_core` fit together, and how
a consumer (the Flutter app via Flutter Rust Bridge, a test, or a future
CLI) actually uses them. For a full worked example, see
[`tutorial-todo-list.md`](tutorial-todo-list.md).

## The three moving parts

Every feature built on this crate touches the same three layers, in the
same order:

1. **A migration** (`src/persistence/schema.rs` + a migration file) —
   defines what a table looks like, in SQL-agnostic form. Runs against
   whichever backend `DATABASE_URL` points at.
2. **An entity** (`src/persistence/entities/`) — a Rust struct mirroring
   one table, generated or hand-written, giving you typed queries instead
   of raw SQL.
3. **A repository** (`src/persistence/repository.rs`) — the API the rest
   of the app actually calls. Wraps an entity with the specific queries
   your feature needs.

You always add them in that order: you can't generate an entity for a
table that doesn't exist yet, and a repository is just typed convenience
on top of an entity.

## Connecting

Everything starts from a `DatabaseConnection`:

```rust
use tax_ctrl_core::persistence::connect_from_env;

let db = connect_from_env().await?;
```

`connect_from_env()` reads `DATABASE_URL` (loading `.env` if present) and
returns a connection — SQLite or PostgreSQL, whichever the URL's scheme
says. Nothing downstream needs to know or care which one it got.

If you're not using environment variables (e.g. the Flutter app passes a
path down through Flutter Rust Bridge instead), build the config
explicitly:

```rust
use tax_ctrl_core::persistence::{connect, DbConfig};

let config = DbConfig::from_url(format!("sqlite://{db_path}?mode=rwc"));
let db = connect(&config).await?;
```

## Applying migrations

Once connected, bring the schema up to date:

```rust
use sea_orm_migration::MigratorTrait;
use tax_ctrl_core::persistence::Migrator;

Migrator::up(&db, None).await?;
```

This is idempotent — safe to call every time the app starts. It's also
identical regardless of backend: `Migrator` just runs each registered
migration's `up()` against whatever `db` is.

## Querying through a repository

```rust
use tax_ctrl_core::persistence::repository::Repository;
// use tax_ctrl_core::persistence::entities::some_entity;

let repo: Repository<some_entity::Entity> = Repository::new(&db);
let all = repo.find_all().await?;
let one = repo.find_by_id(1).await?;
```

`Repository<E>` only covers find-all/find-by-id. Anything more specific
(filters, joins, inserts, updates) belongs in a feature-specific
repository built on top — see the tutorial for a concrete example.

## Where sea-orm-cli fits in

[`sea-orm-cli`](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/)
is a **separate command-line tool**, not a dependency of this crate. It's
a dev-time scaffolding tool with two jobs:

- **`sea-orm-cli migrate generate <name>`** — scaffolds a new, empty
  migration file with the right boilerplate and timestamp-based name, so
  you're not hand-writing that structure from scratch.
- **`sea-orm-cli generate entity --database-url ... --output-dir ...`** —
  connects to a real database, inspects its schema, and writes out the
  matching `Model`/`Entity`/`Relation` Rust structs for you.

It can also run migrations directly from the terminal
(`sea-orm-cli migrate up`, `migrate status`, etc.), which is convenient
while developing but isn't how migrations run in the app itself — that
goes through `Migrator::up()` in code, as shown above.

Install it once, globally, as a dev tool:

```bash
cargo install sea-orm-cli
```

It's deliberately **not** a project dependency: `sea-orm-migration`'s
`cli` feature (which would pull in `sea-orm-cli` and `clap`) is disabled
in `Cargo.toml` on purpose, so the actual embedded library — the thing
that ships inside the Flutter app — doesn't carry a command-line
argument parser and code-generation machinery it will never use at
runtime. You only need `sea-orm-cli` installed on your own machine while
developing, not as something the app depends on.

## Testing

Repository/entity code should be tested the same way
`src/persistence/connection.rs` is: `#[tokio::test]` functions using
`sqlite::memory:` as the connection string. That gives you a real,
disposable database per test with no setup — apply migrations, exercise
the repository, assert, done. See [`CONTRIBUTING.md`](../CONTRIBUTING.md)
for running the equivalent PostgreSQL tests locally too.
