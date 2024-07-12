use crate::{
    commands::CommandCode,
    crypto::aes_128_cbc_decrypt_new,
    err::{EPacket, Error},
    request::IpmiRequestPayload,
    rmcp::{IpmiSession, RmcpHeader, V2HeaderPayloadType},
    CompletionCode, EIpmiPayload, IpmiV2Session,
};

use super::{
    open_session::RMCPPlusOpenSessionPayload, rakp::RakpPayload, response::IpmiRespPayload,
};

#[derive(Clone, Debug)]
pub struct Packet<'a> {
    pub rmcp_header: RmcpHeader,
    pub ipmi_session: IpmiSession,
    pub data: PacketPayload<'a>,
}

impl TryFrom<&[u8]> for Packet<'static> {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len: usize = value.len();
        if len < 20 {
            Err(EPacket::WrongLength)?
        }
        let ipmi_header_len = IpmiSession::header_len(value[4], value[5])?;
        let ipmi_header: IpmiSession = value[4..(ipmi_header_len + 4)].try_into()?;
        let payload_length = ipmi_header.payload_len();
        let mut payload_vec = Vec::new();
        payload_vec.extend_from_slice(&value[(len - payload_length)..len]);
        Ok(Packet {
            rmcp_header: value[..4].try_into()?,
            ipmi_session: ipmi_header,
            data: {
                match payload_length {
                    0 => PacketPayload::None,
                    _ => match ipmi_header.payload_type() {
                        V2HeaderPayloadType::Ipmi => {
                            PacketPayload::IpmiResp(payload_vec.as_slice().try_into()?)
                        }
                        V2HeaderPayloadType::RmcpOpenSessionResponse => {
                            PacketPayload::Rmcp(RMCPPlusOpenSessionPayload::Response(
                                payload_vec.as_slice().try_into()?,
                            ))
                        }
                        V2HeaderPayloadType::RAKP2 => PacketPayload::Rakp(RakpPayload::Message2(
                            payload_vec.as_slice().try_into()?,
                        )),
                        V2HeaderPayloadType::RAKP4 => PacketPayload::Rakp(RakpPayload::Message4(
                            payload_vec.as_slice().try_into()?,
                        )),
                        _ => unreachable!(),
                    },
                }
            },
        })
    }
}

pub type SessionK2<'a> = &'a [u8; 20];
pub type Data<'a> = &'a [u8];

// k2 req, (payload, key)
// impl<'a> TryFrom<(Data<'a>, SessionK2<'a>)> for Packet<'static> {
//     type Error = Error;
//
//     fn try_from(value: (&[u8], &[u8; 20])) -> Result<Self, Self::Error> {
//         let len: usize = value.0.len();
//         if len < 20 {
//             Err(EPacket::WrongLength)?
//         }
//         let ipmi_header_len = IpmiSession::header_len(value.0[4], value.0[5])?;
//         let ipmi_header: IpmiSession = value.0[4..(ipmi_header_len + 4)].try_into()?;
//         let payload_length = ipmi_header.payload_len();
//         let mut payload_vec = Vec::new();
//         if let IpmiSession::V2_0(header) = ipmi_header {
//             if header.payload_enc {
//                 // decrypt slice
//                 let binding = aes_128_cbc_decrypt(
//                     &mut value.0[16..16 + payload_length].to_vec(),
//                     value.1[..16].try_into().unwrap(),
//                 )?;
//                 payload_vec.extend(binding);
//             } else {
//                 payload_vec.extend_from_slice(&value.0[(len - payload_length)..len])
//             }
//         } else {
//             payload_vec.extend_from_slice(&value.0[(len - payload_length)..len])
//         }
//         Ok(Packet {
//             rmcp_header: value.0[..4].try_into()?,
//             ipmi_session: ipmi_header,
//             data: {
//                 match payload_length {
//                     0 => PacketPayload::None,
//                     _ => match ipmi_header.payload_type() {
//                         V2HeaderPayloadType::Ipmi => PacketPayload::IpmiResp(
//                             IpmiRespPayload::try_from(payload_vec.as_slice())?,
//                         ),
//                         V2HeaderPayloadType::RmcpOpenSessionResponse => {
//                             PacketPayload::Rmcp(RMCPPlusOpenSessionPayload::Response(
//                                 payload_vec.as_slice().try_into()?,
//                             ))
//                         }
//                         V2HeaderPayloadType::RAKP2 => PacketPayload::Rakp(RakpPayload::Message2(
//                             payload_vec.as_slice().try_into()?,
//                         )),
//                         V2HeaderPayloadType::RAKP4 => PacketPayload::Rakp(RakpPayload::Message4(
//                             payload_vec.as_slice().try_into()?,
//                         )),
//                         V2HeaderPayloadType::Sol => {
//                             PacketPayload::Sol(payload_vec.as_slice().try_into()?)
//                         }
//                         _ => unreachable!(),
//                     },
//                 }
//             },
//         })
//     }
// }
// impl<'a> Default for Packet<'a> {
//     fn default() -> Self {
//         Self {
//             rmcp_header: RmcpHeader::default(),
//             ipmi_session: IpmiSession::V1_5(IpmiV1Header::default()),
//             data: PacketPayload::None,
//         }
//     }
// }

impl<'a> Packet<'a> {
    pub fn decode_from(k2: &[u8; 20], data: &'a mut [u8], len: usize) -> Result<Packet<'a>, Error> {
        let len: usize = len;
        if len < 20 {
            Err(EPacket::WrongLength)?
        }
        let rmcp_header: RmcpHeader = data[..4].try_into()?;

        let ipmi_header_len = IpmiSession::header_len(data[4], data[5])?;
        let ipmi_session: IpmiSession = data[4..(ipmi_header_len + 4)].try_into()?;
        let payload_length = ipmi_session.payload_len();
        let payload = if let IpmiSession::V2_0(header) = ipmi_session {
            if header.payload_enc {
                // decrypt slice
                aes_128_cbc_decrypt_new(
                    &mut data[16..16 + payload_length],
                    k2[..16].try_into().unwrap(),
                )?
            } else {
                &data[(len - payload_length)..len]
            }
        } else {
            &data[(len - payload_length)..len]
        };
        Ok(Packet {
            rmcp_header,
            ipmi_session,
            data: {
                match payload_length {
                    0 => PacketPayload::None,
                    _ => match ipmi_session.payload_type() {
                        V2HeaderPayloadType::Ipmi => {
                            PacketPayload::IpmiResp(IpmiRespPayload::try_from(payload)?)
                        }
                        V2HeaderPayloadType::RmcpOpenSessionResponse => PacketPayload::Rmcp(
                            RMCPPlusOpenSessionPayload::Response(payload.try_into()?),
                        ),
                        V2HeaderPayloadType::RAKP2 => {
                            PacketPayload::Rakp(RakpPayload::Message2(payload.try_into()?))
                        }
                        V2HeaderPayloadType::RAKP4 => {
                            PacketPayload::Rakp(RakpPayload::Message4(payload.try_into()?))
                        }
                        V2HeaderPayloadType::Sol => PacketPayload::Sol(payload.try_into()?),
                        _ => unreachable!(),
                    },
                }
            },
        })
    }
    pub fn new(
        rmcp_header: RmcpHeader,
        ipmi_header: IpmiSession,
        payload: PacketPayload,
    ) -> Packet {
        Packet {
            rmcp_header,
            ipmi_session: ipmi_header,
            data: payload,
        }
    }

    pub fn new_raw(
        rmcp_header: RmcpHeader,
        ipmi_header: IpmiSession,
        data: &'a [u8],
    ) -> Packet<'a> {
        Packet {
            rmcp_header,
            ipmi_session: ipmi_header,
            data: PacketPayload::Raw(data),
        }
    }

    pub fn new_with_data(data: PacketPayload) -> Packet {
        Packet::new(
            RmcpHeader::default(),
            IpmiSession::V2_0(IpmiV2Session::new_est(32)),
            data,
        )
    }

    pub fn new_sol_packet(data: &[u8]) -> Packet {
        Packet {
            rmcp_header: RmcpHeader::default(),
            ipmi_session: IpmiSession::V2_0(IpmiV2Session::new_sol_payload(
                ((data.len() as u8) + 7).into(),
            )),
            data: PacketPayload::Sol(SolPayload::new(data.to_vec())),
        }
    }

    pub(crate) fn set_session_id(&mut self, id: u32) {
        if let IpmiSession::V2_0(IpmiV2Session {
            rmcp_plus_session_id,
            // session_seq_number,
            ..
        }) = &mut self.ipmi_session
        {
            *rmcp_plus_session_id = id;
        }
    }

    pub(crate) fn set_session_seq_num(&mut self, seq_num: u32) {
        if let IpmiSession::V2_0(IpmiV2Session {
            // rmcp_plus_session_id,
            session_seq_number,
            ..
        }) = &mut self.ipmi_session
        {
            *session_seq_number = seq_num;
        }
    }

    pub fn write_to(self, buf: &mut [u8]) -> usize {
        let mut write_byte = 0;
        write_byte += self.rmcp_header.write_to(&mut buf[write_byte..]);
        write_byte += self.ipmi_session.write_to(&mut buf[write_byte..]);
        write_byte += self.data.write_to(&mut buf[write_byte..]);

        write_byte
    }
}

#[derive(Clone, Debug)]
pub enum PacketPayload<'a> {
    IpmiResp(IpmiRespPayload),
    IpmiReq(IpmiRequestPayload),
    Rmcp(RMCPPlusOpenSessionPayload),
    Rakp(RakpPayload),
    Sol(SolPayload),
    Raw(&'a [u8]),
    None,
}

impl<'a> From<&PacketPayload<'a>> for Vec<u8> {
    fn from(val: &PacketPayload) -> Self {
        match val {
            PacketPayload::Rmcp(payload) => payload.into(),
            PacketPayload::Rakp(payload) => payload.into(),
            PacketPayload::IpmiResp(_) => unreachable!(),
            PacketPayload::IpmiReq(payload) => {
                let mut buf = [0_u8; 1024];
                let n = payload.write_to(&mut buf);
                buf[..n].to_vec()
            }

            PacketPayload::Sol(payload) => payload.into(),
            PacketPayload::None => Vec::new(),
            PacketPayload::Raw(raw) => raw.to_vec(),
        }
    }
}

impl<'a> PacketPayload<'a> {
    pub fn write_to(self, buf: &mut [u8]) -> usize {
        match self {
            PacketPayload::IpmiResp(_) => unreachable!(),
            PacketPayload::IpmiReq(ipmi_req) => ipmi_req.write_to(buf),
            PacketPayload::Rmcp(_) => todo!(),
            PacketPayload::Rakp(_) => todo!(),
            PacketPayload::Sol(_) => todo!(),
            PacketPayload::Raw(_) => todo!(),
            PacketPayload::None => todo!(),
        }
    }
    pub fn data_and_completion(&self) -> (&[u8], CompletionCode) {
        if let PacketPayload::IpmiResp(IpmiRespPayload {
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

    pub fn command(&self) -> Option<CommandCode> {
        match self {
            PacketPayload::IpmiResp(p) => Some(p.command),
            PacketPayload::IpmiReq(p) => Some(p.command),
            // PacketData::IpmiReq(IpmiRequest::Bridge(p)) => Some(p.command),
            _ => None,
        }
    }
}

#[test]
fn test_packetpayload_new() {
    let buf = &mut [0_u8; 1024];
    let packet_payload = PacketPayload::IpmiReq(IpmiRequestPayload::new(
        crate::NetFn::App,
        CommandCode::GetChannelCipherSuites,
        Vec::new(),
    ));

    let payload: Vec<u8> = (&packet_payload).into();
    let n = packet_payload.write_to(buf);
    assert_eq!(&payload, &buf[..n]);
}

#[derive(Clone, Debug, Default)]
pub struct SolPayload {
    packet_sequence_number: u8,
    acked_packet_number: u8,
    accepted_character_count: u8,
    operation: u8,
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for SolPayload {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 4 {
            return Err(EIpmiPayload::WrongLength)?;
        }
        Ok(Self {
            packet_sequence_number: value[0],
            acked_packet_number: value[1],
            accepted_character_count: value[2],
            operation: value[3],
            data: value[3..].to_vec(),
        })
    }
}

impl From<&SolPayload> for Vec<u8> {
    fn from(value: &SolPayload) -> Self {
        let mut v = vec![
            value.packet_sequence_number,
            value.acked_packet_number,
            value.accepted_character_count,
            value.operation,
        ];
        v.extend(value.data.to_owned());
        v
    }
}

impl SolPayload {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            packet_sequence_number: 1,
            accepted_character_count: data.len() as u8,
            data,
            ..Default::default()
        }
    }
}
