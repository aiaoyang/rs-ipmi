use crate::{commands::CommandCode, request::ReqPayload, IpmiCommand, Payload};

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

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Chassis
    }

    fn command(&self) -> crate::commands::CommandCode {
        CommandCode::Raw(0x02)
    }

    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(
            self.netfn(),
            self.command(),
            Vec::from(&[self.to_u8()]),
        ))
    }

    fn parse(&self, _data: &[u8]) -> Result<Self::Output, crate::Error> {
        Ok(())
    }
}
