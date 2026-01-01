use async_trait::async_trait;
use uuid::Uuid;

use crate::models::cart::{Cart, CartView};

#[async_trait]
pub trait CartRepo: Send + Sync {
    async fn find_active_cart(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<Cart>>;

    async fn find_confirming_cart(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<Cart>>;

    async fn create_active_cart(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Cart>;

    async fn inc_item(
        &self,
        cart_id: Uuid,
        menu_item_id: Uuid,
    ) -> anyhow::Result<()>;

    async fn dec_item(
        &self,
        cart_id: Uuid,
        menu_item_id: Uuid,
    ) -> anyhow::Result<()>;

    async fn reset_cart(
        &self,
        cart_id: Uuid,
    ) -> anyhow::Result<()>;

    async fn mark_confirming(
        &self,
        cart_id: Uuid,
    ) -> anyhow::Result<()>;

    async fn mark_active(
        &self,
        cart_id: Uuid,
    ) -> anyhow::Result<()>;

    async fn get_cart_view(
        &self,
        cart_id: Uuid,
    ) -> anyhow::Result<CartView>;
}
