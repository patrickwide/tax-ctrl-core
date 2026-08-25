//! Generic repository abstractions built on top of SeaORM.
//!
//! `Repository<E>` provides basic find operations for any SeaORM entity.
//! Concrete, domain-specific repositories (e.g. a future
//! `TaxReturnRepository`) should be added alongside entities as they are
//! introduced, composing this type rather than re-implementing basic
//! find/list logic for every entity.

use sea_orm::{DatabaseConnection, DbErr, EntityTrait, PrimaryKeyTrait};

/// A minimal, generic repository over a single SeaORM entity `E`.
pub struct Repository<'a, E: EntityTrait> {
    conn: &'a DatabaseConnection,
    _entity: std::marker::PhantomData<E>,
}

impl<'a, E: EntityTrait> Repository<'a, E> {
    /// Create a repository bound to an existing connection.
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self {
            conn,
            _entity: std::marker::PhantomData,
        }
    }

    /// Find a single record by its primary key.
    pub async fn find_by_id(
        &self,
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find_by_id(id).one(self.conn).await
    }

    /// Fetch every record for this entity.
    pub async fn find_all(&self) -> Result<Vec<E::Model>, DbErr> {
        E::find().all(self.conn).await
    }
}
