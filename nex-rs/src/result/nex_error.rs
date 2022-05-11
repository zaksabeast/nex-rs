use crate::nex_types::ResultCode;
use std::fmt::{Debug, Display};

pub trait NexError: Debug + Display + Send {
    fn error_code(&self) -> ResultCode;
}

pub type NexResult<T> = Result<T, Box<dyn NexError>>;
