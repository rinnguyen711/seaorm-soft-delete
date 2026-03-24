use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel};
use chrono::{DateTime, Utc};
type ModelOf<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::Model;
pub trait SoftDeleteActiveModel: ActiveModelTrait + Sized {
    fn set_deleted_at(&mut self, value: Option<DateTime<Utc>>);
    async fn soft_delete(mut self, db: &DatabaseConnection) -> Result<ModelOf<Self>, DbErr>
    where
        Self: ActiveModelBehavior + Send,
        ModelOf<Self>: IntoActiveModel<Self>,
    {
        self.set_deleted_at(Some(Utc::now())); self.update(db).await
    }
    async fn restore(mut self, db: &DatabaseConnection) -> Result<ModelOf<Self>, DbErr>
    where
        Self: ActiveModelBehavior + Send,
        ModelOf<Self>: IntoActiveModel<Self>,
    {
        self.set_deleted_at(None); self.update(db).await
    }
}
