//! SeaORM entity for the `branch_insurance` table.
//!
//! Mirrors the columns defined in
//! [`m20260826_000001_create_branch_insurance_table`](super::super::schema),
//! which in turn mirror `etims_vscu_wrapper::BranchInsuranceInformation`
//! — see `docs/etims-type-mapping.md` for the field-by-field mapping.
//!
//! Hand-written rather than `sea-orm-cli`-generated, per
//! `CONTRIBUTING.md`'s guidance for small, stable tables.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "branch_insurance")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub isrcc_cd: String,
    pub isrcc_nm: String,
    pub isrc_rt: i64,
    pub use_yn: String,
    pub regr_nm: String,
    pub regr_id: String,
    pub modr_nm: String,
    pub modr_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
