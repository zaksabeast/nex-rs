use crate::nex_types::ResultCode;
use std::fmt::{Debug, Display};

/// A trait that represents a nex error.
/// In the event of an error, [NexError::error_code] will be
/// sent to the 3ds.
pub trait NexError: Debug + Display + Send {
    fn error_code(&self) -> ResultCode;
}

pub type NexResult<T> = Result<T, Box<dyn NexError>>;
