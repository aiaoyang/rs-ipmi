use std::{io, num::TryFromIntError};

use thiserror::Error as ThisError;

use crate::{err::Error, rmcp::open_session::StatusCode, CompletionCode};

#[derive(ThisError, Debug)]
pub enum EClient {
    #[error("Failed to bind due to: {0}")]
    FailedBind(#[source] io::Error),
    #[error("Failed to connect to IPMI Server due to: {0}")]
    ConnectToIPMIServer(#[source] io::Error),
    #[error("Failed to set the socket read timeout: {0}")]
    SetReadTimeOutError(#[source] io::Error),
    #[error("Failed to send packet due to: {0}")]
    FailedSend(#[source] io::Error),
    #[error("Failed to set the socket read timeout: {0}")]
    FailedSetSocketReadTimeout(#[from] io::Error),
    #[error("Didn't recieve a response from remote controller")]
    NoResponse,
    #[error("Received incorrect payload type from remote controller")]
    MisformedResponse,
    #[error("This library does not support IPMI v1.5")]
    UnsupportedVersion,
    #[error("Error from BMC when opening rmcp+ session: {0:?}")]
    FailedToOpenSession(StatusCode),
    #[error("Failed to validate key exchange auth code")]
    MismatchedKeyExchangeAuthCode,
    #[error("Failed to validate RAKP Message 2. This could be due to an incorrect password.")]
    FailedToValidateRAKP2,
    #[error("Username too long")]
    UsernameOver255InLength(#[from] TryFromIntError),
    #[error("Session not established yet")]
    SessionNotEstablishedYet,

    #[error("CompletionCode: {0:?}")]
    CompletionCode(#[from] CompletionCode),
}

impl From<EClient> for Error {
    fn from(value: EClient) -> Self {
        Self::Client(value)
    }
}
