use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("phone number is not registered")]
    PhoneNotRegistered,

    #[error("invalid phone number")]
    InvalidPhone,

    #[error("user is not verified")]
    NotVerified,

    #[error("unexpected auth error")]
    Internal,
}
