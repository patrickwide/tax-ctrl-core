//! Generated SeaORM entity definitions live in this module.
//!
//! No tables exist yet, so this module is currently empty. Once
//! migrations are added to [`super::schema`], entities can either be
//! hand-written or generated with:
//!
//! ```text
//! sea-orm-cli generate entity \
//!     --database-url "$DATABASE_URL" \
//!     --output-dir src/persistence/entities
//! ```
//!
//! A hand-written entity follows this shape:
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//!
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//! #[sea_orm(table_name = "example")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: i32,
//!     pub name: String,
//! }
//!
//! #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! pub enum Relation {}
//!
//! impl ActiveModelBehavior for ActiveModel {}
//! ```
//!
//! Once entities exist, declare them here, e.g. `pub mod example;`.
