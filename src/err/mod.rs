use std::io;

pub use ipmi_client::*;
pub use packet::*;

mod ipmi_client;
mod packet;

use thiserror::Error as ThisError;

use crate::CompletionCode;

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
    RawString(String),
}

impl From<Error> for f64 {
    fn from(value: Error) -> Self {
        match value {
            Error::Client(_) => 100_f64,
            Error::Packet(_) => 200_f64,
            Error::Crypto(_) => 300_f64,
            Error::TryFromSlice(_) => 400_f64,
            Error::TryFromU8(_) => 500_f64,
            Error::Io(_) => 600_f64,
            Error::Timeout(_) => 700_f64,
            Error::RawString(_) => 800_f64,
        }
    }
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

impl Error {
    pub fn is_rq_sensor_data_record_not_present(&self) -> bool {
        matches!(self, Error::Client(EClient::CompletionCode((
                _,
                CompletionCode::RqSensorDataRecordNotPresent,
            ))))
    }
}
