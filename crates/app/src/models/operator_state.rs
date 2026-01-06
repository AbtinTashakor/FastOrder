use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorView {
    List,
    Order,
}

impl OperatorView {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperatorView::List => "LIST",
            OperatorView::Order => "ORDER",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "ORDER" => OperatorView::Order,
            _ => OperatorView::List, // safe default
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperatorState {
    pub operator_id: Uuid,
    pub is_on_shift: bool,
    pub current_view: OperatorView,
    pub current_order_id: Option<Uuid>,
}
