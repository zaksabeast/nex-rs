use super::NexError;
use crate::nex_types::ResultCode;
use snafu::Snafu;

/// A convenience error for nex protocol traits that won't need an error.
/// For example, a utility trait that generates random numbers might not
/// have a reason to error.
#[derive(Debug, Snafu)]
pub enum EmptyError {}

impl NexError for EmptyError {
    fn error_code(&self) -> ResultCode {
        0.into()
    }
}

/// A convenience [Result] alias to use for nex protocol traits that won't error.
pub type SuccessfulResult<T> = Result<T, EmptyError>;
