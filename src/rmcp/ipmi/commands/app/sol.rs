use crate::{request::ReqPayload, ECommand, IpmiCommand, Payload};

pub struct ActivatePayloadCommand {}

#[derive(Debug)]
pub struct ActivatePayloadResp {
    pub in_payload_size: u16,
    pub out_payload_size: u16,
    pub udp_port: u16,
}

impl IpmiCommand for ActivatePayloadCommand {
    type Output = ActivatePayloadResp;

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::App
    }

    fn command(&self) -> crate::commands::CommandCode {
        crate::commands::CommandCode::ActivatePayload
    }

    fn payload(&self) -> crate::Payload {
        let payload = [0x01, 0x01, 0x04, 0, 0, 0];
        Payload::IpmiReq(ReqPayload::new(
            self.netfn(),
            self.command(),
            payload.to_vec(),
        ))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, crate::Error> {
        if data.len() != 12 {
            Err(ECommand::NotEnoughData(crate::ECommandCode {
                command: self.command(),
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
}
