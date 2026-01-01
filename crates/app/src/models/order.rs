use uuid::Uuid;
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_day: NaiveDate,
    pub daily_no: i32,
    pub order_code: String,
    pub total_price: i64,
    pub status: OrderStatus,
    pub prep_time_minutes: Option<i32>,
    pub created_at: NaiveDateTime
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Accepted,
    Rejected,
    Completed,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Accepted => "accepted",
            OrderStatus::Rejected => "rejected",
            OrderStatus::Completed => "completed",
        }
    }
}
