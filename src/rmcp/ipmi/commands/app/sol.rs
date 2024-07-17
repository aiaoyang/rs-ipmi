use crate::{
    commands::CommandCode, request::IpmiRequestPayload, ECommand, IpmiCommand, NetFn,
    PacketPayload, SolPayload,
};

pub struct ActivatePayloadCommand<'a> {
    pub data: Option<&'a [u8]>,
}

#[derive(Debug)]
pub struct ActivatePayloadResp {
    pub in_payload_size: u16,
    pub out_payload_size: u16,
    pub udp_port: u16,
}

impl<'a> IpmiCommand for ActivatePayloadCommand<'a> {
    type Output = ActivatePayloadResp;

    const NET_FN: NetFn = NetFn::App;
    const COMMAND: CommandCode = CommandCode::ActivatePayload;

    fn parse(&self, data: &[u8]) -> Result<Self::Output, crate::Error> {
        if data.len() != 12 {
            Err(ECommand::NotEnoughData(crate::ECommandCode {
                command: Self::COMMAND,
                expected_len: 12,
                get_len: data.len(),
                data: data.to_vec(),
            }))?
        }
        let in_payload_size: u16 = data[4] as u16 + ((data[5] as u16) << 8);
        let out_payload_size: u16 = data[6] as u16 + ((data[7] as u16) << 8);
        let udp_port: u16 = data[8] as u16 + ((data[9] as u16) << 8);
        println!("data: {data:?}, in_payload_size: {in_payload_size}, out_payload_size: {out_payload_size}, udp_port: {udp_port}");
        let res = ActivatePayloadResp {
            in_payload_size,
            out_payload_size,
            udp_port,
        };
        Ok(res)
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        // let data_payload = [0x01, 0x01, 0x08, 0, 0, 0];
        let data_payload = [0x01, 0x01, 0x80 | 0x40 | 0x04 | 0x02, 0, 0, 0];
        let packet_data = if let Some(data) = self.data {
            PacketPayload::Sol(SolPayload::new(data.to_vec()))
        } else {
            PacketPayload::IpmiReq(IpmiRequestPayload::new(
                Self::NET_FN,
                Self::COMMAND,
                Some(&data_payload),
            ))
        };
        let p = if self.data.is_some() {
            crate::Packet::new(
                crate::RmcpHeader::default(),
                crate::IpmiSession::V2_0(crate::IpmiV2Session::new_sol_payload(
                    data_payload.len() as u16
                )),
                packet_data,
            )
        } else {
            crate::Packet::new(
                crate::RmcpHeader::default(),
                crate::IpmiSession::V2_0(crate::IpmiV2Session::new_sol_activate(
                    data_payload.len() as u16,
                )),
                packet_data,
            )
        };
        p.write_to(buf)
    }
}
