pub mod v1;
pub mod v2;

use crate::err::{EIpmiHeader, Error};
use v1::IpmiV1Header;
use v2::{IpmiV2Header, PayloadType};

#[derive(Clone, Copy, Debug)]
pub enum IpmiHeader {
    V1_5(IpmiV1Header),
    V2_0(IpmiV2Header),
}

impl TryFrom<&[u8]> for IpmiHeader {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let auth_type: AuthType = value[0].try_into()?;

        match auth_type {
            AuthType::RmcpPlus => Ok(IpmiHeader::V2_0(value.try_into()?)),
            _ => Ok(IpmiHeader::V1_5(value.try_into()?)),
        }
    }
}

impl From<IpmiHeader> for Vec<u8> {
    fn from(val: IpmiHeader) -> Self {
        match val {
            IpmiHeader::V1_5(header) => header.into(),
            IpmiHeader::V2_0(header) => header.into(),
        }
    }
}

impl IpmiHeader {
    pub fn payload_type(&self) -> PayloadType {
        match self {
            IpmiHeader::V1_5(_header) => PayloadType::Ipmi,
            IpmiHeader::V2_0(header) => header.payload_type,
        }
    }

    pub fn header_len(first_byte: u8, second_byte: u8) -> Result<usize, EIpmiHeader> {
        let auth_type: AuthType = first_byte.try_into()?;
        match auth_type {
            AuthType::RmcpPlus => {
                let length = 12;
                let payload_type: PayloadType = second_byte.try_into()?;
                match payload_type {
                    PayloadType::Oem => Ok(length + 6),
                    _ => Ok(length),
                }
            }
            AuthType::None => Ok(10),
            _ => Ok(26),
        }
    }
    pub fn set_payload_len(&mut self, len: usize) {
        match self {
            IpmiHeader::V1_5(h) => h.payload_length = len as u8,
            IpmiHeader::V2_0(h) => h.payload_length = len as u16,
        }
    }

    pub fn payload_len(&self) -> usize {
        match self {
            IpmiHeader::V1_5(a) => a.payload_length.into(),
            IpmiHeader::V2_0(a) => a.payload_length.into(),
        }
    }

    pub fn seq_num(&self) -> u32 {
        match self{
            IpmiHeader::V1_5(h) => h.session_seq_number,
            IpmiHeader::V2_0(h) => h.session_seq_number, 
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AuthType {
    None,
    MD2,
    MD5,
    PasswordOrKey,
    Oem,
    RmcpPlus,
}

impl TryFrom<u8> for AuthType {
    type Error = EIpmiHeader;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(AuthType::None),
            0x01 => Ok(AuthType::MD2),
            0x02 => Ok(AuthType::MD5),
            0x04 => Ok(AuthType::PasswordOrKey),
            0x05 => Ok(AuthType::Oem),
            0x06 => Ok(AuthType::RmcpPlus),
            _ => Err(EIpmiHeader::UnsupportedAuthType(value)),
        }
    }
}

impl From<AuthType> for u8 {
    fn from(val: AuthType) -> Self {
        match &val {
            AuthType::None => 0x00,
            AuthType::MD2 => 0x01,
            AuthType::MD5 => 0x02,
            AuthType::PasswordOrKey => 0x04,
            AuthType::Oem => 0x05,
            AuthType::RmcpPlus => 0x06,
        }
    }
}
