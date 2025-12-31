use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Cart {
    pub user_id: Uuid,
    pub items: HashMap<Uuid, CartItem>, // menu_item_id -> item
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct CartItem {
    pub menu_item_id: Uuid,
    pub quantity: u32,
}

impl Cart {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            items: HashMap::new(),
            locked: false,
        }
    }

    pub fn add_item(&mut self, menu_item_id: Uuid) -> Result<(), &'static str> {
        if self.locked {
            return Err("cart is locked");
        }

        self.items
            .entry(menu_item_id)
            .and_modify(|i| i.quantity += 1)
            .or_insert(CartItem {
                menu_item_id,
                quantity: 1,
            });

        Ok(())
    }

    pub fn remove_item(&mut self, menu_item_id: Uuid) -> Result<(), &'static str> {
        if self.locked {
            return Err("cart is locked");
        }

        if let Some(item) = self.items.get_mut(&menu_item_id) {
            if item.quantity > 1 {
                item.quantity -= 1;
            } else {
                self.items.remove(&menu_item_id);
            }
        }

        Ok(())
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }
}
