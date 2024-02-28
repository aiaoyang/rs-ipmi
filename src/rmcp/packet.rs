use crate::{
    err::PacketError,
    rmcp::{plus::crypto::aes_128_cbc_decrypt, IpmiHeader, IpmiV1Header, PayloadType, RmcpHeader},
    CompletionCode, IpmiV2Header,
};

use super::{
    open_session::RMCPPlusOpenSession, rakp::Rakp, request::ReqPayload, response::RespPayload,
};

#[derive(Clone, Debug)]
pub struct Packet {
    pub rmcp_header: RmcpHeader,
    pub ipmi_header: IpmiHeader,
    pub payload: Payload,
}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketError;
    fn try_from(value: &[u8]) -> Result<Self, PacketError> {
        let nbytes: usize = value.len();
        if nbytes < 20 {
            Err(PacketError::WrongLength)?
        }
        let ipmi_header_len = IpmiHeader::header_len(value[4], value[5])?;
        let ipmi_header: IpmiHeader = value[4..(ipmi_header_len + 4)].try_into()?;
        let payload_length = ipmi_header.payload_len();
        let mut payload_vec = Vec::new();
        payload_vec.extend_from_slice(&value[(nbytes - payload_length)..nbytes]);
        Ok(Packet {
            rmcp_header: value[..4].try_into()?,
            ipmi_header,
            payload: {
                match payload_length {
                    0 => Payload::None,
                    _ => match ipmi_header.payload_type() {
                        PayloadType::Ipmi => Payload::IpmiResp(payload_vec.as_slice().try_into()?),
                        PayloadType::RmcpOpenSessionResponse => Payload::Rmcp(
                            RMCPPlusOpenSession::Response(payload_vec.as_slice().try_into()?),
                        ),
                        PayloadType::RAKP2 => {
                            Payload::Rakp(Rakp::Message2(payload_vec.as_slice().try_into()?))
                        }
                        PayloadType::RAKP4 => {
                            Payload::Rakp(Rakp::Message4(payload_vec.as_slice().try_into()?))
                        }
                        _ => unreachable!(),
                    },
                }
            },
        })
    }
}

// k2 req
impl TryFrom<(&[u8], &[u8; 20])> for Packet {
    type Error = PacketError;

    fn try_from(value: (&[u8], &[u8; 20])) -> Result<Self, PacketError> {
        let nbytes: usize = value.0.len();
        if nbytes < 20 {
            Err(PacketError::WrongLength)?
        }
        let ipmi_header_len = IpmiHeader::header_len(value.0[4], value.0[5])?;
        let ipmi_header: IpmiHeader = value.0[4..(ipmi_header_len + 4)].try_into()?;
        let payload_length = ipmi_header.payload_len();
        let mut payload_vec = Vec::new();
        if let IpmiHeader::V2_0(header) = ipmi_header {
            if header.payload_enc {
                // decrypt slice
                let binding = aes_128_cbc_decrypt(
                    &mut value.0[16..16 + payload_length].to_vec(),
                    value.1[..16].try_into().unwrap(),
                );
                payload_vec.extend(binding);
            } else {
                payload_vec.extend_from_slice(&value.0[(nbytes - payload_length)..nbytes])
            }
        } else {
            payload_vec.extend_from_slice(&value.0[(nbytes - payload_length)..nbytes])
        }
        Ok(Packet {
            rmcp_header: value.0[..4].try_into()?,
            ipmi_header,
            payload: {
                match payload_length {
                    0 => Payload::None,
                    _ => match ipmi_header.payload_type() {
                        PayloadType::Ipmi => {
                            Payload::IpmiResp(RespPayload::try_from(payload_vec.as_slice())?)
                        }
                        PayloadType::RmcpOpenSessionResponse => Payload::Rmcp(
                            RMCPPlusOpenSession::Response(payload_vec.as_slice().try_into()?),
                        ),
                        PayloadType::RAKP2 => {
                            Payload::Rakp(Rakp::Message2(payload_vec.as_slice().try_into()?))
                        }
                        PayloadType::RAKP4 => {
                            Payload::Rakp(Rakp::Message4(payload_vec.as_slice().try_into()?))
                        }
                        _ => unreachable!(),
                    },
                }
            },
        })
    }
}

impl From<Packet> for Vec<u8> {
    fn from(val: Packet) -> Self {
        let mut result = Vec::new();
        result.extend(&<[u8; 4]>::from(&val.rmcp_header));
        result.extend(&<Vec<u8>>::from(val.ipmi_header));
        match val.payload {
            Payload::None => {}
            payload => result.extend(&<Vec<u8>>::from(payload)),
        }
        result
    }
}

impl Packet {
    pub fn new(rmcp_header: RmcpHeader, ipmi_header: IpmiHeader, payload: Payload) -> Packet {
        Packet {
            rmcp_header,
            ipmi_header,
            payload,
        }
    }

    pub(crate) fn set_session_id(&mut self, id: u32) {
        if let IpmiHeader::V2_0(IpmiV2Header {
            rmcp_plus_session_id,
            // session_seq_number,
            ..
        }) = &mut self.ipmi_header
        {
            *rmcp_plus_session_id = id;
        }
    }

    pub(crate) fn set_session_seq_num(&mut self, seq_num: u32) {
        if let IpmiHeader::V2_0(IpmiV2Header {
            // rmcp_plus_session_id,
            session_seq_number,
            ..
        }) = &mut self.ipmi_header
        {
            *session_seq_number = seq_num;
        }
    }
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            rmcp_header: RmcpHeader::default(),
            ipmi_header: IpmiHeader::V1_5(IpmiV1Header::default()),
            payload: Payload::None,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Payload {
    IpmiResp(RespPayload),
    IpmiReq(ReqPayload),
    Rmcp(RMCPPlusOpenSession),
    Rakp(Rakp),
    None,
}

impl Payload {
    pub fn data_and_completion(&self) -> (&[u8], CompletionCode) {
        if let Payload::IpmiResp(RespPayload {
            data,
            completion_code,
            ..
        }) = self
        {
            (data, *completion_code)
        } else {
            (&[], CompletionCode::CompletedNormally)
        }
    }
}

impl From<Payload> for Vec<u8> {
    fn from(val: Payload) -> Self {
        match val {
            Payload::Rmcp(payload) => payload.into(),
            Payload::Rakp(payload) => payload.into(),
            Payload::IpmiResp(_) => unreachable!(),
            Payload::IpmiReq(payload) => payload.into(),
            Payload::None => Vec::new(),
        }
    }
}
