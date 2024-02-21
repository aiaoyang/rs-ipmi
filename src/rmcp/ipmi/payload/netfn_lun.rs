use std::fmt;

use crate::err::LunError;

pub type RqseqLun = NetfnLun;
impl RqseqLun {
    pub fn rqseq(&self) -> u8 {
        self.0 >> 2
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetfnLun(pub u8);

impl std::fmt::Display for NetfnLun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl From<u8> for NetfnLun {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl NetfnLun {
    const IPMB_LUN_MASK: u8 = 0x03;
    pub fn new(netfn: impl Into<u8>, lun: Lun) -> Self {
        Self(netfn.into() << 2 | lun as u8 & Self::IPMB_LUN_MASK)
    }

    pub fn netfn(&self) -> NetFn {
        (self.0 >> 2).into()
    }
    pub fn lun(&self) -> Result<Lun, LunError> {
        Lun::new(self.0 & Self::IPMB_LUN_MASK)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NetFn {
    Chassis,
    Bridge,
    SensorEvent,
    App,
    Firmware,
    Storage,
    Transport,
    Reserved,
    Unknown(u8),
}

impl fmt::Display for NetFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetFn::Chassis => write!(f, "Chassis"),
            NetFn::Bridge => write!(f, "Bridge"),
            NetFn::SensorEvent => write!(f, "Sensor Event"),
            NetFn::App => write!(f, "App"),
            NetFn::Firmware => write!(f, "Firmware"),
            NetFn::Storage => write!(f, "Storage"),
            NetFn::Transport => write!(f, "Transport"),
            NetFn::Reserved => write!(f, "Reserved"),
            NetFn::Unknown(x) => write!(f, "Unknown: {}", x),
        }
    }
}

impl From<u8> for NetFn {
    fn from(value: u8) -> Self {
        match value {
            0x00..=0x01 => NetFn::Chassis,
            0x02..=0x03 => NetFn::Bridge,
            0x04..=0x05 => NetFn::SensorEvent,
            0x06..=0x07 => NetFn::App,
            0x08..=0x09 => NetFn::Firmware,
            0x0A..=0x0B => NetFn::Storage,
            0x0C..=0x0D => NetFn::Transport,
            0x0E..=0x2B => NetFn::Reserved,
            _ => NetFn::Unknown(value),
        }
    }
}

impl From<NetFn> for u8 {
    fn from(value: NetFn) -> u8 {
        match value {
            NetFn::Chassis => 0x00,
            NetFn::Bridge => 0x02,
            NetFn::SensorEvent => 0x04,
            NetFn::App => 0x06,
            NetFn::Firmware => 0x08,
            NetFn::Storage => 0x0A,
            NetFn::Transport => 0x0C,
            NetFn::Reserved => 0x0E,
            NetFn::Unknown(fn_code) => fn_code,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Lun {
    Bmc,
    Oem1,
    Sms,
    Oem2,
}

impl Lun {
    pub fn new(value: u8) -> Result<Self, LunError> {
        match value {
            0b00 => Ok(Lun::Bmc),
            0b01 => Ok(Lun::Oem1),
            0b10 => Ok(Lun::Sms),
            0b11 => Ok(Lun::Oem2),
            _ => Err(LunError::UnknownLun(value)),
        }
    }
}
