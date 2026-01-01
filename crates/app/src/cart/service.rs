
use db::cart_repo::CartRepo;
use uuid::Uuid;


use super::error::CartError;
use db::models::{Cart, CartView};


#[derive(Clone)]
pub struct CartService<R: CartRepo> {
    repo: R,
}

impl<R: CartRepo> CartService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /* ───────────── State resolution ───────────── */

    pub async fn resolve_cart_state(&self, user_id: Uuid) -> Result<CartState, CartError> {
        if let Some(cart) = self
            .repo
            .find_active_cart(user_id)
            .await
            .map_err(|_| CartError::Internal)?
        {
            return Ok(CartState::Active(cart.id));
        }

        if let Some(cart) = self
            .repo
            .find_confirming_cart(user_id)
            .await
            .map_err(|_| CartError::Internal)?
        {
            return Ok(CartState::Confirming(cart.id));
        }

        // locked یا اصلاً cart نداریم → سفارش جدید
        Ok(CartState::New)
    }

    /* ───────────── Commands (by cart_id) ───────────── */
    pub async fn complete_new_cart(&self, user_id: Uuid) -> Result<Cart, CartError> {
        self.get_or_create_active_cart(user_id).await
    }

    pub async fn inc_item_by_cart(&self, cart_id: Uuid, item_id: Uuid) -> Result<(), CartError> {
        self.repo
            .inc_item(cart_id, item_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    pub async fn dec_item_by_cart(&self, cart_id: Uuid, item_id: Uuid) -> Result<(), CartError> {
        self.repo
            .dec_item(cart_id, item_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    pub async fn reset_by_cart(&self, cart_id: Uuid) -> Result<(), CartError> {
        self.repo
            .reset_cart(cart_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    pub async fn mark_confirming(&self, cart_id: Uuid) -> Result<(), CartError> {
        self.repo
            .mark_confirming(cart_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    pub async fn mark_active(&self, cart_id: Uuid) -> Result<(), CartError> {
        self.repo
            .mark_active(cart_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    /* ───────────── Views ───────────── */

    pub async fn get_cart_view(&self, cart_id: Uuid) -> Result<CartView, CartError> {
        self.repo
            .get_cart_view(cart_id)
            .await
            .map_err(|_| CartError::Internal)
    }

    pub async fn get_confirming_cart(&self, user_id: Uuid) -> Result<Cart, CartError> {
        self.repo
            .find_confirming_cart(user_id)
            .await
            .map_err(|_| CartError::Internal)?
            .ok_or(CartError::NoConfirmingCart)
    }

    pub async fn get_or_create_active_cart(&self, user_id: Uuid) -> Result<Cart, CartError> {
        if let Some(cart) = self
            .repo
            .find_active_cart(user_id)
            .await
            .map_err(|_| CartError::Internal)?
        {
            return Ok(cart);
        }

        self.repo
            .create_active_cart(user_id)
            .await
            .map_err(|_| CartError::Internal)
    }
}

/* ───────────── Cart state ───────────── */

pub enum CartState {
    Active(Uuid),
    Confirming(Uuid),
    New, // locked یا هیچ cart فعالی نیست
}
