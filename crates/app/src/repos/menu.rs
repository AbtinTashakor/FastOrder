use async_trait::async_trait;

use crate::models::menu::MenuItem;

/// ─────────────────────────────
/// Repository contract (policy)
/// ─────────────────────────────
///
/// Read-only menu access.
/// Can be backed by DB, cache, API, etc.
#[async_trait]
pub trait MenuRepo: Send + Sync {
    async fn list_available_items(
        &self,
    ) -> anyhow::Result<Vec<MenuItem>>;
}
