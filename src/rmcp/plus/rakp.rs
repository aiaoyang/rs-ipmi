use crate::{
    commands::{CommandCode, Privilege},
    err::{ECommand, Error},
    rmcp::{IpmiHeader, IpmiV2Header, Packet, Payload, PayloadType, RmcpHeader},
};

use super::open_session::StatusCode;

#[derive(Clone, Debug)]
pub enum Rakp {
    Message1(RAKPMessage1),
    Message2(RAKPMessage2),
    Message3(RAKPMessage3),
    Message4(RAKPMessage4),
}

impl From<Rakp> for Vec<u8> {
    fn from(val: Rakp) -> Self {
        match val {
            Rakp::Message1(payload) => payload.into(),
            Rakp::Message3(payload) => payload.into(),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RAKPMessage1 {
    pub message_tag: u8,
    pub managed_id: u32,
    pub console_rnd_number: u128,
    pub privilege_level: Privilege,
    pub nameonly_lookup: bool,
    pub username_length: u8,
    pub username: String,
}

impl std::fmt::Display for RAKPMessage1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "msg_tag: {}, mgr_id: {}, console_rnd_num: {}, privilege: {:?}",
            self.message_tag, self.managed_id, self.console_rnd_number, self.privilege_level
        ))
    }
}

fn privilege(nameonly_lookup: bool, privilege: Privilege) -> u8 {
    if !nameonly_lookup {
        privilege as u8 | 0x10
    } else {
        privilege as u8
    }
}

impl From<RAKPMessage1> for Vec<u8> {
    fn from(val: RAKPMessage1) -> Self {
        let username = if val.username_length > 16 {
            &val.username.as_bytes()[..=16]
        } else {
            val.username.as_bytes()
        };
        let mut result = vec![0; 28 + username.len()];
        result[0] = val.message_tag;
        // result[1..4] // reserved 0
        result[4..8].copy_from_slice(&u32::to_le_bytes(val.managed_id));
        result[8..24].copy_from_slice(&u128::to_le_bytes(val.console_rnd_number));

        result[24] = privilege(val.nameonly_lookup, val.privilege_level);
        // result[25] reserved 0
        // result[26] reserved 0

        result[27] = val.username_length;

        result[28..(28 + username.len())].copy_from_slice(username);
        result
    }
}

impl From<RAKPMessage1> for Packet {
    fn from(val: RAKPMessage1) -> Self {
        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_pre(
                PayloadType::RAKP1,
                (val.username_length + 28).into(),
            )),
            Payload::Rakp(Rakp::Message1(val.clone())),
        )
    }
}

impl RAKPMessage1 {
    pub fn new(
        message_tag: u8,
        managed_system_session_id: u32,
        remote_console_random_number: u128,
        nameonly_lookup: bool,
        requested_max_privilege: Privilege,
        username: String,
    ) -> RAKPMessage1 {
        RAKPMessage1 {
            message_tag,
            managed_id: managed_system_session_id,
            console_rnd_number: remote_console_random_number,
            nameonly_lookup,
            privilege_level: requested_max_privilege,
            username_length: { username.len().try_into().unwrap() },
            username,
        }
    }
    pub fn request_role(&self) -> u8 {
        let b = self.privilege_level as u8;
        if !self.nameonly_lookup {
            b | 0x10
        } else {
            b
        }
    }
}

#[derive(Clone, Debug)]
pub struct RAKPMessage2 {
    pub message_tag: u8,
    pub status_code: StatusCode,
    pub console_id: u32,
    pub managed_rnd_number: u128,
    pub managed_guid: u128,
    pub key_exchange_auth_code: Option<[u8; 20]>,
}

impl TryFrom<&[u8]> for RAKPMessage2 {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(ECommand::NotEnoughData {
                command: CommandCode::Raw(1),
                expected_len: 8,
                get_len: value.len(),
                data: value.into(),
            })?
        }

        let (managed_rnd_number, managed_guid, key_exchange_auth_code) = match value.len() {
            0..=7 => Err(ECommand::NotEnoughData {
                command: CommandCode::Raw(1),
                expected_len: 8,
                get_len: value.len(),
                data: value.into(),
            })?,
            40 => (
                u128::from_le_bytes(value[8..24].try_into().unwrap()),
                u128::from_le_bytes(value[24..40].try_into().unwrap()),
                None,
            ),
            60 => (
                u128::from_le_bytes(value[8..24].try_into().unwrap()),
                u128::from_le_bytes(value[24..40].try_into().unwrap()),
                Some({
                    let mut arr = [0; 20];
                    arr.copy_from_slice(&value[40..]);
                    arr
                }),
            ),
            8 => (0, 0, None),
            v => {
                unreachable!("v: {v},data: {:?}", value)
            }
        };

        Ok(RAKPMessage2 {
            message_tag: value[0],
            status_code: value[1].into(),
            console_id: u32::from_le_bytes(value[4..8].try_into().unwrap()),
            managed_rnd_number,
            managed_guid,
            key_exchange_auth_code,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RAKPMessage3 {
    pub message_tag: u8,
    pub status_code: StatusCode,
    pub managed_id: u32,
    pub key_exchange_auth_code: Option<Vec<u8>>,
}

impl From<RAKPMessage3> for Vec<u8> {
    fn from(val: RAKPMessage3) -> Self {
        let mut ret = if let Some(auth_code) = val.key_exchange_auth_code {
            let mut vec = vec![0; 8 + auth_code.len()];
            vec[8..].copy_from_slice(&auth_code);
            vec
        } else {
            vec![0; 8]
        };
        ret[0] = val.message_tag;
        ret[1] = val.status_code.into();
        // re[2] reserved
        // re[3] reserved
        ret[4..8].copy_from_slice(&val.managed_id.to_le_bytes());

        ret
    }
}

impl From<RAKPMessage3> for Packet {
    fn from(val: RAKPMessage3) -> Self {
        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_pre(
                PayloadType::RAKP3,
                match &val.key_exchange_auth_code {
                    None => 8_u16,
                    Some(auth_code) => (auth_code.len() + 8) as u16,
                },
            )),
            Payload::Rakp(Rakp::Message3(val)),
        )
    }
}

impl RAKPMessage3 {
    pub fn new(
        message_tag: u8,
        rmcp_plus_status_code: StatusCode,
        managed_system_session_id: u32,
        key_exchange_auth_code: Option<Vec<u8>>,
    ) -> RAKPMessage3 {
        RAKPMessage3 {
            message_tag,
            status_code: rmcp_plus_status_code,
            managed_id: managed_system_session_id,
            key_exchange_auth_code,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RAKPMessage4 {
    pub message_tag: u8,
    pub status_code: StatusCode,
    pub console_id: u32,
    pub integrity_auth_code: Option<[u8; 12]>,
}

impl TryFrom<&[u8]> for RAKPMessage4 {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(ECommand::NotEnoughData {
                command: CommandCode::Raw(3),
                expected_len: 8,
                get_len: value.len(),
                data: value.into(),
            })?
        }
        const EXPECTED_LEN: usize = 8 + 20;
        let value = if value.len() < EXPECTED_LEN {
            [value, vec![0; EXPECTED_LEN - value.len()].as_slice()].concat()
        } else {
            value.to_vec()
        };

        Ok(RAKPMessage4 {
            message_tag: value[0],
            status_code: value[1].into(),
            console_id: u32::from_le_bytes(value[4..8].try_into().unwrap()),
            integrity_auth_code: {
                if value.len() == 8 {
                    Some(value[8..].try_into().unwrap())
                } else {
                    None
                }
            },
        })
    }
}
