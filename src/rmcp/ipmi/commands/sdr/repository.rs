use crate::commands::CommandCode;
use crate::err::{ECommand, Error};
use crate::request::ReqPayload;
use crate::{IpmiCommand, Payload};
pub struct GetSDRRepositoryInfoCommand;

#[derive(Debug)]
pub struct SDRRepositoryInfo {
    // valid for 0x01 | 0x51 | 0x02
    pub sdr_version: u8,
    pub record_count: u16,
}

impl IpmiCommand for GetSDRRepositoryInfoCommand {
    type Output = SDRRepositoryInfo;

    type Error = Error;

    fn netfn() -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn command() -> CommandCode {
        CommandCode::Raw(0x20)
    }

    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(Self::netfn(), Self::command(), Vec::new()))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Self::Error> {
        if data.len() < 14 {
            println!("data: {:?}", data);
            Err(ECommand::NotEnoughData {
                command: CommandCode::Raw(0x20),
                expected_len: 14,
                get_len: data.len(),
                data: data.into(),
            })?
        }
        Ok(SDRRepositoryInfo {
            sdr_version: data[0],
            record_count: u16::from_le_bytes([data[1], data[2]]),
        })
    }
}

// #[derive(Debug)]
// pub enum SdrSensorType {
//     FullSensor = 0x01,
//     CompactSensor = 0x02,
//     EventOnlySensor = 0x03,
//     EntityAssociation = 0x08,
//     DeviceEntityAssociation = 0x09,
//     GenericDeviceLocator = 0x10,
//     FRUDeviceLocator = 0x11,
//     MCDeviceLocator = 0x12,
//     MCConfirmation = 0x13,
//     BMCMessageChannelInfo = 0x14,
//     Oem = 0xc0,
// }

// impl TryFrom<u8> for SdrSensorType {
//     type Error = Error;

//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value as isize {
//             0x01 => Ok(SdrSensorType::FullSensor),
//             0x02 => Ok(SdrSensorType::CompactSensor),
//             0x03 => Ok(SdrSensorType::EventOnlySensor),
//             0x08 => Ok(SdrSensorType::EntityAssociation),
//             0x09 => Ok(SdrSensorType::DeviceEntityAssociation),
//             0x10 => Ok(SdrSensorType::GenericDeviceLocator),
//             0x11 => Ok(SdrSensorType::FRUDeviceLocator),
//             0x12 => Ok(SdrSensorType::MCDeviceLocator),
//             0x13 => Ok(SdrSensorType::MCConfirmation),
//             0x14 => Ok(SdrSensorType::BMCMessageChannelInfo),
//             0xc0 => Ok(SdrSensorType::Oem),
//             _ => Err(ECommand::UnknownSensorType(value))?,
//         }
//     }
// }

// use crate::connection::{IpmiCommand, Message, NetFn, ParseResponseError};
