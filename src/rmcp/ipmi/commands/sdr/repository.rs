use crate::commands::CommandCode;
use crate::err::{ECommand, Error};
use crate::request::IpmiRequestPayload;
use crate::{ECommandCode, IpmiCommand, NetFn, Packet, PacketPayload};
pub struct GetSDRRepositoryInfo;

#[derive(Debug)]
pub struct SDRRepositoryInfo {
    // valid for 0x01 | 0x51 | 0x02
    pub sdr_version: u8,
    pub record_count: u16,
}

impl IpmiCommand for GetSDRRepositoryInfo {
    type Output = SDRRepositoryInfo;

    const NET_FN: NetFn = NetFn::Storage;
    const COMMAND: CommandCode = CommandCode::Raw(0x20);

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

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let data = PacketPayload::IpmiReq(IpmiRequestPayload::new(
            Self::NET_FN,
            Self::COMMAND,
            Vec::new(),
        ));
        Packet::new_with_data(data).write_to(buf)
    }
}
