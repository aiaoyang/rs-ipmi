use std::{
    num::{NonZeroU16, NonZeroU8},
    rc::Rc,
};

use crate::{
    commands::CommandCode,
    err::{ECommand, Error},
    rmcp::{request::IpmiRequestPayload, IpmiCommand, PacketPayload},
    ECommandCode, NetFn, SelEntry,
};

#[derive(Clone)]
pub struct GetSelInfo;

#[allow(unused)]
#[derive(Debug)]
pub struct SelInfo {
    // Response Data
    pub sel_version: u8,
    pub entries: u16,
    pub free_space: u16,
    pub last_add_time: u32,
    pub last_del_time: u32,
    pub support_alloc_info: bool,
    pub support_reserve: bool,
    pub support_partial_add: bool,
    pub support_delete: bool,
    pub overflow: bool,
}

impl IpmiCommand for GetSelInfo {
    type Output = SelInfo;

    const NET_FN: NetFn = NetFn::Storage;
    const COMMAND: CommandCode = CommandCode::Raw(0x40);

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 14 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                Self::COMMAND,
                14,
                data.len(),
                data.into(),
            )))?
        }
        Ok(SelInfo {
            sel_version: data[0],
            entries: u16::from_le_bytes([data[1], data[2]]),
            free_space: u16::from_le_bytes([data[3], data[4]]),
            last_add_time: u32::from_le_bytes([data[5], data[6], data[7], data[8]]),
            last_del_time: u32::from_le_bytes([data[9], data[10], data[11], data[12]]),
            support_alloc_info: data[13] & 0x01 != 0,
            support_reserve: data[13] & 0x02 != 0,
            support_partial_add: data[13] & 0x04 != 0,
            support_delete: data[13] & 0x08 != 0,
            overflow: data[13] & 0x80 != 0,
        })
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        IpmiRequestPayload::new(Self::NET_FN, Self::COMMAND, None).write_to(buf)
    }
}

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
    const NET_FN: NetFn = NetFn::Storage;
    const COMMAND: CommandCode = CommandCode::Raw(0x43);

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 2 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                Self::COMMAND,
                2,
                data.len(),
                data.into(),
            )))?
        }
        Ok(SelRecord {
            next_record_id: [data[0], data[1]],
            entry: SelEntry::parse(&data[2..])?,
            raw_data: Rc::from(data),
        })
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let GetSelEntry {
            reservation_id,
            record_id,
            offset,
            bytes_to_read,
        } = self;

        let mut data = [0u8; 6];

        data[0..2].copy_from_slice(&reservation_id.map(|v| v.get()).unwrap_or(0).to_le_bytes());
        data[2..4].copy_from_slice(record_id);
        data[4] = *offset;
        data[5] = bytes_to_read.map(|v| v.get()).unwrap_or(0xFF);

        let data = crate::rmcp::PacketPayload::IpmiReq(IpmiRequestPayload::new(
            Self::NET_FN,
            Self::COMMAND,
            Some(&data),
        ));
        data.write_to(buf)
        // Packet::new_with_data(data).write_to(buf)
    }
}

impl GetSelEntry {
    pub fn new(reservation_id: u16, record_id: u16, offset: u8) -> Self {
        Self {
            reservation_id: NonZeroU16::new(reservation_id),
            record_id: u16::to_le_bytes(record_id),
            offset,
            bytes_to_read: NonZeroU8::new(0xFF),
        }
    }
}
