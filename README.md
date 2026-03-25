# seaorm-soft-delete

Soft-delete support for [SeaORM](https://www.sea-ql.org/SeaORM/) entities.

Instead of `DELETE`, soft-deleted records have `deleted_at` set to the current timestamp.
Queries exclude soft-deleted records by default.

## Installation

```toml
[dependencies]
seaorm-soft-delete = "0.1"
```

If you don't need the migration helper, disable the default feature:

```toml
seaorm-soft-delete = { version = "0.1", default-features = false }
```

## Setup

### 1. Add `deleted_at` column via migration

```rust
use seaorm_soft_delete::SoftDeleteMigration;

// inside your MigrationTrait::up()
SoftDeleteMigration::add_column(&manager, Users::Table).await?;
```

> **Note:** Uses `ADD COLUMN IF NOT EXISTS` — idempotent on **PostgreSQL only**.
> On MySQL/SQLite, guard the migration yourself.

### 2. Implement `SoftDeleteEntity` on your entity

```rust
use seaorm_soft_delete::SoftDeleteEntity;

impl SoftDeleteEntity for users::Entity {
    fn deleted_at_column() -> users::Column {
        users::Column::DeletedAt
    }
}
```

### 3. Implement `SoftDeleteActiveModel` on your active model

```rust
use seaorm_soft_delete::SoftDeleteActiveModel;
use sea_orm::Set;

impl SoftDeleteActiveModel for users::ActiveModel {
    fn set_deleted_at(&mut self, value: Option<chrono::DateTime<chrono::Utc>>) {
        self.deleted_at = Set(value.map(|v| v.into()));
    }
}
```

### 4. Implement `SoftDeleteModel` on your model (optional)

Adds `is_deleted()` directly on model instances.

```rust
use seaorm_soft_delete::SoftDeleteModel;

impl SoftDeleteModel for users::Model {
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.deleted_at.map(|v| v.into())
    }
}
```

## Querying

```rust
// Active records only (WHERE deleted_at IS NULL) — use by default
users::Entity::find_active().all(&db).await?;
users::Entity::find_active_by_id(id).one(&db).await?;

// Soft-deleted records only (WHERE deleted_at IS NOT NULL)
users::Entity::find_deleted().all(&db).await?;
users::Entity::find_deleted_by_id(id).one(&db).await?;

// All records including soft-deleted
users::Entity::find_with_deleted().all(&db).await?;
```

All return `Select<Entity>` — chain `.filter()`, `.order_by()`, `.paginate()` etc. normally.

## Write operations

```rust
let model: ActiveModel = user.into();

// Soft-delete: sets deleted_at = NOW()
model.soft_delete(&db).await?;

// Restore: sets deleted_at = NULL
model.restore(&db).await?;

// Hard delete: permanently removes the row
model.hard_delete(&db).await?;
```

## Checking deleted state

```rust
// Requires SoftDeleteModel impl (see Setup step 4)
if user.is_deleted() {
    println!("deleted at {:?}", user.deleted_at());
}
```

## Performance tip

For large tables, add a partial index so active-record queries stay fast:

```sql
CREATE INDEX idx_users_active ON users (id) WHERE deleted_at IS NULL;
```

## MSRV

Rust 1.85+

## Out of scope (v1)

- Cascading soft deletes
- Custom column names (always `deleted_at`)
- MySQL/SQLite idempotent migration

## License

MIT
