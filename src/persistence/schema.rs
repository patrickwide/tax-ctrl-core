//! Migration entry point.
//!
//! Concrete migrations (creating tables, columns, indexes, etc.) are
//! registered here as they're added. No tables are defined yet — this is
//! the wiring needed to add them incrementally, following the standard
//! `sea-orm-migration` pattern:
//!
//! ```ignore
//! mod m20240101_000001_create_example_table;
//!
//! fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!     vec![Box::new(m20240101_000001_create_example_table::Migration)]
//! }
//! ```
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

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Register new migrations here as they're written, e.g.:
            // Box::new(m20240101_000001_create_example_table::Migration),
        ]
    }
}
