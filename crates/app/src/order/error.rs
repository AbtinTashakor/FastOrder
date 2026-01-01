use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("cart not found or not confirmable")]
    InvalidCart,

    #[error("cart is empty")]
    EmptyCart,

    #[error("database error")]
    Database,

    #[error("internal error")]
    Internal,
}
