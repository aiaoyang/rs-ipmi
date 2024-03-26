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

    fn netfn() -> crate::NetFn {
        crate::NetFn::Storage
    }

    fn command() -> CommandCode {
        CommandCode::Raw(0x22)
    }

    fn payload(&self) -> Payload {
        Payload::IpmiReq(ReqPayload::new(Self::netfn(), Self::command(), Vec::new()))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Error> {
        if data.len() < 2 {
            println!("data: {:?}", data);
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
