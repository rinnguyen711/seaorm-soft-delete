use chrono::{DateTime, Utc};

/// Implement this trait on your SeaORM `Model` struct to get `is_deleted()` and `deleted_at()`.
///
/// # Example
/// ```rust,ignore
/// impl SoftDeleteModel for Model {
///     fn deleted_at(&self) -> Option<DateTime<Utc>> {
///         self.deleted_at.map(|v| v.into())
///     }
/// }
/// ```
pub trait SoftDeleteModel {
    /// Return the value of the `deleted_at` field, or `None` if not deleted.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Returns `true` if this record has been soft-deleted.
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}
