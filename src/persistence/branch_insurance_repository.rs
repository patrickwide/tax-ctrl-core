//! Repository for the `branch_insurance` table.
//!
//! Persists `etims_vscu_wrapper::BranchInsuranceInformation` records —
//! KRA's "Branch Insurance Information" (VSCU §3.3.3.3 / OSCU §3.3.4.4)
//! — reusing the wrapper's own [`Validate`] impl before writing, so a
//! record that wouldn't pass ETIMS's own field-length rules can't be
//! persisted here either.

use etims_vscu_wrapper::BranchInsuranceInformation;
use etims_vscu_wrapper::utils::validate_fields::Validate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use super::entities::branch_insurance::{self, Entity as BranchInsurance};
use super::repository::Repository;

/// Repository for `branch_insurance` rows.
pub struct BranchInsuranceRepository<'a> {
    inner: Repository<'a, BranchInsurance>,
    conn: &'a DatabaseConnection,
}

impl<'a> BranchInsuranceRepository<'a> {
    /// Creates a repository bound to an existing connection.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self {
            inner: Repository::new(conn),
            conn,
        }
    }

    /// Validates `record` against ETIMS's own field rules
    /// (`BranchInsuranceInformation::validate`) and inserts it.
    ///
    /// # Errors
    /// Returns `DbErr::Custom` (the validation errors, joined by `; `) if
    /// `record` fails that validation, without touching the database.
    /// Otherwise propagates whatever `DbErr` SeaORM/SQLx returns.
    pub async fn create(
        &self,
        record: &BranchInsuranceInformation,
    ) -> Result<branch_insurance::Model, DbErr> {
        record
            .validate()
            .map_err(|errors| DbErr::Custom(errors.join("; ")))?;

        branch_insurance::ActiveModel {
            isrcc_cd: Set(record.isrcc_cd.clone()),
            isrcc_nm: Set(record.isrcc_nm.clone()),
            isrc_rt: Set(record.isrc_rt),
            use_yn: Set(record.use_yn.clone()),
            regr_nm: Set(record.regr_nm.clone()),
            regr_id: Set(record.regr_id.clone()),
            modr_nm: Set(record.modr_nm.clone()),
            modr_id: Set(record.modr_id.clone()),
            ..Default::default()
        }
        .insert(self.conn)
        .await
    }

    /// Looks up a branch insurance record by its `isrccCd` (insurance
    /// company code) — the business key ETIMS itself uses to identify
    /// these records. Not asserted unique at the schema level; see the
    /// migration's doc comment for why.
    pub async fn find_by_code(
        &self,
        isrcc_cd: &str,
    ) -> Result<Option<branch_insurance::Model>, DbErr> {
        BranchInsurance::find()
            .filter(branch_insurance::Column::IsrccCd.eq(isrcc_cd))
            .one(self.conn)
            .await
    }

    /// Fetches every branch insurance record.
    pub async fn list_all(&self) -> Result<Vec<branch_insurance::Model>, DbErr> {
        self.inner.find_all().await
    }
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::MigratorTrait;

    use super::*;
    use crate::persistence::{DbConfig, connect, schema::Migrator};

    async fn setup() -> DatabaseConnection {
        let db = connect(&DbConfig::from_url("sqlite::memory:"))
            .await
            .unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    /// `BranchInsuranceInformation::new` matches Python's defaults:
    /// `useYn='Y'`, audit fields all `'Admin'`.
    fn sample(isrcc_cd: &str, isrcc_nm: &str, isrc_rt: i64) -> BranchInsuranceInformation {
        BranchInsuranceInformation::new(isrcc_cd, isrcc_nm, isrc_rt)
    }

    #[tokio::test]
    async fn creates_and_finds_by_code() {
        let db = setup().await;
        let repo = BranchInsuranceRepository::new(&db);

        let created = repo
            .create(&sample("IC01", "Sample Insurer", 5))
            .await
            .unwrap();
        assert_eq!(created.isrcc_cd, "IC01");
        assert_eq!(created.isrc_rt, 5);
        assert_eq!(created.use_yn, "Y");

        let found = repo.find_by_code("IC01").await.unwrap();
        assert_eq!(found.unwrap().isrcc_nm, "Sample Insurer");
    }

    #[tokio::test]
    async fn find_by_code_returns_none_when_absent() {
        let db = setup().await;
        let repo = BranchInsuranceRepository::new(&db);

        assert!(repo.find_by_code("NOPE").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn rejects_records_that_fail_etims_validation() {
        let db = setup().await;
        let repo = BranchInsuranceRepository::new(&db);

        // isrccCd's validate_field bound is 10 chars; this is 36.
        let invalid = sample("way-too-long-for-the-ten-char-limit", "Insurer", 5);

        let result = repo.create(&invalid).await;
        assert!(matches!(result, Err(DbErr::Custom(_))));
    }

    #[tokio::test]
    async fn list_all_returns_every_row() {
        let db = setup().await;
        let repo = BranchInsuranceRepository::new(&db);

        repo.create(&sample("IC01", "First Insurer", 5))
            .await
            .unwrap();
        repo.create(&sample("IC02", "Second Insurer", 3))
            .await
            .unwrap();

        assert_eq!(repo.list_all().await.unwrap().len(), 2);
    }
}
