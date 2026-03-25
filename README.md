# seaorm-soft-delete

Soft-delete support for [SeaORM](https://www.sea-ql.org/SeaORM/) entities.

Instead of `DELETE`, soft-deleted records have `deleted_at` set to the current timestamp.
Queries exclude soft-deleted records by default.

## Installation

```toml
[dependencies]
seaorm-soft-delete = "0.1"
```

## Usage

### 1. Add `deleted_at` column via migration

```rust
use seaorm_soft_delete::SoftDeleteMigration;

// inside your MigrationTrait::up()
SoftDeleteMigration::add_column(&mut manager, Users::Table).await?;
```

> **Note:** `add_column` uses `IF NOT EXISTS` — idempotent on PostgreSQL only.
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

This unlocks:

```rust
// Excludes soft-deleted records (WHERE deleted_at IS NULL) — use this by default
users::Entity::find_active().all(&db).await?;

// Includes soft-deleted records
users::Entity::find_all().all(&db).await?;
```

Both return `Select<Entity>` — chain `.filter()`, `.order_by()`, etc. normally.

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

This unlocks:

```rust
// Soft-delete (sets deleted_at = NOW())
let active_model = user.into_active_model();
active_model.soft_delete(&db).await?;

// Restore (sets deleted_at = NULL)
let active_model = user.into_active_model();
active_model.restore(&db).await?;
```

Both consume `self` and return `Result<Model, DbErr>`.

## Performance tip

For large tables, add a partial index on `deleted_at IS NULL`:

```sql
CREATE INDEX idx_users_active ON users (id) WHERE deleted_at IS NULL;
```

## MSRV

Rust 1.85+

## Out of scope (v1)

- Cascading soft deletes
- Custom column names (always `deleted_at`)
- MySQL/SQLite idempotent migration
