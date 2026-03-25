use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Set,
};
use sea_orm_migration::prelude::*;
use seaorm_soft_delete::{SoftDeleteActiveModel, SoftDeleteEntity, SoftDeleteMigration, SoftDeleteModel};

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

impl SoftDeleteModel for test_entity::Model {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at.map(|v| v.into())
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

async fn insert(db: &DatabaseConnection, name: &str) -> test_entity::Model {
    test_entity::ActiveModel {
        name: Set(name.into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap()
}

// --- Tests ---

#[tokio::test]
async fn test_find_active_excludes_soft_deleted() {
    let db = setup().await;

    let a = insert(&db, "alpha").await;
    let _b = insert(&db, "beta").await;

    test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    let active = test_entity::Entity::find_active().all(&db).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "beta");
}

#[tokio::test]
async fn test_find_with_deleted_includes_soft_deleted() {
    let db = setup().await;

    let a = insert(&db, "gamma").await;

    test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    let all = test_entity::Entity::find_with_deleted().all(&db).await.unwrap();
    assert!(all.iter().any(|r| r.id == a.id));
}

#[tokio::test]
async fn test_restore_clears_deleted_at() {
    let db = setup().await;

    let a = insert(&db, "delta").await;

    test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    let restored = test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .restore(&db)
        .await
        .unwrap();

    assert!(restored.deleted_at.is_none());

    let found = test_entity::Entity::find_active()
        .filter(test_entity::Column::Id.eq(a.id))
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_soft_delete_sets_deleted_at_timestamp() {
    let db = setup().await;

    let a = insert(&db, "epsilon").await;

    let deleted = test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    assert!(deleted.deleted_at.is_some());
}

#[tokio::test]
async fn test_find_deleted_returns_only_soft_deleted() {
    let db = setup().await;

    let a = insert(&db, "zeta").await;
    let _b = insert(&db, "eta").await;

    test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    let deleted = test_entity::Entity::find_deleted().all(&db).await.unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].id, a.id);
}

#[tokio::test]
async fn test_find_deleted_by_id() {
    let db = setup().await;

    let a = insert(&db, "theta").await;

    test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();

    let found = test_entity::Entity::find_deleted_by_id(a.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());

    let not_found = test_entity::Entity::find_active_by_id(a.id)
        .one(&db)
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_hard_delete_removes_row() {
    let db = setup().await;

    let a = insert(&db, "iota").await;
    let id = a.id;

    test_entity::ActiveModel { id: Set(id), ..Default::default() }
        .hard_delete(&db)
        .await
        .unwrap();

    let found = test_entity::Entity::find_with_deleted()
        .filter(test_entity::Column::Id.eq(id))
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_is_deleted_reflects_state() {
    let db = setup().await;

    let a = insert(&db, "kappa").await;
    assert!(!a.is_deleted());

    let deleted = test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .soft_delete(&db)
        .await
        .unwrap();
    assert!(deleted.is_deleted());

    let restored = test_entity::ActiveModel { id: Set(a.id), ..Default::default() }
        .restore(&db)
        .await
        .unwrap();
    assert!(!restored.is_deleted());
}
