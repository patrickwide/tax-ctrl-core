//! Individual migration modules, registered in [`super::schema`].
//!
//! Kept as a sibling of `schema.rs` (rather than nested inside it) so
//! each migration file's path matches what
//! [`CONTRIBUTING.md`](../../../CONTRIBUTING.md#adding-a-migration)
//! documents: `src/persistence/migrations/m<timestamp>_<name>.rs`.

pub mod m20260826_000001_create_branch_insurance_table;
