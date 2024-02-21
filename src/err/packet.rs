use thiserror::Error;

use super::{IpmiHeaderError, IpmiPayloadError, RMCPHeaderError};

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Length of slice too small")]
    WrongLength,
    #[error("Failed to parse slice to Packet")]
    FailedToParse,
    #[error("Failed to parse slice to RMCP Header")]
    Rmcp(#[from] RMCPHeaderError),
    #[error("Failed to parse slice to Ipmi Header")]
    Ipmi(#[from] IpmiHeaderError),
    #[error("Failed to parse slice to Ipmi Header")]
    IPMIPayload(#[from] IpmiPayloadError),
}
