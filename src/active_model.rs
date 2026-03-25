use sea_orm::{ActiveModelTrait, ActiveModelBehavior, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel};
use chrono::{DateTime, Utc};
use std::future::Future;

type ModelOf<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::Model;

pub trait SoftDeleteActiveModel: ActiveModelTrait + ActiveModelBehavior + Sized {
    /// Set the `deleted_at` field to the given value.
    /// Implementors must map this to the correct active-model column setter.
    fn set_deleted_at(&mut self, value: Option<DateTime<Utc>>);

    /// Soft-delete this record by setting `deleted_at = NOW()`.
    /// Consumes `self` — the active model cannot be reused after this call.
    fn soft_delete<'a>(
        mut self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<ModelOf<Self>, DbErr>> + Send + 'a
    where
        Self: Send + 'a,
        ModelOf<Self>: IntoActiveModel<Self>,
    {
        self.set_deleted_at(Some(Utc::now()));
        self.update(db)
    }

    /// Restore a soft-deleted record by setting `deleted_at = NULL`.
    /// Consumes `self` — the active model cannot be reused after this call.
    fn restore<'a>(
        mut self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<ModelOf<Self>, DbErr>> + Send + 'a
    where
        Self: Send + 'a,
        ModelOf<Self>: IntoActiveModel<Self>,
    {
        self.set_deleted_at(None);
        self.update(db)
    }
}
