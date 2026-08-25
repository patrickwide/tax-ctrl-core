//! Persistence layer: database connection, entities, migrations, and
//! repository abstractions, all built on SeaORM.
//!
//! This module is deliberately backend-agnostic — see [`connection`] for
//! how the SQLite vs PostgreSQL choice is made at runtime.

pub mod connection;
pub mod entities;
pub mod repository;
pub mod schema;

pub use connection::{connect, connect_from_env, DbConfig};
pub use schema::Migrator;
