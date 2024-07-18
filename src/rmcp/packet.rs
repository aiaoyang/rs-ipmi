use core::marker::PhantomData;

use crate::{
    commands::CommandCode,
    crypto::{aes_128_cbc_decrypt, aes_128_cbc_encrypt},
    err::{EPacket, Error},
    open_session::RMCPPlusOpenSessionRequest,
    rakp::{RAKPMessage1, RAKPMessage3},
    request::IpmiRequestPayload,
    rmcp::{IpmiSession, RmcpHeader, V2HeaderPayloadType},
    AuthType, CompletionCode, EIpmiPayload, IpmiCommand, IpmiV1Session, IpmiV2Session,
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

impl<'a> TryFrom<&'a [u8]> for Packet<'a> {
    type Error = Error;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let len: usize = value.len();
        if len < 20 {
            Err(EPacket::WrongLength)?
        }
        let ipmi_header_len = IpmiSession::header_len(value[4], value[5])?;
        let ipmi_header: IpmiSession = value[4..(ipmi_header_len + 4)].try_into()?;
        let payload_length = ipmi_header.payload_len();
        let payload = &value[(len - payload_length)..len];
        Ok(Packet {
            rmcp_header: value[..4].try_into()?,
            ipmi_session: ipmi_header,
            data: {
                match payload_length {
                    0 => PacketPayload::None,
                    _ => match ipmi_header.payload_type() {
                        V2HeaderPayloadType::Ipmi => PacketPayload::IpmiResp((payload).try_into()?),
                        V2HeaderPayloadType::RmcpOpenSessionResponse => PacketPayload::Rmcp(
                            RMCPPlusOpenSessionPayload::Response(payload.try_into()?),
                        ),
                        V2HeaderPayloadType::RAKP2 => {
                            PacketPayload::Rakp(RakpPayload::Message2(payload.try_into()?))
                        }
                        V2HeaderPayloadType::RAKP4 => {
                            PacketPayload::Rakp(RakpPayload::Message4(payload.try_into()?))
                        }
                        _ => unreachable!(),
                    },
                }
            },
        })
    }
}

pub type SessionK2<'a> = &'a [u8; 20];
pub type Data<'a> = &'a [u8];

impl<'a> Packet<'a> {
    pub fn decode_from(data: &'a mut [u8], k2: &[u8; 20], len: usize) -> Result<Packet<'a>, Error> {
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
                aes_128_cbc_decrypt(
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

    //
    // pub fn new_raw(
    //     rmcp_header: RmcpHeader,
    //     ipmi_header: IpmiSession,
    //     data: &'a [u8],
    // ) -> Packet<'a> {
    //     Packet {
    //         rmcp_header,
    //         ipmi_session: ipmi_header,
    //         data: PacketPayload::Raw(data),
    //     }
    // }

    // pub fn new_with_data(data: PacketPayload) -> Packet {
    //     Packet::new(
    //         RmcpHeader::default(),
    //         IpmiSession::V2_0(IpmiV2Session::new_est(32)),
    //         data,
    //     )
    // }

    // pub fn new_sol_packet(data: &[u8]) -> Packet {
    //     Packet {
    //         rmcp_header: RmcpHeader::default(),
    //         ipmi_session: IpmiSession::V2_0(IpmiV2Session::new_sol_payload(
    //             ((data.len() as u8) + 7).into(),
    //         )),
    //         data: PacketPayload::Sol(SolPayload::new(data.to_vec())),
    //     }
    // }

    // pub(crate) fn set_session_id(&mut self, id: u32) {
    //     if let IpmiSession::V2_0(IpmiV2Session {
    //         rmcp_plus_session_id,
    //         // session_seq_number,
    //         ..
    //     }) = &mut self.ipmi_session
    //     {
    //         *rmcp_plus_session_id = id;
    //     }
    // }
    //
    // pub(crate) fn set_session_seq_num(&mut self, seq_num: u32) {
    //     if let IpmiSession::V2_0(IpmiV2Session {
    //         // rmcp_plus_session_id,
    //         session_seq_number,
    //         ..
    //     }) = &mut self.ipmi_session
    //     {
    //         *session_seq_number = seq_num;
    //     }
    // }

    // pub fn write_to(self, buf: &mut [u8]) -> usize {
    //     let mut write_byte = 0;
    //     write_byte += self.rmcp_header.write_to(&mut buf[write_byte..]);
    //     write_byte += self.ipmi_session.write_to(&mut buf[write_byte..]);
    //     write_byte += self.data.write_to(&mut buf[write_byte..]);
    //
    //     write_byte
    // }
    // pub fn encrypt_and_write_to(self, buf: &mut [u8], key: &[u8; 16]) -> usize {
    //     let mut write_byte = 0;
    //     write_byte += self.rmcp_header.write_to(&mut buf[write_byte..]);
    //     write_byte += self.ipmi_session.write_to(&mut buf[write_byte..]);
    //     write_byte += self.data.write_to(&mut buf[write_byte..]);
    //     aes_128_cbc_encrypt(buf, &mut write_byte, key);
    //
    //     write_byte
    // }
}

pub struct IpmiV1_5Packet;
pub struct IpmiV2Packet;
pub struct RakpPacket;

pub struct PayloadTypeRakp1;
pub struct PayloadTypeRakp3;
pub struct PayloadTypeIpmi;
pub struct PayloadTypeIpmiEncrypted;
pub struct PayloadTypeSol;
pub struct PayloadTypeRmcpOpenSession;

pub struct PacketBuilder<IpmiSession, PayloadType> {
    rmcp_header: RmcpHeader,

    session_id: u32,
    session_seq: u32,

    session: PhantomData<IpmiSession>,
    payload_type: PhantomData<PayloadType>,
}

impl PacketBuilder<IpmiV2Packet, PayloadTypeIpmi> {
    pub fn write_payload<T>(self, buf: &mut [u8], request: T) -> &[u8]
    where
        T: IpmiCommand,
    {
        let mut n = self.rmcp_header.write_to(buf);

        let session = IpmiV2Session::new(
            AuthType::RmcpPlus,
            false,
            false,
            V2HeaderPayloadType::Ipmi,
            0,
            0,
            0,
        );
        n += session.write_to(&mut buf[n..]);
        let tmp = n;
        let payload_len = request.write_to(&mut buf[n..]);
        n += payload_len;
        session.set_payload_length(&mut buf[tmp - 2..tmp], payload_len as u16);
        &buf[..n]
    }
}
impl PacketBuilder<IpmiV2Packet, PayloadTypeIpmiEncrypted> {
    pub fn write_payload<'a, T>(
        self,
        buf: &'a mut [u8],
        request: &T,
        key: &[u8; 16],
        k1: &[u8; 20],
    ) -> &'a [u8]
    where
        T: IpmiCommand,
    {
        let mut n = self.rmcp_header.write_to(buf);

        let session = IpmiV2Session::new(
            AuthType::RmcpPlus,
            true,
            true,
            V2HeaderPayloadType::Ipmi,
            self.session_id,
            self.session_seq,
            0,
        );
        let rmcp_header_tail_idx = n;
        n += session.write_to(&mut buf[n..]);
        let session_tail_idx = n;
        let mut payload_len = request.write_to(&mut buf[n..]);
        aes_128_cbc_encrypt(&mut buf[session_tail_idx..], &mut payload_len, key);
        session.set_payload_length(
            &mut buf[session_tail_idx - 2..session_tail_idx],
            payload_len as u16,
        );
        n += payload_len;
        n += add_tailer(
            &mut buf[rmcp_header_tail_idx..],
            n - rmcp_header_tail_idx,
            k1,
        );
        &buf[..n]
    }
}

// Trailer's source is the session header and payload
fn add_tailer(data: &mut [u8], mut len: usize, k1: &[u8; 20]) -> usize {
    // Session Trailer (Table 13-8)
    // +---------------+
    // | Integrity PAD |  n bytes
    // | Pad Length    |  1 byte
    // | Next Header   |  1 byte  (0x07)
    // | AuthCode      | 12 bytes
    // +---------------+
    let origin_len = len;
    let m = (origin_len + 2 + 12) % 4;
    let pad_len = if m != 0 { 4 - m } else { 0 };

    for _ in 0..pad_len {
        data[len] = 0xff;
        len += 1;
    }
    data[len] = pad_len as u8;
    len += 1;

    // Next Header, Reserved in IPMI v2.0. Set to 07h for RMCP+ packets defined in this specification.
    data[len] = 0x07;
    len += 1;

    let auth_code = hmac_sha1::hmac_sha1(k1, &data[..len]);
    data[len..len + 12].copy_from_slice(&auth_code[..12]);
    len += 12;
    len - origin_len
}

impl PacketBuilder<IpmiV1_5Packet, ()> {
    pub fn write_payload<'a>(self, buf: &'a mut [u8], payload: PacketPayload<'_>) -> &'a [u8] {
        let ipmi_v1_5_session = IpmiV1Session::default();
        let mut n = self.rmcp_header.write_to(buf);
        n += ipmi_v1_5_session.write_to(&mut buf[n..]);
        n += payload.write_to(&mut buf[n..]);
        &buf[..n]
    }
}

impl PacketBuilder<RakpPacket, PayloadTypeRakp1> {
    pub fn write_payload(self, buf: &mut [u8], message: RAKPMessage1) -> &[u8] {
        let session = IpmiSession::V2_0(IpmiV2Session::new_pre(
            V2HeaderPayloadType::RAKP1,
            (message.username_length + 28).into(),
        ));

        let mut n = self.rmcp_header.write_to(buf);
        n += session.write_to(&mut buf[n..]);
        n += message.write_to(&mut buf[n..]);
        &buf[..n]
    }
}
impl PacketBuilder<RakpPacket, PayloadTypeRakp3> {
    pub fn write_payload(self, buf: &mut [u8], message: RAKPMessage3) -> &[u8] {
        let session = IpmiSession::V2_0(IpmiV2Session::new_pre(
            V2HeaderPayloadType::RAKP3,
            match &message.key_exchange_auth_code {
                None => 8_u16,
                Some(auth_code) => (auth_code.len() + 8) as u16,
            },
        ));
        let mut n = self.rmcp_header.write_to(buf);
        n += session.write_to(&mut buf[n..]);
        n += message.write_to(&mut buf[n..]);
        &buf[..n]
    }
}

impl PacketBuilder<IpmiV2Packet, PayloadTypeRmcpOpenSession> {
    pub fn write_payload(self, buf: &mut [u8], req: RMCPPlusOpenSessionRequest) -> &[u8] {
        let session = IpmiSession::V2_0(IpmiV2Session::new_pre(
            V2HeaderPayloadType::RmcpOpenSessionRequest,
            29,
        ));
        let mut n = self.rmcp_header.write_to(buf);
        n += session.write_to(&mut buf[n..]);
        n += req.write_to(&mut buf[n..]);
        &buf[..n]
    }
}

impl PacketBuilder<(), ()> {
    pub fn v1() -> PacketBuilder<IpmiV1_5Packet, ()> {
        PacketBuilder {
            rmcp_header: RmcpHeader::default(),
            session: PhantomData,
            payload_type: PhantomData,
            session_id: 0,
            session_seq: 0,
        }
    }

    pub fn v2(session_id: u32, session_seq: &mut u32) -> PacketBuilder<IpmiV2Packet, ()> {
        let ret = PacketBuilder {
            rmcp_header: RmcpHeader::default(),
            session: PhantomData,
            payload_type: PhantomData,
            session_id,
            session_seq: *session_seq,
        };
        *session_seq = session_seq.wrapping_add(1);
        ret
    }

    pub fn ipmi() -> PacketBuilder<IpmiV2Packet, PayloadTypeIpmi> {
        Self::new_with(0, &mut 0)
    }

    pub fn ipmi_encrypted(
        session_id: u32,
        session_seq: &mut u32,
    ) -> PacketBuilder<IpmiV2Packet, PayloadTypeIpmiEncrypted> {
        Self::new_with(session_id, session_seq)
    }

    pub fn sol(
        session_id: u32,
        session_seq: &mut u32,
    ) -> PacketBuilder<IpmiV2Packet, PayloadTypeSol> {
        Self::new_with(session_id, session_seq)
    }

    pub fn open_session() -> PacketBuilder<IpmiV2Packet, PayloadTypeRmcpOpenSession> {
        Self::new()
    }

    pub fn rakp1() -> PacketBuilder<RakpPacket, PayloadTypeRakp1> {
        Self::new()
    }
    pub fn rakp3() -> PacketBuilder<RakpPacket, PayloadTypeRakp3> {
        Self::new()
    }

    fn new_with<T, P>(session_id: u32, session_seq: &mut u32) -> PacketBuilder<T, P> {
        let ret = PacketBuilder {
            rmcp_header: RmcpHeader::default(),
            session_id,
            session_seq: *session_seq,
            session: PhantomData,
            payload_type: PhantomData,
        };
        *session_seq = session_seq.wrapping_add(1);
        ret
    }

    fn new<T, P>() -> PacketBuilder<T, P> {
        PacketBuilder {
            rmcp_header: RmcpHeader::default(),
            session_id: 0,
            session_seq: 0,
            session: PhantomData,
            payload_type: PhantomData,
        }
    }
}

// impl<'a> PacketBuilder<'a, IpmiV1_5Packet> {}

#[derive(Clone, Debug)]
pub enum PacketPayload<'a> {
    IpmiResp(IpmiRespPayload<'a>),
    IpmiReq(IpmiRequestPayload<'a>),
    Rmcp(RMCPPlusOpenSessionPayload),
    Rakp(RakpPayload),
    Sol(SolPayload),
    Raw(&'a [u8]),
    None,
}

impl<'a> PacketPayload<'a> {
    pub fn data(&self) -> Option<&'a [u8]> {
        match self {
            PacketPayload::IpmiResp(resp) => resp.data,
            _ => todo!(),
        }
    }
    pub fn write_to(self, buf: &mut [u8]) -> usize {
        match self {
            PacketPayload::IpmiResp(_) => unreachable!(),
            PacketPayload::IpmiReq(ipmi_req) => ipmi_req.write_to(buf),
            PacketPayload::Rmcp(rmcp) => rmcp.write_to(buf),
            PacketPayload::Rakp(rakp) => rakp.write_to(buf),
            PacketPayload::Sol(_) => todo!(),
            PacketPayload::Raw(_) => todo!(),
            PacketPayload::None => todo!(),
        }
    }
    pub fn with_data(&mut self, data: &'a [u8]) {
        match self {
            PacketPayload::IpmiReq(ipmi_req) => ipmi_req.with_data(data),
            _ => todo!(),
        }
    }
    pub fn data_and_completion(&self) -> (Option<&[u8]>, CompletionCode) {
        if let PacketPayload::IpmiResp(IpmiRespPayload {
            data,
            completion_code,
            ..
        }) = self
        {
            (*data, *completion_code)
        } else {
            (None, CompletionCode::CompletedNormally)
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

// impl From<&SolPayload> for Vec<u8> {
//     fn from(value: &SolPayload) -> Self {
//         let mut v = vec![
//             value.packet_sequence_number,
//             value.acked_packet_number,
//             value.accepted_character_count,
//             value.operation,
//         ];
//         v.extend(value.data.to_owned());
//         v
//     }
// }

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
