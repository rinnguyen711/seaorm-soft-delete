use sea_orm::{EntityTrait, Select, QueryFilter, ColumnTrait};

pub trait SoftDeleteEntity: EntityTrait {
    fn deleted_at_column() -> Self::Column;
    fn find_active() -> Select<Self> { Self::find().filter(Self::deleted_at_column().is_null()) }
    fn find_all() -> Select<Self> { Self::find() }
}
