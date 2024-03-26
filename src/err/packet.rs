use thiserror::Error as ThisError;

use crate::{commands::CommandCode, err::Error, CompletionCode};

#[derive(ThisError, Debug)]
pub enum EPacket {
    #[error("Length of slice too small, at least 20 bytes")]
    WrongLength,
    #[error("Failed to parse slice to RMCP Header: {0}")]
    RmcpHeader(#[from] ERMCPHeader),
    #[error("Failed to parse slice to Ipmi Header: {0}")]
    IpmiHeader(#[from] EIpmiHeader),
    #[error("Failed to parse slice to Ipmi Payload: {0}")]
    IpmiPayload(#[from] EIpmiPayload),
    #[error("Unknown Reason Failed to parse slice to Packet")]
    Unknown,
}

impl From<EPacket> for Error {
    fn from(value: EPacket) -> Self {
        Self::Packet(value)
    }
}

#[derive(ThisError, Debug)]
pub enum ERMCPHeader {
    #[error("Rmcp Header should be 4 bytes")]
    WrongLength,
    #[error("Failed to parse slice to rmcp header")]
    FailedToParse,
    #[error("Unsupported Message class: {0}")]
    UnsupportedMessageClass(u8),
}

impl From<ERMCPHeader> for Error {
    fn from(value: ERMCPHeader) -> Self {
        Self::Packet(EPacket::RmcpHeader(value))
    }
}

#[derive(ThisError, Debug)]
pub enum EIpmiHeader {
    #[error("Unsupported Auth Type: {0}")]
    UnsupportedAuthType(u8),
    #[error("Failed parsing IPMI v1.5 header: {0}")]
    V1(#[from] EV1Header),
    #[error("Failed parsing IPMI v2 header: {0}")]
    V2(#[from] EV2Header),
}

impl From<EIpmiHeader> for Error {
    fn from(value: EIpmiHeader) -> Self {
        Self::Packet(EPacket::IpmiHeader(value))
    }
}

#[derive(ThisError, Debug)]
pub enum EV2Header {
    #[error("Ipmi V2 Header should be either 12 or 18 bytes")]
    WrongLength,

    #[error("Unsupported Payload Type: {0}")]
    UnsupportedPayloadType(u8),
}

#[derive(ThisError, Debug)]
pub enum EV1Header {
    #[error("Ipmi V1 Header should be either 10 or 26 bytes")]
    WrongLength,
}

#[derive(ThisError, Debug)]
pub enum EIpmiPayload {
    #[error("IpmiPayload should be at least 7 bytes")]
    WrongLength,
    #[error("CompletionCode unsucceed: {0:?}")]
    CompletionCode(CompletionCode),
    #[error("Command Error: {0}")]
    Command(ECommand),
}

impl From<EIpmiPayload> for Error {
    fn from(value: EIpmiPayload) -> Self {
        Self::Packet(EPacket::IpmiPayload(value))
    }
}

#[derive(ThisError, Debug)]
pub enum ECommand {
    #[error("unknown netfn {0}")]
    UnknownNetFnError(u8),
    #[error("unknown command code {0}")]
    UnknownCommandCode(u8),
    #[error("unknown lun {0}")]
    UnknownLun(u8),
    #[error("unknown privilege {0}")]
    UnknownPrivilege(u8),
    #[error("unknown auth algorithm {0}")]
    UnknownAuthAlgorithm(u8),
    #[error("unknown integrity algorithm {0}")]
    UnknownIntegrityAlgorithm(u8),
    #[error("unknown confidentiality algorithm {0}")]
    UnknownConfidentialityAlgorithm(u8),
    #[error("{0:?}")]
    NotEnoughData(ECommandCode),
    #[error("unknown record type {0}")]
    UnknownRecordType(u8),
    #[error("unknown sdr sensor type {0}")]
    UnknownSensorType(u8),

    #[error("parse: {0}")]
    Parse(String),
}

impl From<ECommand> for Error {
    fn from(value: ECommand) -> Self {
        Error::Packet(EPacket::IpmiPayload(EIpmiPayload::Command(value)))
    }
}

#[derive(Debug)]
pub struct ECommandCode {
    pub command: CommandCode,
    pub expected_len: u8,
    pub get_len: usize,
    pub data: Vec<u8>,
}

impl ECommandCode {
    pub fn new(command: CommandCode, expected_len: u8, get_len: usize, data: Vec<u8>) -> Self {
        Self {
            command,
            expected_len,
            get_len,
            data,
        }
    }
}
