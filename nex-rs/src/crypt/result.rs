use crypto::symmetriccipher::SymmetricCipherError;
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu()]
    InvalidLength,
    #[snafu()]
    InvalidPadding,
    #[snafu()]
    InvalidKeySize,
    #[snafu()]
    InvalidChecksum,
}

impl From<SymmetricCipherError> for Error {
    fn from(error: SymmetricCipherError) -> Self {
        match error {
            SymmetricCipherError::InvalidLength => Self::InvalidLength,
            SymmetricCipherError::InvalidPadding => Self::InvalidPadding,
        }
    }
}

pub type CryptResult<T> = Result<T, Error>;
