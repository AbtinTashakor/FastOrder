use db::{menu_repo::PgMenuRepo, models::MenuItem};

use crate::menu::error::MenuError;

#[derive(Clone)]
pub struct MenuService {
    repo: PgMenuRepo,
}

impl MenuService {
    pub fn new(repo: PgMenuRepo) -> Self {
        Self { repo }
    }

    pub async fn get_menu_items(&self) -> Result<Vec<MenuItem>, MenuError> {
        let rows = self
            .repo
            .list_available_items()
            .await
            .map_err(|_| MenuError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| MenuItem {
                id: r.id,
                category_id: r.category_id,
                title: r.title,
                price: r.price,
                is_available: true, // چون فقط active ها query شدن
                category_title: r.category_title,
                position: r.position,
            })
            .collect())
    }
}
