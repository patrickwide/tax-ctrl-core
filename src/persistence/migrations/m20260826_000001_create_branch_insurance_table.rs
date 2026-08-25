//! Creates the `branch_insurance` table.
//!
//! This persists `etims_vscu_wrapper::BranchInsuranceInformation` records
//! — KRA's "Branch Insurance Information" (VSCU §3.3.3.3 / OSCU §3.3.4.4)
//! — and is the first concrete table built on the mapping documented in
//! [`crate::persistence::etims_types`] / `docs/etims-type-mapping.md`.
//!
//! Column choices aren't arbitrary:
//! - `isrcc_cd` (10), `isrcc_nm` (100), `use_yn` (1), and the four audit
//!   fields `regr_nm`/`modr_nm` (60) and `regr_id`/`modr_id` (20) use the
//!   exact `string_len` bounds from
//!   `BranchInsuranceInformation::validate`'s `validate_field` calls —
//!   the per-field-max-length source of truth for this struct.
//! - `isrc_rt` is `big_integer`, matching `EtimsColumnType`'s `i64` →
//!   `ColumnType::BigInteger` mapping.
//!
//! `isrcc_cd` is deliberately not declared `UNIQUE` here: ETIMS treats it
//! as a business key, but nothing in the wrapper's own validation
//! enforces uniqueness, and KRA's API is the actual source of truth for
//! whether a code is a duplicate — adding a DB-level uniqueness
//! constraint this table can't actually verify against would be
//! misleading. `BranchInsuranceRepository::find_by_code` provides
//! lookup by that key without asserting exclusivity.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BranchInsurance::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BranchInsurance::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::IsrccCd)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::IsrccNm)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::IsrcRt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::UseYn)
                            .string_len(1)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::RegrNm)
                            .string_len(60)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::RegrId)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::ModrNm)
                            .string_len(60)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BranchInsurance::ModrId)
                            .string_len(20)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BranchInsurance::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BranchInsurance {
    Table,
    Id,
    IsrccCd,
    IsrccNm,
    IsrcRt,
    UseYn,
    RegrNm,
    RegrId,
    ModrNm,
    ModrId,
}
