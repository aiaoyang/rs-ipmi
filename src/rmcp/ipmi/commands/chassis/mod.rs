pub mod boot_opt;
pub mod control;

use crate::{
    request::IpmiRequestPayload, ECommand, ECommandCode, IpmiCommand, Packet, PacketPayload,
};

pub struct GetChassisStatus;

#[allow(dead_code)]
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
    const NET_FN: crate::NetFn = crate::NetFn::Chassis;
    const COMMAND: super::CommandCode = super::CommandCode::Chassis;

    fn parse(&self, data: &[u8]) -> Result<Self::Output, crate::Error> {
        if data.len() < 3 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                Self::COMMAND,
                3,
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

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let data =
            PacketPayload::IpmiReq(IpmiRequestPayload::new(Self::NET_FN, Self::COMMAND, None));
        Packet::new_with_data(data).write_to(buf)
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
    const NET_FN: crate::NetFn = crate::NetFn::Chassis;
    const COMMAND: super::CommandCode = super::CommandCode::Raw(0x02);
    fn parse(&self, _data: &[u8]) -> Result<Self::Output, crate::Error> {
        Ok(())
    }
    fn write_to(&self, buf: &mut [u8]) -> usize {
        let d = &[*self as u8];
        let data = PacketPayload::IpmiReq(IpmiRequestPayload::new(
            Self::NET_FN,
            Self::COMMAND,
            Some(d),
        ));
        Packet::new_with_data(data).write_to(buf)
    }
}
