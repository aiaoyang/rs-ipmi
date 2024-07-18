use crate::{commands::CommandCode, request::IpmiRequestPayload, IpmiCommand};

pub enum ChassisControl {
    PowerDown,
    PowerUp,
    PowerCycle,
    PowerHardReset,
    PowerPulseDiag,
    PowerAcpiSoft,
}

impl ChassisControl {
    fn to_u8(&self) -> u8 {
        match self {
            ChassisControl::PowerDown => 0x0,
            ChassisControl::PowerUp => 0x1,
            ChassisControl::PowerCycle => 0x2,
            ChassisControl::PowerHardReset => 0x3,
            ChassisControl::PowerPulseDiag => 0x4,
            ChassisControl::PowerAcpiSoft => 0x5,
        }
    }
}

impl IpmiCommand for ChassisControl {
    type Output = ();

    const NET_FN: crate::NetFn = crate::NetFn::Chassis;

    const COMMAND: CommandCode = CommandCode::Raw(0x02);

    fn parse(&self, _data: &[u8]) -> Result<Self::Output, crate::Error> {
        Ok(())
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let d = &[self.to_u8()];
        IpmiRequestPayload::new(Self::NET_FN, Self::COMMAND, Some(d)).write_to(buf)
    }
}
