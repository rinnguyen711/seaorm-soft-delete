use sea_orm::{EntityTrait, PrimaryKeyTrait, Select, QueryFilter, ColumnTrait};

pub trait SoftDeleteEntity: EntityTrait {
    /// Return the column variant that maps to the `deleted_at` column for this entity.
    fn deleted_at_column() -> Self::Column;

    /// Returns a query that excludes soft-deleted records (`WHERE deleted_at IS NULL`).
    /// Use this as the default replacement for `Entity::find()`.
    fn find_active() -> Select<Self> {
        Self::find().filter(Self::deleted_at_column().is_null())
    }

    /// Returns a query filtered by primary key that excludes soft-deleted records.
    /// Use this as the default replacement for `Entity::find_by_id()`.
    fn find_active_by_id<T>(id: T) -> Select<Self>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        Self::find_by_id(id).filter(Self::deleted_at_column().is_null())
    }

    /// Returns a query that includes only soft-deleted records (`WHERE deleted_at IS NOT NULL`).
    fn find_deleted() -> Select<Self> {
        Self::find().filter(Self::deleted_at_column().is_not_null())
    }

    /// Returns a query filtered by primary key that includes only soft-deleted records.
    fn find_deleted_by_id<T>(id: T) -> Select<Self>
    where
        T: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        Self::find_by_id(id).filter(Self::deleted_at_column().is_not_null())
    }

    /// Returns a query that includes all records, including soft-deleted ones.
    fn find_all() -> Select<Self> {
        Self::find()
    }
}
