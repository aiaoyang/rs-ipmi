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
