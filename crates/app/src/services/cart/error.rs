use thiserror::Error;

#[derive(Debug, Error)]
pub enum CartError {
    #[error("no active cart found")]
    NoActiveCart,

    #[error("no confirming cart found")]
    NoConfirmingCart,

    #[error("invalid cart state")]
    InvalidState,

    #[error("internal cart error")]
    Internal,
    
    #[error("cart is locked")]
    CartLocked,
}
