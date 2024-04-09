use crate::commands::CommandCode;
use crate::err::{ECommand, Error};
use crate::request::ReqPayload;
use crate::{ECommandCode, IpmiCommand, Payload};
pub struct GetSDRRepositoryInfo;

#[derive(Debug)]
pub struct SDRRepositoryInfo {
    // valid for 0x01 | 0x51 | 0x02
    pub sdr_version: u8,
    pub record_count: u16,
}

impl IpmiCommand for GetSDRRepositoryInfo {
    type Output = SDRRepositoryInfo;

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn command(&self) -> CommandCode {
        CommandCode::Raw(0x20)
    }

    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(self.netfn(), self.command(), Vec::new()))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 14 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                CommandCode::Raw(0x20),
                14,
                data.len(),
                data.into(),
            )))?
        }
        Ok(SDRRepositoryInfo {
            sdr_version: data[0],
            record_count: u16::from_le_bytes([data[1], data[2]]),
        })
    }
}
