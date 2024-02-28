use crate::err::{IpmiHeaderError, IpmiV2HeaderError};
use crate::u8_ms_bit;

use super::AuthType;

#[derive(Clone, Copy, Debug)]
pub struct IpmiV2Header {
    pub auth_type: AuthType,
    pub payload_enc: bool,
    pub payload_auth: bool,
    pub payload_type: PayloadType,
    pub oem_iana: Option<u32>,
    pub oem_payload_id: Option<u16>,
    pub rmcp_plus_session_id: u32,
    pub session_seq_number: u32,
    pub payload_length: u16,
}

impl TryFrom<&[u8]> for IpmiV2Header {
    type Error = IpmiHeaderError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() != 12) && (value.len() != 18) {
            Err(IpmiV2HeaderError::WrongLength)?
        }

        let auth_type: AuthType = value[0].try_into()?;
        let payload_bit_slice = value[1];
        let payload_enc = u8_ms_bit(payload_bit_slice, 0);
        let payload_auth = u8_ms_bit(payload_bit_slice, 1);
        let payload_type: PayloadType = (payload_bit_slice & 0x3f).try_into()?;
        let oem_iana: Option<u32>;
        let oem_payload_id: Option<u16>;
        let rmcp_plus_session_id: u32;
        let session_seq_number: u32;
        let payload_length: u16;
        match payload_type {
            PayloadType::Oem => {
                if value.len() != 18 {
                    Err(IpmiV2HeaderError::WrongLength)?
                }
                oem_iana = Some(u32::from_le_bytes([value[2], value[3], value[4], value[5]]));
                oem_payload_id = Some(u16::from_le_bytes([value[6], value[7]]));
                rmcp_plus_session_id =
                    u32::from_le_bytes([value[8], value[9], value[10], value[11]]);
                session_seq_number =
                    u32::from_le_bytes([value[12], value[13], value[14], value[15]]);
                payload_length = u16::from_le_bytes([value[16], value[17]]);
            }
            _ => {
                oem_iana = None;
                oem_payload_id = None;
                rmcp_plus_session_id = u32::from_le_bytes([value[2], value[3], value[4], value[5]]);
                session_seq_number = u32::from_le_bytes([value[6], value[7], value[8], value[9]]);
                payload_length = u16::from_le_bytes([value[10], value[11]]);
            }
        }

        Ok(IpmiV2Header {
            auth_type,
            payload_enc,
            payload_auth,
            payload_type,
            oem_iana,
            oem_payload_id,
            rmcp_plus_session_id,
            session_seq_number,
            payload_length,
        })
    }
}

impl From<IpmiV2Header> for Vec<u8> {
    fn from(val: IpmiV2Header) -> Self {
        match val.payload_type {
            PayloadType::Oem => {
                let oem_iana_le = val.oem_iana.unwrap().to_le_bytes();
                let oem_payload_id_le = val.oem_payload_id.unwrap().to_le_bytes();
                let rmcp_ses_le = val.rmcp_plus_session_id.to_le_bytes();
                let ses_seq_le = val.session_seq_number.to_le_bytes();
                let len_le = val.payload_length.to_le_bytes();

                let mut result = Vec::new();
                result.extend([val.auth_type.into(), {
                    match (val.payload_enc, val.payload_auth) {
                        (true, true) => (0x1 << 7) | (0x1 << 6) | (u8::from(val.payload_type)),
                        (true, false) => (0x1 << 7) | (u8::from(val.payload_type)),
                        (false, true) => (0x1 << 6) | (u8::from(val.payload_type)),
                        (false, false) => u8::from(val.payload_type),
                    }
                }]);
                result.extend(oem_iana_le);
                result.extend(oem_payload_id_le);
                result.extend(rmcp_ses_le);
                result.extend(ses_seq_le);
                result.extend(len_le);
                result
            }
            _ => {
                let mut ret = [0_u8; 12];
                ret[0] = val.auth_type.into();
                ret[1] = (val.payload_enc as u8) << 7
                    | (val.payload_auth as u8) << 6
                    | (u8::from(val.payload_type));
                ret[2..6].copy_from_slice(&val.rmcp_plus_session_id.to_le_bytes());
                ret[6..10].copy_from_slice(&val.session_seq_number.to_le_bytes());
                ret[10..].copy_from_slice(&val.payload_length.to_le_bytes());

                ret.into()
            }
        }
    }
}

impl IpmiV2Header {
    pub fn new_est(payload_length: u16) -> IpmiV2Header {
        Self::new(
            AuthType::RmcpPlus,
            true,
            true,
            PayloadType::Ipmi,
            0,
            0,
            payload_length,
        )
    }
    pub fn new_pre(payload_type: PayloadType, payload_length: u16) -> Self {
        Self::new(
            AuthType::RmcpPlus,
            false,
            false,
            payload_type,
            0x0,
            0x0,
            payload_length,
        )
    }
    pub fn new(
        auth_type: AuthType,
        payload_enc: bool,
        payload_auth: bool,
        payload_type: PayloadType,
        rmcp_plus_session_id: u32,
        session_seq_number: u32,
        payload_length: u16,
    ) -> IpmiV2Header {
        IpmiV2Header {
            auth_type,
            payload_enc,
            payload_auth,
            payload_type,
            oem_iana: None,
            oem_payload_id: None,
            rmcp_plus_session_id,
            session_seq_number,
            payload_length,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PayloadType {
    Ipmi,
    Sol,
    Oem,
    RmcpOpenSessionRequest,
    RmcpOpenSessionResponse,
    RAKP1,
    RAKP2,
    RAKP3,
    RAKP4,
}

impl TryFrom<u8> for PayloadType {
    type Error = IpmiV2HeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0b0011_1111 {
            0x00 => Ok(PayloadType::Ipmi),
            0x01 => Ok(PayloadType::Sol),
            0x02 => Ok(PayloadType::Oem),
            0x10 => Ok(PayloadType::RmcpOpenSessionRequest),
            0x11 => Ok(PayloadType::RmcpOpenSessionResponse),
            0x12 => Ok(PayloadType::RAKP1),
            0x13 => Ok(PayloadType::RAKP2),
            0x14 => Ok(PayloadType::RAKP3),
            0x15 => Ok(PayloadType::RAKP4),
            _ => Err(IpmiV2HeaderError::UnsupportedPayloadType(value)),
        }
    }
}

impl From<PayloadType> for u8 {
    fn from(val: PayloadType) -> Self {
        match &val {
            PayloadType::Ipmi => 0x00,
            PayloadType::Sol => 0x01,
            PayloadType::Oem => 0x02,
            PayloadType::RmcpOpenSessionRequest => 0x10,
            PayloadType::RmcpOpenSessionResponse => 0x11,
            PayloadType::RAKP1 => 0x12,
            PayloadType::RAKP2 => 0x13,
            PayloadType::RAKP3 => 0x14,
            PayloadType::RAKP4 => 0x15,
        }
    }
}
