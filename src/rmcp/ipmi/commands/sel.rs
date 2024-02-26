use std::{
    num::{NonZeroU16, NonZeroU8},
    rc::Rc,
};

use crate::{
    rmcp::{request::ReqPayload, storage::sel::entry::ParseEntryError, IpmiCommand, Payload},
    SelEntry,
};

#[allow(unused)]
#[derive(Debug)]
pub struct SelRecord {
    pub next_record_id: [u8; 2],
    pub entry: SelEntry,
    pub raw_data: Rc<[u8]>,
}

pub struct GetSelEntry {
    reservation_id: Option<NonZeroU16>,
    record_id: [u8; 2],
    offset: u8,
    bytes_to_read: Option<NonZeroU8>,
}

impl IpmiCommand for GetSelEntry {
    type Output = SelRecord;

    type Error = ParseEntryError;

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn commnad(&self) -> crate::Command {
        0x43.into()
    }

    fn payload(self) -> Payload {
        let GetSelEntry {
            reservation_id,
            record_id,
            offset,
            bytes_to_read,
        } = self;

        let mut data = [0u8; 6];

        data[0..2].copy_from_slice(&reservation_id.map(|v| v.get()).unwrap_or(0).to_le_bytes());
        data[2..4].copy_from_slice(&record_id);
        data[4] = offset;
        data[5] = bytes_to_read.map(|v| v.get()).unwrap_or(0xFF);

        crate::rmcp::Payload::IpmiReq(ReqPayload::new(self.netfn(), self.commnad(), data.to_vec()))
    }

    fn parse(data: &[u8]) -> Result<Self::Output, Self::Error> {
        if data.len() < 2 {
            Err(ParseEntryError::NotEnoughData)?
        }
        Ok(SelRecord {
            next_record_id: [data[0], data[1]],
            entry: SelEntry::parse(&data[2..])?,
            raw_data: Rc::from(data),
        })
    }
}

impl GetSelEntry {
    pub fn new(reservation_id: u16, record_id: u16, offset: u8) -> Self {
        Self {
            reservation_id: NonZeroU16::new(reservation_id),
            record_id: u16::to_le_bytes(record_id),
            offset,
            bytes_to_read: NonZeroU8::new(0xff),
        }
    }
}
