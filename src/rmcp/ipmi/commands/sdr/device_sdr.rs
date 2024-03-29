use std::num::NonZeroU16;

use nonmax::NonMaxU8;

use crate::{
    commands::CommandCode,
    request::ReqPayload,
    storage::sdr::{RecordId, SdrRecord},
    ECommand, ECommandCode, Error, IpmiCommand, Payload,
};

#[derive(Debug, Clone, Copy)]
pub struct GetDeviceSdrCommand {
    reservation_id: Option<NonZeroU16>,
    record_id: RecordId,
    offset: u8,
    bytes_to_read: Option<NonMaxU8>,
}

#[derive(Debug, Clone)]
pub struct RecordInfo {
    pub next_entry: RecordId,
    pub record: SdrRecord,
}

impl IpmiCommand for GetDeviceSdrCommand {
    type Output = RecordInfo;

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 2 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                self.command(),
                2,
                data.len(),
                data.into(),
            )))?
        }

        let next_entry = RecordId::new_raw(u16::from_le_bytes([data[0], data[1]]));
        let data = &data[2..];
        Ok(RecordInfo {
            next_entry,
            record: SdrRecord::parse(data, self.command())?,
        })
    }

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn command(&self) -> crate::commands::CommandCode {
        CommandCode::Raw(0x23)
    }

    fn payload(&self) -> crate::Payload {
        let mut data = vec![0u8; 6];

        data[0..2].copy_from_slice(
            &self
                .reservation_id
                .map(NonZeroU16::get)
                .unwrap_or(0)
                .to_le_bytes(),
        );

        data[2..4].copy_from_slice(&self.record_id.value().to_le_bytes());
        data[4] = self.offset;
        data[5] = self.bytes_to_read.map(|v| v.get()).unwrap_or(0xFF);

        Payload::IpmiReq(ReqPayload::new(self.netfn(), self.command(), data))
    }
}

impl GetDeviceSdrCommand {
    pub fn new(reservation_id: Option<NonZeroU16>, record_id: RecordId) -> Self {
        Self {
            reservation_id,
            record_id,
            offset: 0,
            bytes_to_read: None,
        }
    }
}
