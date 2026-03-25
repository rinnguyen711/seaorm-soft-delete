use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr,
    QueryFilter, Set,
};
use sea_orm_migration::prelude::*;
use seaorm_soft_delete::{SoftDeleteActiveModel, SoftDeleteEntity, SoftDeleteMigration};

// --- Minimal test entity ---

mod test_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "items")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub deleted_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// --- Trait implementations ---

impl SoftDeleteEntity for test_entity::Entity {
    fn deleted_at_column() -> test_entity::Column {
        test_entity::Column::DeletedAt
    }
}

impl SoftDeleteActiveModel for test_entity::ActiveModel {
    fn set_deleted_at(&mut self, value: Option<DateTime<Utc>>) {
        self.deleted_at = Set(value.map(|v| v.into()));
    }
}

// --- Migration ---

#[derive(Iden)]
enum Items {
    Table,
    Id,
    Name,
}

struct TestMigration;

impl MigrationName for TestMigration {
    fn name(&self) -> &str {
        "test_migration"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for TestMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Items::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Items::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Items::Name).string().not_null())
                    .to_owned(),
            )
            .await?;
        SoftDeleteMigration::add_column(manager, Items::Table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Items::Table).to_owned())
            .await
    }
}

// --- Test helpers ---

async fn setup() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    let db = Database::connect(&url).await.unwrap();
    let manager = SchemaManager::new(&db);
    TestMigration.down(&manager).await.ok();
    TestMigration.up(&manager).await.unwrap();
    db
}

// --- Tests ---

#[tokio::test]
async fn test_find_active_excludes_soft_deleted() {
    let db = setup().await;

    let a = test_entity::ActiveModel {
        name: Set("alpha".into()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let b = test_entity::ActiveModel {
        name: Set("beta".into()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    test_entity::ActiveModel {
        id: Set(a.id),
        ..Default::default()
    }
    .soft_delete(&db)
    .await
    .unwrap();

    let active = test_entity::Entity::find_active().all(&db).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "beta");

    let _ = b; // suppress unused warning
}

#[tokio::test]
async fn test_find_all_includes_soft_deleted() {
    let db = setup().await;

    let a = test_entity::ActiveModel {
        name: Set("gamma".into()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    test_entity::ActiveModel {
        id: Set(a.id),
        ..Default::default()
    }
    .soft_delete(&db)
    .await
    .unwrap();

    let all = test_entity::Entity::find_all().all(&db).await.unwrap();
    assert!(all.iter().any(|r| r.id == a.id));
}

#[tokio::test]
async fn test_restore_clears_deleted_at() {
    let db = setup().await;

    let a = test_entity::ActiveModel {
        name: Set("delta".into()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    test_entity::ActiveModel {
        id: Set(a.id),
        ..Default::default()
    }
    .soft_delete(&db)
    .await
    .unwrap();

    let restored = test_entity::ActiveModel {
        id: Set(a.id),
        ..Default::default()
    }
    .restore(&db)
    .await
    .unwrap();

    assert!(restored.deleted_at.is_none());

    let active = test_entity::Entity::find_active()
        .filter(test_entity::Column::Id.eq(a.id))
        .one(&db)
        .await
        .unwrap();
    assert!(active.is_some());
}

#[tokio::test]
async fn test_soft_delete_sets_deleted_at_timestamp() {
    let db = setup().await;

    let a = test_entity::ActiveModel {
        name: Set("epsilon".into()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let deleted = test_entity::ActiveModel {
        id: Set(a.id),
        ..Default::default()
    }
    .soft_delete(&db)
    .await
    .unwrap();

    assert!(deleted.deleted_at.is_some());
}
