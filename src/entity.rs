use sea_orm::{ColumnTrait, EntityTrait, PrimaryKeyTrait, QueryFilter, Select};

pub trait SoftDeleteEntity: EntityTrait {
    /// Return the column variant that maps to the `deleted_at` column for this entity.
    fn deleted_at_column() -> Self::Column;

    /// Returns a query excluding soft-deleted records (`WHERE deleted_at IS NULL`).
    /// Use this as the default replacement for `Entity::find()`.
    #[must_use]
    fn find_active() -> Select<Self> {
        Self::find().filter(Self::deleted_at_column().is_null())
    }

    /// Returns a query filtered by primary key, excluding soft-deleted records.
    /// Use this as the default replacement for `Entity::find_by_id()`.
    #[must_use]
    fn find_active_by_id<T>(id: T) -> Select<Self>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        Self::find_by_id(id).filter(Self::deleted_at_column().is_null())
    }

    /// Returns a query including only soft-deleted records (`WHERE deleted_at IS NOT NULL`).
    /// Useful for admin views, audit logs, or restore workflows.
    #[must_use]
    fn find_deleted() -> Select<Self> {
        Self::find().filter(Self::deleted_at_column().is_not_null())
    }

    /// Returns a query filtered by primary key, including only soft-deleted records.
    #[must_use]
    fn find_deleted_by_id<T>(id: T) -> Select<Self>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        Self::find_by_id(id).filter(Self::deleted_at_column().is_not_null())
    }

    /// Returns a query including all records, both active and soft-deleted.
    #[must_use]
    fn find_with_deleted() -> Select<Self> {
        Self::find()
    }
}
