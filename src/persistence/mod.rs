//! Persistence layer: database connection, entities, migrations, and
//! repository abstractions, all built on SeaORM.
//!
//! This module is deliberately backend-agnostic — see [`connection`] for
//! how the SQLite vs PostgreSQL choice is made at runtime.
//!
//! See [`etims_types`] for the mapping from `etims-vscu-wrapper`'s Rust
//! data types onto SeaORM-compatible SQL column types, used when writing
//! migrations/entities for ETIMS-backed tables.

pub mod connection;
pub mod entities;
pub mod etims_types;
pub mod repository;
pub mod schema;

pub use connection::{DbConfig, connect, connect_from_env};
pub use etims_types::EtimsColumnType;
pub use schema::Migrator;
