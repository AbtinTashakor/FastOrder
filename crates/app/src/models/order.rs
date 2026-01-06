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

    // operator-related (future-proof)
    pub operator_id: Option<Uuid>,
    pub assigned_at: Option<NaiveDateTime>,
    pub seen_at: Option<NaiveDateTime>,

    pub prep_time_minutes: Option<i32>,
    pub retry_count: i32,

    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    PendingAssign,
    AssignedUnseen,
    AssignedInReview,
    WaitingForOperator,
    Accepted,
    Rejected,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::PendingAssign => "PENDING_ASSIGN",
            OrderStatus::AssignedUnseen => "ASSIGNED_UNSEEN",
            OrderStatus::AssignedInReview => "ASSIGNED_IN_REVIEW",
            OrderStatus::WaitingForOperator => "WAITING_FOR_OPERATOR",
            OrderStatus::Accepted => "ACCEPTED",
            OrderStatus::Rejected => "REJECTED",
        }
    }
}

impl TryFrom<&str> for OrderStatus {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PENDING_ASSIGN" => Ok(OrderStatus::PendingAssign),
            "ASSIGNED_UNSEEN" => Ok(OrderStatus::AssignedUnseen),
            "ASSIGNED_IN_REVIEW" => Ok(OrderStatus::AssignedInReview),
            "WAITING_FOR_OPERATOR" => Ok(OrderStatus::WaitingForOperator),
            "ACCEPTED" => Ok(OrderStatus::Accepted),
            "REJECTED" => Ok(OrderStatus::Rejected),
            other => Err(anyhow::anyhow!("unknown order status: {}", other)),
        }
    }
}
