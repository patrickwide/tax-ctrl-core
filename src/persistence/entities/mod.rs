//! Generated SeaORM entity definitions live in this module.
//!
//! Once a migration is added to [`super::schema`], entities can either be
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
//! [`branch_insurance`] is the first hand-written entity following this
//! shape.

pub mod branch_insurance;
