use super::NexError;
use crate::nex_types::ResultCode;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum EmptyError {}

impl NexError for EmptyError {
    fn error_code(&self) -> ResultCode {
        0.into()
    }
}

pub type SuccessfulResult<T> = Result<T, EmptyError>;
