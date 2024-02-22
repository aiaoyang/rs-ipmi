// use crate::ipmi::data::data::Data;
use std::fmt::Debug;

use crate::{
    err::{IpmiPayloadError, ParseError, PrivilegeError},
    rmcp::{
        netfn_lun::NetFn, request::ReqPayload, AuthType, IpmiHeader, IpmiV1Header, Packet, Payload,
        RmcpHeader,
    },
    u8_ms_bit, Command,
};

#[derive(Clone)]
pub struct GetChannelAuthCapabilitiesRequest {
    pub channel_version: bool,
    pub channel_number: u8,
    pub max_privilege: Privilege,
}

impl From<GetChannelAuthCapabilitiesRequest> for Vec<u8> {
    fn from(val: GetChannelAuthCapabilitiesRequest) -> Self {
        vec![
            ((val.channel_version as u8) << 7 | val.channel_number << 4 >> 4),
            val.max_privilege as u8,
        ]
    }
}

impl GetChannelAuthCapabilitiesRequest {
    pub fn new(
        channel_version: bool,
        channel_number: u8,
        max_privilege: Privilege,
    ) -> GetChannelAuthCapabilitiesRequest {
        GetChannelAuthCapabilitiesRequest {
            channel_version,
            channel_number,
            max_privilege,
        }
    }

    pub fn create_packet(
        self,
        auth_type: AuthType,
        session_seq_number: u32,
        session_id: u32,
        auth_code: Option<u128>,
    ) -> Packet {
        let data_bytes: Vec<u8> = self.into();
        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V1_5(IpmiV1Header {
                auth_type,
                session_seq_number,
                session_id,
                auth_code,
                payload_length: (data_bytes.len() as u8) + 7,
            }),
            Payload::IpmiReq(ReqPayload::new(
                NetFn::App,
                Command::GetChannelAuthCapabilities,
                Some(data_bytes),
            )),
        )
    }
}

#[derive(Debug)]
pub struct GetChannelAuthCapabilitiesResponse {
    pub channel_number: u8,
    pub auth_version: AuthVersion,
    pub auth_type: Vec<AuthType>,
    pub kg_status: KG,
    pub per_message_auth: bool,
    pub user_level_auth: bool,
    pub anon_login: AnonLogin,
    pub channel_extended_cap: AuthVersion,
    pub oem_id: u32, // 3 bytes not 4
    pub oem_aux_data: u8,
}

impl TryFrom<&[u8]> for GetChannelAuthCapabilitiesResponse {
    type Error = IpmiPayloadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 8 {
            Err(IpmiPayloadError::WrongLength)?
        }
        // let auth_bv = BitSlice::<u8, Msb0>::from_element(&value[1]);
        // let auth2_bv = BitSlice::<u8, Msb0>::from_element(&value[2]);
        Ok(GetChannelAuthCapabilitiesResponse {
            channel_number: value[0],
            auth_version: u8_ms_bit(value[1], 0).into(),
            auth_type: {
                let mut result: Vec<AuthType> = vec![];
                // if auth_bv[2] {
                if u8_ms_bit(value[1], 2) {
                    result.push(AuthType::Oem)
                }
                if u8_ms_bit(value[1], 3) {
                    result.push(AuthType::PasswordOrKey)
                }
                if u8_ms_bit(value[1], 5) {
                    result.push(AuthType::MD5)
                }
                if u8_ms_bit(value[1], 6) {
                    result.push(AuthType::MD2)
                }
                if u8_ms_bit(value[1], 7) {
                    result.push(AuthType::None)
                }
                result
            },
            // kg_status: auth2_bv[2].into(),
            kg_status: u8_ms_bit(value[2], 2).into(),
            per_message_auth: u8_ms_bit(value[2], 3),
            user_level_auth: !u8_ms_bit(value[2], 4),
            anon_login: AnonLogin::new(
                u8_ms_bit(value[2], 5).into(),
                u8_ms_bit(value[2], 6).into(),
                u8_ms_bit(value[2], 7).into(),
            ),
            // channel_extended_cap: BitSlice::<u8, Msb0>::from_element(&value[3])[6].into(),
            channel_extended_cap: u8_ms_bit(value[3], 6).into(),
            oem_id: u32::from_le_bytes([0, value[4], value[5], value[6]]),
            oem_aux_data: value[7],
        })
    }
}

#[derive(Debug)]

pub enum KG {
    /*
        0b = KG is set to default (all 0’s). User key KUID will be used in place of
        KG in RAKP. (Knowledge of KG not required for activating session.)
        1b = KG is set to non-zero value. (Knowledge of both KG and user
        password (if not anonymous login) required for activating session.)
    */
    Defualt,
    NonZero,
}

impl From<bool> for KG {
    fn from(value: bool) -> Self {
        match value {
            false => KG::Defualt,
            true => KG::NonZero,
        }
    }
}

#[derive(Debug)]
pub struct AnonLogin {
    /*
        1b = Non-null usernames enabled. (One or more users are enabled
        that have non-null usernames).
        1b = Null usernames enabled (One or more users that have a null
        username, but non-null password, are presently enabled)
        1b = Anonymous Login enabled (A user that has
    */
    pub non_null_username: AnonStatus,
    pub null_username: AnonStatus,
    pub anonymous_login: AnonStatus,
}

impl AnonLogin {
    pub fn new(
        non_null_username: AnonStatus,
        null_username: AnonStatus,
        anonymous_login: AnonStatus,
    ) -> AnonLogin {
        AnonLogin {
            non_null_username,
            null_username,
            anonymous_login,
        }
    }
}
#[derive(Debug)]

pub enum AnonStatus {
    Enabled,
    Disabled,
}

impl From<bool> for AnonStatus {
    fn from(value: bool) -> Self {
        match value {
            true => AnonStatus::Enabled,
            false => AnonStatus::Disabled,
        }
    }
}

#[derive(Debug)]
pub enum AuthVersion {
    IpmiV2,
    IpmiV1_5,
}

impl From<bool> for AuthVersion {
    fn from(value: bool) -> Self {
        match value {
            true => AuthVersion::IpmiV2,
            false => AuthVersion::IpmiV1_5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Privilege {
    Reserved,
    Callback,
    User,
    Operator,
    Administrator,
    Oem,
    // Unknown(u8),
}

impl TryFrom<u8> for Privilege {
    type Error = IpmiPayloadError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Privilege::Reserved),
            0x01 => Ok(Privilege::Callback),
            0x02 => Ok(Privilege::User),
            0x03 => Ok(Privilege::Operator),
            0x04 => Ok(Privilege::Administrator),
            0x05 => Ok(Privilege::Oem),
            _ => Err(ParseError::Privilege(PrivilegeError::UnknownPrivilege(
                value,
            )))?,
        }
    }
}

// impl From<Privilege> for u8 {
//     fn from(val: Privilege) -> Self {
//         match val {
//             Privilege::Reserved => 0x00,
//             Privilege::Callback => 0x01,
//             Privilege::User => 0x02,
//             Privilege::Operator => 0x03,
//             Privilege::Administrator => 0x04,
//             Privilege::Oem => 0x05,
//         }
//     }
// }
