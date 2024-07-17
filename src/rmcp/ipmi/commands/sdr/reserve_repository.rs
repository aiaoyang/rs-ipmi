use crate::{
    commands::CommandCode, request::IpmiRequestPayload, ECommand, ECommandCode, Error, IpmiCommand,
    NetFn, Packet, PacketPayload,
};

pub struct GetReserveSDRRepository;

#[derive(Debug)]
pub struct ReserveSDRRepository {
    pub reservation_id: u16,
}

impl IpmiCommand for GetReserveSDRRepository {
    type Output = ReserveSDRRepository;

    const NET_FN: NetFn = NetFn::Storage;
    const COMMAND: CommandCode = CommandCode::Raw(0x22);

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

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let data =
            PacketPayload::IpmiReq(IpmiRequestPayload::new(Self::NET_FN, Self::COMMAND, None));
        Packet::new_with_data(data).write_to(buf)
    }
}
