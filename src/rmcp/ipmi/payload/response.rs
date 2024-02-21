use std::fmt;

// use bitvec::prelude::*;

use crate::{
    err::{IpmiPayloadError, IpmiPayloadRequestError},
    rmcp::commands::Command,
};

use super::{
    bmc::{AddrType, SlaveAddress, SoftwareType},
    netfn_lun::{NetfnLun, RqseqLun},
};

#[derive(Clone, Debug)]
pub struct RespPayload {
    pub rq_addr: Address,
    pub netfn_rqlun: NetfnLun,
    // checksum 1
    pub rs_addr: Address,
    pub rqseq_rslun: RqseqLun,
    pub command: Command,
    pub completion_code: CompletionCode,
    pub data: Option<Vec<u8>>,
    // checksum 2
}

// impl fmt::Display for RespPayload {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let data: String = match self.data.clone() {
//             Some(x) => format!("{:x?}", x),
//             None => "None".to_string(),
//         };
//         write!(
//             f,
//             "IPMI Response:\n\tRequester Address: {}\n\tNetFn: {}\n\tResponder Address: {}\n\tRequester Sequence Number: {}\n\tCommand: {}\n\tCompletion Code: {:?}\n\tDate: {}",
//             self.rq_addr,
//             self.netfn_rqlun.netfn(),
//             self.rs_addr,
//             self.rqseq_rslun.rqseq(),
//             self.command,
//             self.completion_code,
//             data
//         )
//     }
// }

impl TryFrom<&[u8]> for RespPayload {
    type Error = IpmiPayloadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(IpmiPayloadRequestError::WrongLength)?
        }
        let netfn_rqlun = NetfnLun::from(value[1]);
        let rqseq_rslun = RqseqLun::from(value[4]);
        Ok(RespPayload {
            rq_addr: value[0].into(),
            netfn_rqlun,
            rs_addr: value[3].into(),
            rqseq_rslun,
            command: value[5].into(),
            completion_code: value[6].into(),
            data: {
                let len = value.len() - 1;
                if len == 7 {
                    None
                } else {
                    Some(value[7..len].into())
                }
            },
        })
    }
}

impl RespPayload {
    pub fn payload_length(&self) -> usize {
        match &self.data {
            Some(d) => d.len() + 8,
            None => 8,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Address {
    Slave(SlaveAddress),
    Software(SoftwareType),
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Slave(x) => write!(f, "{:?}", x),
            Address::Software(x) => write!(f, "{:?}", x),
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

// impl fmt::Display for CompletionCode {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             CompletionCode::CompletedNormally => write!(f, "Command Completed Normally"),
//             CompletionCode::NodeBusy => write!(f, "Node Busy. Command could not be processed because command processing resources are temporarily unavailable"),
//             CompletionCode::InvalidCommand => write!(f, "Invalid Command. Used to indicate an unrecognized or unsupported command"),
//             CompletionCode::InvalidCommandForLun => write!(f, "Command invalid for given LUN"),
//             CompletionCode::Timeout => write!(f, "Timeout while processing command. Response unavailable"),
//             CompletionCode::OutOfSpace => write!(f, "Out of space. Command could not be completed because of a lack of storage space required to execute the given command operation"),
//             CompletionCode::ReservationCancelled => write!(f, "Reservation Canceled or Invalid Reservation ID"),
//             CompletionCode::RequestDataTruncated => write!(f, "Request data truncated"),
//             CompletionCode::RequestDataLengthInvalid => write!(f, "Request data length invalid"),
//             CompletionCode::RequestDataFieldLengthLimitExceeded => write!(f, "Request data field length limit exceeded"),
//             CompletionCode::ParameterOutOfRange => write!(f, "Parameter out of range. One or more parameters in the data field of the Request are out of range. This is different from ‘Invalid data field’ (CCh) code in that it indicates that the erroneous field(s) has a contiguous range of possible values"),
//             CompletionCode::CannotReturnNumberOfRqDataBytes => write!(f, "Cannot return number of requested data bytes"),
//             CompletionCode::RqSensorDataRecordNotPresent => write!(f, "Requested Sensor, data, or record not present"),
//             CompletionCode::InvalidDataFieldInRequest => write!(f, "Invalid data field in Request"),
//             CompletionCode::CommandIllegalForSensor => write!(f, "Command illegal for specified sensor or record type"),
//             CompletionCode::CommandResponseNotProvided => write!(f, "Command response could not be provided"),
//             CompletionCode::CantExecuteDuplicateRq => write!(f, "Cannot execute duplicated request"),
//             CompletionCode::FailedSDRUpdateMode => write!(f, "Command response could not be provided. SDR Repository in update mode"),
//             CompletionCode::FailedDevFirmwareMode => write!(f, "Command response could not be provided. Device in firmware update mode"),
//             CompletionCode::FailedInitInProgress => write!(f, "Command response could not be provided. BMC initialization or initialization agent in progress"),
//             CompletionCode::DestinationUnavailable => write!(f, "Destination unavailable"),
//             CompletionCode::CannotExecuteCommandInsuffientPrivileges => write!(f, "Cannot execute command due to insufficient privilege level or other securitybased restriction (e.g. disabled for ‘firmware firewall’)."),
//             CompletionCode::CommandSubFunctionUnavailable => write!(f, "Cannot execute command. Command, or request parameter(s), not supported in present state"),
//             CompletionCode::CannotExecuteCommandIllegalParam => write!(f, "Cannot execute command. Parameter is illegal because command sub-function has been disabled or is unavailable (e.g. disabled for ‘firmware firewall’)"),
//             CompletionCode::UnspecifiedError => write!(f, "Unspecified error"),
//             CompletionCode::OEM(x) => write!(f, "Device specific (OEM) completion code: 0x{:X}", x),
//             CompletionCode::CommandCode(x) => write!(f, "Command specific code: 0x{:X}", x),
//             CompletionCode::Reserved(x) => write!(f, "Reserved code: 0x{:X}", x),
//         }
//     }
// }

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
