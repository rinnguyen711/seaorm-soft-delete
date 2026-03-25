use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, IntoActiveModel};
use chrono::{DateTime, Utc};

type ModelOf<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::Model;

pub trait SoftDeleteActiveModel: ActiveModelTrait + ActiveModelBehavior + Sized {
    /// Set the `deleted_at` field to the given value.
    /// Implementors must map this to the correct active-model column setter.
    fn set_deleted_at(&mut self, value: Option<DateTime<Utc>>);

    /// Soft-delete this record by setting `deleted_at = NOW()`.
    ///
    /// Idempotent — calling this on an already-deleted record overwrites the
    /// existing timestamp with the current time.
    ///
    /// Consumes `self`. Use [`restore`] to undo.
    ///
    /// # Note
    /// This trait uses `impl Trait` in return position and is not object-safe.
    /// `Box<dyn SoftDeleteActiveModel>` will not compile.
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
    ///
    /// Idempotent — calling this on an already-active record is a no-op
    /// (sets `deleted_at` to `NULL` again).
    ///
    /// Consumes `self`.
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

    /// Permanently remove this record from the database. Cannot be undone.
    ///
    /// This does not check whether the record is soft-deleted first — it will
    /// hard-delete active records too. Use [`soft_delete`] if you want
    /// recoverable deletion.
    ///
    /// Consumes `self`.
    fn hard_delete<'a>(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + Send + 'a
    where
        Self: Send + 'a,
    {
        self.delete(db)
    }
}
