//! Migration entry point.
//!
//! Concrete migrations (creating tables, columns, indexes, etc.) are
//! registered here as they're added, following the standard
//! `sea-orm-migration` pattern documented in
//! [`tutorial-todo-list.md`](../../../docs/tutorial-todo-list.md).
//!
//! Apply pending migrations at startup or in a setup step with:
//!
//! ```ignore
//! use sea_orm_migration::MigratorTrait;
//! Migrator::up(&db, None).await?;
//! ```
//!
//! This works unmodified against either backend (SQLite or PostgreSQL)
//! since `Migrator` operates over the same runtime-resolved connection
//! used everywhere else — see [`super::connection`].

mod migrations {
    pub mod m20260826_000001_create_branch_insurance_table;
}

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Register new migrations here, in chronological order, e.g.:
            // Box::new(m20240101_000001_create_example_table::Migration),
            Box::new(migrations::m20260826_000001_create_branch_insurance_table::Migration),
        ]
    }
}
