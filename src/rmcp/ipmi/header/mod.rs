pub mod v1;
pub mod v2;

use crate::err::{EIpmiHeader, Error};
use v1::IpmiV1Session;
use v2::{IpmiV2Session, V2HeaderPayloadType};

#[derive(Clone, Copy, Debug)]
pub enum IpmiSession {
    V1_5(IpmiV1Session),
    V2_0(IpmiV2Session),
}

impl TryFrom<&[u8]> for IpmiSession {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let auth_type: AuthType = value[0].try_into()?;

        match auth_type {
            AuthType::RmcpPlus => Ok(IpmiSession::V2_0(value.try_into()?)),
            _ => Ok(IpmiSession::V1_5(value.try_into()?)),
        }
    }
}

impl From<&IpmiSession> for Vec<u8> {
    fn from(val: &IpmiSession) -> Self {
        match val {
            IpmiSession::V1_5(header) => header.into(),
            IpmiSession::V2_0(header) => header.into(),
        }
    }
}

impl IpmiSession {
    pub fn write_to(self, buf: &mut [u8]) -> usize {
        match self {
            IpmiSession::V1_5(s) => s.write_to(buf),
            IpmiSession::V2_0(s) => s.write_to(buf),
        }
    }
    pub fn payload_type(&self) -> V2HeaderPayloadType {
        match self {
            IpmiSession::V1_5(_header) => V2HeaderPayloadType::Ipmi,
            IpmiSession::V2_0(header) => header.payload_type,
        }
    }

    pub fn header_len(first_byte: u8, second_byte: u8) -> Result<usize, EIpmiHeader> {
        let auth_type: AuthType = first_byte.try_into()?;
        match auth_type {
            AuthType::RmcpPlus => {
                let length = 12;
                let payload_type: V2HeaderPayloadType = second_byte.try_into()?;
                match payload_type {
                    V2HeaderPayloadType::Oem => Ok(length + 6),
                    _ => Ok(length),
                }
            }
            AuthType::None => Ok(10),
            _ => Ok(26),
        }
    }
    pub fn set_payload_len(&mut self, len: usize) {
        match self {
            IpmiSession::V1_5(h) => h.payload_length = len as u8,
            IpmiSession::V2_0(h) => h.payload_length = len as u16,
        }
    }

    pub fn payload_len(&self) -> usize {
        match self {
            IpmiSession::V1_5(a) => a.payload_length.into(),
            IpmiSession::V2_0(a) => a.payload_length.into(),
        }
    }

    pub fn seq_num(&self) -> u32 {
        match self {
            IpmiSession::V1_5(h) => h.session_seq_number,
            IpmiSession::V2_0(h) => h.session_seq_number,
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
