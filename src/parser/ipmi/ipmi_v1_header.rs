use crate::err::{IpmiHeaderError, IpmiV1HeaderError};

use super::ipmi_header::AuthType;

#[derive(Clone, Copy, Debug)]
pub struct IpmiV1Header {
    pub auth_type: AuthType,
    pub session_seq_number: u32,
    pub session_id: u32,
    pub auth_code: Option<u128>,
    pub payload_length: u8,
}

impl TryFrom<&[u8]> for IpmiV1Header {
    type Error = IpmiHeaderError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() != 10) && (value.len() != 26) {
            Err(IpmiV1HeaderError::WrongLength)?
        }
        let auth_type: AuthType = value[0].try_into()?;
        let auth_code: Option<u128>;
        let payload_length: u8;

        match auth_type {
            AuthType::None => {
                auth_code = None;
                payload_length = value[9]
            }
            _ => {
                auth_code = Some(u128::from_be_bytes(
                    value[9..25]
                        .try_into()
                        .map_err(|_| IpmiV1HeaderError::WrongLength)?,
                ));
                payload_length = value[25];
            }
        };

        Ok(IpmiV1Header {
            auth_type: value[0].try_into()?,
            session_seq_number: u32::from_be_bytes([value[1], value[2], value[3], value[4]]),
            session_id: u32::from_be_bytes([value[5], value[6], value[7], value[8]]),
            auth_code,
            payload_length,
        })
    }
}

impl From<IpmiV1Header> for Vec<u8> {
    fn from(val: IpmiV1Header) -> Self {
        let seq_be = val.session_seq_number.to_be_bytes();
        let ses_be = val.session_id.to_be_bytes();
        let mut result: Vec<u8> = Vec::new();
        result.push(val.auth_type.into());
        result.extend(seq_be);
        result.extend(ses_be);
        match val.auth_type {
            AuthType::None => {
                result.push(val.payload_length);
                result
            }
            _ => {
                let auth_be = val.auth_code.unwrap().to_be_bytes();
                result.extend(auth_be);
                result.push(val.payload_length);
                result
            }
        }
    }
}

impl Default for IpmiV1Header {
    fn default() -> Self {
        Self {
            auth_type: AuthType::None,
            session_seq_number: 0x00,
            session_id: 0x00,
            auth_code: None,
            payload_length: 0,
        }
    }
}
