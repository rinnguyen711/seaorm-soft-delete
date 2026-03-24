use sea_orm::DbErr;
use sea_orm_migration::{prelude::Iden, SchemaManager, sea_query::{Alias, ColumnDef, Expr, Table}};

pub struct SoftDeleteMigration;
impl SoftDeleteMigration {
    pub async fn add_column<T: Iden + 'static>(manager: &SchemaManager<'_>, table: T) -> Result<(), DbErr> {
        manager.alter_table(Table::alter().table(table).add_column_if_not_exists(
            ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null().default(Expr::cust("NULL"))
        ).to_owned()).await
    }
}
