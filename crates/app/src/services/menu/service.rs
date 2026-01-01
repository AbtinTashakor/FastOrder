use crate::services::menu::error::MenuError;
use crate::models::menu::MenuItem;
use crate::repos::menu::MenuRepo;

/// ─────────────────────────────
/// Menu service (use-cases)
/// ─────────────────────────────

#[derive(Clone)]
pub struct MenuService<R: MenuRepo + Clone> {
    repo: R,
}

impl<R: MenuRepo + Clone> MenuService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Get all available menu items for customer view
    pub async fn get_menu_items(
        &self,
    ) -> Result<Vec<MenuItem>, MenuError> {
        self.repo
            .list_available_items()
            .await
            .map_err(|_| MenuError::Database)
    }
}
