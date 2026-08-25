//! Persistence layer: database connection, entities, migrations, and
//! repository abstractions, all built on SeaORM.
//!
//! This module is deliberately backend-agnostic — see [`connection`] for
//! how the SQLite vs PostgreSQL choice is made at runtime.

pub mod connection;
pub mod entities;
pub mod repository;
pub mod schema;

pub use connection::{DbConfig, connect, connect_from_env};
pub use schema::Migrator;
