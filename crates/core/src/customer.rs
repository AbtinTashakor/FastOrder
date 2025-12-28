use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: Uuid,
    pub phone_number: String,
    pub is_verified: bool,
}

impl Customer {
    pub fn can_order(&self) -> bool {
        self.is_verified
    }
}
