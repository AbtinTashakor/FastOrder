use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Accepted { prep_time_minutes: u32 },
    Rejected,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub title: String,
    pub price: i64,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub items: Vec<OrderItem>,
    pub total_price: i64,
    pub status: OrderStatus,
}

impl Order {
    pub fn accept(&mut self, prep_time_minutes: u32) -> Result<(), &'static str> {
        match self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Accepted { prep_time_minutes };
                Ok(())
            }
            _ => Err("order cannot be accepted"),
        }
    }

    pub fn reject(&mut self) -> Result<(), &'static str> {
        match self.status {
            OrderStatus::Pending => {
                self.status = OrderStatus::Rejected;
                Ok(())
            }
            _ => Err("order cannot be rejected"),
        }
    }
}
