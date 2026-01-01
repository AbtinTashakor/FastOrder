use thiserror::Error;

#[derive(Debug, Error)]
pub enum MenuError {
    #[error("database error")]
    Database,

    #[error("internal error")]
    Internal,
}
