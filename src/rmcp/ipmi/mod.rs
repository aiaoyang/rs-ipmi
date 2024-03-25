pub use header::v1::*;
pub use header::v2::*;
pub use header::*;
pub use payload::*;
pub use storage::sel::*;

pub mod commands;
pub mod raw;
pub mod storage;

mod header;
mod payload;

use std::fmt;

use thiserror::Error;

use crate::Packet;
use crate::RmcpHeader;
use crate::{
    commands::CommandCode,
    err::{ECommand, EIpmiPayload, Error},
};

use super::Payload;

pub trait IpmiCommand {
    type Output;
    fn netfn() -> NetFn;
    fn command() -> CommandCode;
    fn payload(&self) -> Payload;

    fn check_cc_success(cc: CompletionCode) -> Result<CompletionCode, Error> {
        if cc.is_success() {
            Ok(cc)
        } else {
            Err(EIpmiPayload::CompletionCode(cc))?
        }
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error>;

    fn gen_packet(&self) -> Packet {
        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_est(32)),
            self.payload(),
        )
    }
}

#[derive(Clone, Debug, Copy, Error)]
pub enum CompletionCode {
    CompletedNormally,
    NodeBusy,
    InvalidCommand,
    InvalidCommandForLun,
    Timeout,
    OutOfSpace,
    ReservationCancelled,
    RequestDataTruncated,
    RequestDataLengthInvalid,
    RequestDataFieldLengthLimitExceeded,
    ParameterOutOfRange,
    CannotReturnNumberOfRqDataBytes,
    RqSensorDataRecordNotPresent,
    InvalidDataFieldInRequest,
    CommandIllegalForSensor,
    CommandResponseNotProvided,
    CantExecuteDuplicateRq,
    FailedSDRUpdateMode,
    FailedDevFirmwareMode,
    FailedInitInProgress,
    DestinationUnavailable,
    CannotExecuteCommandInsuffientPrivileges,
    CommandSubFunctionUnavailable,
    CannotExecuteCommandIllegalParam,
    UnspecifiedError,
    OEM(u8),
    CommandCode(u8),
    Reserved(u8),
}

impl std::fmt::Display for CompletionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl From<u8> for CompletionCode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => CompletionCode::CompletedNormally,
            0xc0 => CompletionCode::NodeBusy,
            0xc1 => CompletionCode::InvalidCommand,
            0xc2 => CompletionCode::InvalidCommandForLun,
            0xc3 => CompletionCode::Timeout,
            0xc4 => CompletionCode::OutOfSpace,
            0xc5 => CompletionCode::ReservationCancelled,
            0xc6 => CompletionCode::RequestDataTruncated,
            0xc7 => CompletionCode::RequestDataLengthInvalid,
            0xc8 => CompletionCode::RequestDataFieldLengthLimitExceeded,
            0xc9 => CompletionCode::ParameterOutOfRange,
            0xca => CompletionCode::CannotReturnNumberOfRqDataBytes,
            0xcb => CompletionCode::RqSensorDataRecordNotPresent,
            0xcc => CompletionCode::InvalidDataFieldInRequest,
            0xcd => CompletionCode::CommandIllegalForSensor,
            0xce => CompletionCode::CommandResponseNotProvided,
            0xcf => CompletionCode::CantExecuteDuplicateRq,
            0xd0 => CompletionCode::FailedSDRUpdateMode,
            0xd1 => CompletionCode::FailedDevFirmwareMode,
            0xd2 => CompletionCode::FailedInitInProgress,
            0xd3 => CompletionCode::DestinationUnavailable,
            0xd4 => CompletionCode::CannotExecuteCommandInsuffientPrivileges,
            0xd5 => CompletionCode::CommandSubFunctionUnavailable,
            0xd6 => CompletionCode::CannotExecuteCommandIllegalParam,
            0xff => CompletionCode::UnspecifiedError,
            0x01..=0x7e => CompletionCode::OEM(value),
            0x80..=0xbe => CompletionCode::CommandCode(value),
            _ => CompletionCode::Reserved(value),
        }
    }
}

impl From<CompletionCode> for u8 {
    fn from(value: CompletionCode) -> u8 {
        match value {
            CompletionCode::CompletedNormally => 0x00,
            CompletionCode::NodeBusy => 0xc0,
            CompletionCode::InvalidCommand => 0xc1,
            CompletionCode::InvalidCommandForLun => 0xc2,
            CompletionCode::Timeout => 0xc3,
            CompletionCode::OutOfSpace => 0xc4,
            CompletionCode::ReservationCancelled => 0xc5,
            CompletionCode::RequestDataTruncated => 0xc6,
            CompletionCode::RequestDataLengthInvalid => 0xc7,
            CompletionCode::RequestDataFieldLengthLimitExceeded => 0xc8,
            CompletionCode::ParameterOutOfRange => 0xc9,
            CompletionCode::CannotReturnNumberOfRqDataBytes => 0xca,
            CompletionCode::RqSensorDataRecordNotPresent => 0xcb,
            CompletionCode::InvalidDataFieldInRequest => 0xcc,
            CompletionCode::CommandIllegalForSensor => 0xcd,
            CompletionCode::CommandResponseNotProvided => 0xce,
            CompletionCode::CantExecuteDuplicateRq => 0xcf,
            CompletionCode::FailedSDRUpdateMode => 0xd0,
            CompletionCode::FailedDevFirmwareMode => 0xd1,
            CompletionCode::FailedInitInProgress => 0xd2,
            CompletionCode::DestinationUnavailable => 0xd3,
            CompletionCode::CannotExecuteCommandInsuffientPrivileges => 0xd4,
            CompletionCode::CommandSubFunctionUnavailable => 0xd5,
            CompletionCode::CannotExecuteCommandIllegalParam => 0xd6,
            CompletionCode::UnspecifiedError => 0xff,
            CompletionCode::OEM(code) => code,
            CompletionCode::CommandCode(code) => code,
            CompletionCode::Reserved(code) => code,
        }
    }
}

impl CompletionCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::CompletedNormally)
    }

    pub fn is_reserved(&self) -> bool {
        matches!(self, Self::Reserved(_))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetfnLun(pub u8);

impl From<u8> for NetfnLun {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl NetfnLun {
    const IPMB_LUN_MASK: u8 = 0x03;
    pub fn new(netfn: impl Into<u8>, lun: Lun) -> Self {
        Self(netfn.into() << 2 | lun as u8 & Self::IPMB_LUN_MASK)
    }

    pub fn netfn(&self) -> NetFn {
        (self.0 >> 2).into()
    }
    pub fn lun(&self) -> Result<Lun, ECommand> {
        Lun::new(self.0 & Self::IPMB_LUN_MASK)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NetFn {
    Chassis,
    Bridge,
    SensorEvent,
    App,
    Firmware,
    Storage,
    Transport,
    Reserved,
    Unknown(u8),
}
impl From<u8> for NetFn {
    fn from(value: u8) -> Self {
        match value {
            0x00..=0x01 => NetFn::Chassis,
            0x02..=0x03 => NetFn::Bridge,
            0x04..=0x05 => NetFn::SensorEvent,
            0x06..=0x07 => NetFn::App,
            0x08..=0x09 => NetFn::Firmware,
            0x0A..=0x0B => NetFn::Storage,
            0x0C..=0x0D => NetFn::Transport,
            0x0E..=0x2B => NetFn::Reserved,
            _ => NetFn::Unknown(value),
        }
    }
}

impl From<NetFn> for u8 {
    fn from(value: NetFn) -> u8 {
        match value {
            NetFn::Chassis => 0x00,
            NetFn::Bridge => 0x02,
            NetFn::SensorEvent => 0x04,
            NetFn::App => 0x06,
            NetFn::Firmware => 0x08,
            NetFn::Storage => 0x0A,
            NetFn::Transport => 0x0C,
            NetFn::Reserved => 0x0E,
            NetFn::Unknown(fn_code) => fn_code,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Lun {
    Bmc,
    Oem1,
    Sms,
    Oem2,
}

impl TryFrom<u8> for Lun {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Lun::Bmc),
            1 => Ok(Lun::Oem1),
            2 => Ok(Lun::Sms),
            3 => Ok(Lun::Oem2),
            _ => Err(Error::TryFromU8(value)),
        }
    }
}

impl Lun {
    pub fn new(value: u8) -> Result<Self, ECommand> {
        match value {
            0b00 => Ok(Lun::Bmc),
            0b01 => Ok(Lun::Oem1),
            0b10 => Ok(Lun::Sms),
            0b11 => Ok(Lun::Oem2),
            _ => Err(ECommand::UnknownLun(value)),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Address {
    Slave(SlaveAddress),
    Software(SoftwareType),
}
impl From<u8> for Address {
    fn from(value: u8) -> Self {
        let rs_addr_type: AddrType = ((value >> 7) != 0).into();
        match rs_addr_type {
            AddrType::SlaveAddress => Self::Slave((value << 1 >> 1).into()),
            AddrType::SoftwareId => Self::Software((value << 1 >> 1).into()),
        }
    }
}

pub enum AddrType {
    SlaveAddress,
    SoftwareId,
}

impl From<bool> for AddrType {
    fn from(value: bool) -> Self {
        match value {
            false => AddrType::SlaveAddress,
            true => AddrType::SoftwareId,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum SlaveAddress {
    Bmc,
    Unknown(u8),
}

impl From<u8> for SlaveAddress {
    fn from(value: u8) -> Self {
        match value {
            0x20 => SlaveAddress::Bmc,
            _ => SlaveAddress::Unknown(value),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum SoftwareType {
    Bios,
    SmiHandler,
    SystemManagementSoftware,
    Oem,
    RemoteConsoleSoftware(u8),
    TerminalModeRemoteConsole,
    Reserved(u8),
}

impl From<u8> for SoftwareType {
    fn from(value: u8) -> Self {
        match value {
            0x00..=0x0F => SoftwareType::Bios,
            0x10..=0x1F => SoftwareType::SmiHandler,
            0x20..=0x2F => SoftwareType::SystemManagementSoftware,
            0x30..=0x3F => SoftwareType::Oem,
            0x40..=0x46 => SoftwareType::RemoteConsoleSoftware(value - 0x3F),
            0x47 => SoftwareType::TerminalModeRemoteConsole,
            _ => SoftwareType::Reserved(value),
        }
    }
}
