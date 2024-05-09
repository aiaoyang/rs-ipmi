use std::io;

pub use ipmi_client::*;
pub use packet::*;

mod ipmi_client;
mod packet;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Client: {0}")]
    Client(EClient),
    #[error("Packet: {0}")]
    Packet(EPacket),
    #[error("Crypto: {0}")]
    Crypto(ECrypto),
    #[error("Try from slice: {0}")]
    TryFromSlice(std::array::TryFromSliceError),

    #[error("Try from u8: {0}")]
    TryFromU8(u8),
    
    #[error("io: {0}")]
    Io(std::io::Error),

    #[error("timeout")]
    Timeout(tokio::time::error::Elapsed),

    #[error("other: {0}")]
    RawString(String)
}

#[derive(ThisError, Debug)]
pub enum ECrypto {
    #[error("Unpad: {0}")]
    Unpad(aes::cipher::block_padding::UnpadError),
}

impl From<ECrypto> for Error {
    fn from(value: ECrypto) -> Self {
        Self::Crypto(value)
    }
}

impl From<aes::cipher::block_padding::UnpadError> for Error {
    fn from(value: aes::cipher::block_padding::UnpadError) -> Self {
        Self::Crypto(ECrypto::Unpad(value))
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(value: std::array::TryFromSliceError) -> Self {
        Self::TryFromSlice(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(value: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(value)
    }
}