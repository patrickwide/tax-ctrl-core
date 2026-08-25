//! Persistence layer: database connection, entities, migrations, and
//! repository abstractions, all built on SeaORM.
//!
//! This module is deliberately backend-agnostic — see [`connection`] for
//! how the SQLite vs PostgreSQL choice is made at runtime.
//!
//! See [`etims_types`] for the mapping from `etims-vscu-wrapper`'s Rust
//! data types onto SeaORM-compatible SQL column types, used when writing
//! migrations/entities for ETIMS-backed tables.
//!
//! [`branch_insurance_repository`] is the first concrete table built on
//! that mapping: it persists `etims_vscu_wrapper::BranchInsuranceInformation`
//! records via the `branch_insurance` table (migration under [`schema`],
//! entity under [`entities::branch_insurance`]).

pub mod branch_insurance_repository;
pub mod connection;
pub mod entities;
pub mod etims_types;
mod migrations;
pub mod repository;
pub mod schema;

pub use connection::{DbConfig, connect, connect_from_env};
pub use etims_types::EtimsColumnType;
pub use schema::Migrator;
