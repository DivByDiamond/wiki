pub mod access;
pub mod pages;
pub mod proposals;

#[derive(Debug)]
pub struct StoreError(pub String);

impl From<StoreError> for crate::error::AppError {
    fn from(e: StoreError) -> Self {
        crate::error::AppError::Internal(e.0)
    }
}

pub type StoreResult<T> = Result<T, StoreError>;
