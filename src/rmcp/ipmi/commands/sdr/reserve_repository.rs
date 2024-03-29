use crate::{
    commands::CommandCode, request::ReqPayload, ECommand, ECommandCode, Error, IpmiCommand, Payload,
};

pub struct ReserveSDRRepositoryCommand;

#[derive(Debug)]
pub struct ReserveSDRRepository {
    pub reservation_id: u16,
}

impl IpmiCommand for ReserveSDRRepositoryCommand {
    type Output = ReserveSDRRepository;

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn command(&self) -> CommandCode {
        CommandCode::Raw(0x22)
    }

    fn payload(&self) -> Payload {
        Payload::IpmiReq(ReqPayload::new(self.netfn(), self.command(), Vec::new()))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 2 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                CommandCode::Raw(0x22),
                2,
                data.len(),
                data.into(),
            )))?
        }
        Ok(ReserveSDRRepository {
            reservation_id: u16::from_le_bytes([data[0], data[1]]),
        })
    }
}
