use sea_orm::{EntityTrait, Select, QueryFilter, ColumnTrait};

pub trait SoftDeleteEntity: EntityTrait {
    /// Return the column variant that maps to the `deleted_at` column for this entity.
    fn deleted_at_column() -> Self::Column;

    /// Returns a query that excludes soft-deleted records (`WHERE deleted_at IS NULL`).
    /// Use this as the default replacement for `Entity::find()`.
    fn find_active() -> Select<Self> {
        Self::find().filter(Self::deleted_at_column().is_null())
    }

    /// Returns a query that includes all records, including soft-deleted ones.
    fn find_all() -> Select<Self> {
        Self::find()
    }
}
