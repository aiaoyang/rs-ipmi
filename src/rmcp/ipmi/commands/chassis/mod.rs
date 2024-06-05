pub mod boot_opt;
pub mod control;

use crate::{request::ReqPayload, ECommand, ECommandCode, IpmiCommand, Payload};

pub struct GetChassisStatus;

#[derive(Debug)]
pub struct GetChassisStatusResp {
    pub p_state: PowerState,
    power_state: u8,
    last_power_event: u8,
    state: u8,
    front_control_panel: u8,
}

const SYSTEM_POWER: u8 = 0x01;

#[derive(Debug)]
pub enum PowerState {
    On,
    Off,
}
impl From<u8> for PowerState {
    fn from(value: u8) -> Self {
        if value & SYSTEM_POWER == SYSTEM_POWER {
            Self::On
        } else {
            Self::Off
        }
    }
}
impl From<PowerState> for &'static str {
    fn from(value: PowerState) -> Self {
        match value {
            PowerState::On => "Power is on",
            PowerState::Off => "Power is off",
        }
    }
}

impl IpmiCommand for GetChassisStatus {
    type Output = GetChassisStatusResp;
    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Chassis
    }
    fn parse(&self, data: &[u8]) -> Result<Self::Output, crate::Error> {
        if data.len() < 4 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                self.command(),
                14,
                data.len(),
                data.into(),
            )))?
        }
        Ok(Self::Output {
            p_state: PowerState::from(data[0]),
            power_state: data[0],
            last_power_event: data[1],
            state: data[2],
            front_control_panel: if data.len() > 3 { data[3] } else { 0 },
        })
    }
    fn command(&self) -> super::CommandCode {
        super::CommandCode::Raw(0x01)
    }
    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(self.netfn(), self.command(), Vec::new()))
    }
}

#[derive(Clone, Copy)]
pub enum ChassisControl {
    PowerDown = 0,
    PowerUp = 1,
    PowerCycle = 2,
    PowerHardReset = 3,
    PowerPulseDiag = 4,
    PowerAcpiSoft = 5,
}

impl IpmiCommand for ChassisControl {
    type Output = ();
    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Chassis
    }
    fn parse(&self, _data: &[u8]) -> Result<Self::Output, crate::Error> {
        Ok(())
    }
    fn command(&self) -> super::CommandCode {
        super::CommandCode::Raw(0x02)
    }
    fn payload(&self) -> Payload {
        Payload::IpmiReq(ReqPayload::new(
            self.netfn(),
            self.command(),
            Vec::from(&[*self as u8]),
        ))
    }
}
