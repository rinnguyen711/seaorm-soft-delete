use sea_orm::DbErr;
use sea_orm_migration::{prelude::Iden, SchemaManager, sea_query::{Alias, ColumnDef, Table}};

/// The column name used for soft-delete timestamps. Always `deleted_at`.
pub const SOFT_DELETE_COLUMN: &str = "deleted_at";

pub struct SoftDeleteMigration;

impl SoftDeleteMigration {
    /// Add a nullable `deleted_at TIMESTAMPTZ` column to an existing table.
    ///
    /// # Idempotency
    /// Uses `IF NOT EXISTS` — idempotent on **PostgreSQL only**.
    /// On MySQL or SQLite, calling this on a table that already has `deleted_at`
    /// will return an error. Guard the migration yourself on those databases.
    pub async fn add_column<T: Iden + 'static>(
        manager: &SchemaManager<'_>,
        table: T,
    ) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new(SOFT_DELETE_COLUMN))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }
}
