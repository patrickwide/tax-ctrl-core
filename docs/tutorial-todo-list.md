# Tutorial: adding a feature end-to-end (using a to-do list as the example)

This walks through the full path from "no table exists" to "a working,
tested repository" — using a simple to-do list as a stand-in feature.
`tax_ctrl_core` isn't a to-do app; this is purely a worked example so the
pattern is on hand next time a real feature (tax returns, clients,
whatever) needs the same treatment. See [`USAGE.md`](USAGE.md) for the
conceptual version of this same walkthrough.

None of the code below is meant to be merged as-is — it's illustrative,
not a real module in this crate.

## 0. What we're building

A `todos` table with an id, a title, and a completed flag, plus a
`TodoRepository` with `create`, `list_pending`, and `mark_done`.

## 1. Write the migration

Migrations live under `src/persistence/migrations/` (create this folder
the first time you add one) and get registered in
[`src/persistence/schema.rs`](../src/persistence/schema.rs).

`src/persistence/migrations/m20240115_000001_create_todos_table.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Todo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Todo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Todo::Title).string().not_null())
                    .col(
                        ColumnDef::new(Todo::IsDone)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Todo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Todo {
    Table,
    Id,
    Title,
    IsDone,
}
```

This is plain `sea_query` table-building — it compiles to the right SQL
for whichever backend `Migrator` is run against, SQLite or PostgreSQL,
with no branching in this file.

You can hand-write this (as above), or scaffold the boilerplate with:

```bash
sea-orm-cli migrate generate create_todos_table
```

(see [`USAGE.md`](USAGE.md#where-sea-orm-cli-fits-in) for what
`sea-orm-cli` is and isn't for).

## 2. Register the migration

In `src/persistence/schema.rs`:

```rust
mod migrations {
    pub mod m20240115_000001_create_todos_table;
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            migrations::m20240115_000001_create_todos_table::Migration,
        )]
    }
}
```

## 3. Generate (or hand-write) the entity

With a real database available at `DATABASE_URL` and the migration
applied (`Migrator::up(&db, None).await?`, or `sea-orm-cli migrate up`),
generate the entity:

```bash
sea-orm-cli generate entity \
    --database-url "$DATABASE_URL" \
    --output-dir src/persistence/entities
```

That produces `src/persistence/entities/todo.rs`, roughly:

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "todos")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub is_done: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

Declare it in `src/persistence/entities/mod.rs`:

```rust
pub mod todo;
```

## 4. Build the repository

Compose the generic `Repository<E>` from
[`src/persistence/repository.rs`](../src/persistence/repository.rs)
rather than re-implementing find-all/find-by-id:

`src/persistence/todo_repository.rs`:

```rust
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use super::entities::todo::{self, Entity as Todo};
use super::repository::Repository;

pub struct TodoRepository<'a> {
    inner: Repository<'a, Todo>,
    conn: &'a DatabaseConnection,
}

impl<'a> TodoRepository<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self {
            inner: Repository::new(conn),
            conn,
        }
    }

    pub async fn create(&self, title: impl Into<String>) -> Result<todo::Model, DbErr> {
        todo::ActiveModel {
            title: Set(title.into()),
            is_done: Set(false),
            ..Default::default()
        }
        .insert(self.conn)
        .await
    }

    pub async fn list_pending(&self) -> Result<Vec<todo::Model>, DbErr> {
        Ok(self
            .inner
            .find_all()
            .await?
            .into_iter()
            .filter(|t| !t.is_done)
            .collect())
    }

    pub async fn mark_done(&self, id: i32) -> Result<Option<todo::Model>, DbErr> {
        let Some(existing) = self.inner.find_by_id(id).await? else {
            return Ok(None);
        };

        let mut active: todo::ActiveModel = existing.into();
        active.is_done = Set(true);
        Ok(Some(active.update(self.conn).await?))
    }
}
```

## 5. Put it together

```rust
use sea_orm_migration::MigratorTrait;
use tax_ctrl_core::persistence::{connect_from_env, Migrator};
// use tax_ctrl_core::persistence::todo_repository::TodoRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = connect_from_env().await?;
    Migrator::up(&db, None).await?;

    let repo = TodoRepository::new(&db);
    repo.create("Finish quarterly report").await?;
    repo.create("File the thing").await?;

    for todo in repo.list_pending().await? {
        println!("[ ] {}", todo.title);
    }

    Ok(())
}
```

Runs identically whether `DATABASE_URL` points at
`sqlite://tax_ctrl.db?mode=rwc` or a PostgreSQL instance.

## 6. Test it

Follow the pattern in
[`src/persistence/connection.rs`](../src/persistence/connection.rs):
an in-memory SQLite database per test, no setup required.

```rust
#[cfg(test)]
mod tests {
    use sea_orm_migration::MigratorTrait;

    use super::*;
    use crate::persistence::{connect, schema::Migrator, DbConfig};

    #[tokio::test]
    async fn creates_and_lists_pending_todos() {
        let db = connect(&DbConfig::from_url("sqlite::memory:"))
            .await
            .unwrap();
        Migrator::up(&db, None).await.unwrap();

        let repo = TodoRepository::new(&db);
        repo.create("Write tutorial").await.unwrap();

        let pending = repo.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].title, "Write tutorial");
    }
}
```

## Recap

The shape is always the same, regardless of feature:

1. Migration (schema, backend-agnostic)
2. Entity (generated or hand-written, typed model of one table)
3. Repository (the API the rest of the app calls)
4. Tests against `sqlite::memory:`

No step depends on which database is actually configured — that's the
whole point of the runtime-swappable design in
[`connection.rs`](../src/persistence/connection.rs).
