use std::fmt;

use crate::err::LunError;

use super::lun::Lun;

pub enum CommandType {
    Request,
    Response,
}

impl From<u8> for CommandType {
    fn from(value: u8) -> Self {
        if value % 2 == 0 {
            CommandType::Request
        } else {
            CommandType::Response
        }
    }
}

pub type RqseqLun = NetfnLun;

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
    pub fn new(netfn: u8, lun: u8) -> Self {
        Self(netfn << 2 | lun & Self::IPMB_LUN_MASK)
    }

    pub fn netfn(&self) -> NetFn {
        (self.0 >> 2).into()
    }
    pub fn lun(&self) -> Result<Lun, LunError> {
        (self.0 & Self::IPMB_LUN_MASK).try_into()
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

impl NetFn {
    pub fn to_u8(&self, command_type: CommandType) -> u8 {
        match (self, command_type) {
            (NetFn::Chassis, CommandType::Request) => 0x00,
            (NetFn::Chassis, CommandType::Response) => 0x01,
            (NetFn::Bridge, CommandType::Request) => 0x02,
            (NetFn::Bridge, CommandType::Response) => 0x03,
            (NetFn::SensorEvent, CommandType::Request) => 0x04,
            (NetFn::SensorEvent, CommandType::Response) => 0x05,
            (NetFn::App, CommandType::Request) => 0x06,
            (NetFn::App, CommandType::Response) => 0x07,
            (NetFn::Firmware, CommandType::Request) => 0x08,
            (NetFn::Firmware, CommandType::Response) => 0x09,
            (NetFn::Storage, CommandType::Request) => 0x0A,
            (NetFn::Storage, CommandType::Response) => 0x0B,
            (NetFn::Transport, CommandType::Request) => 0x0C,
            (NetFn::Transport, CommandType::Response) => 0x0D,
            (NetFn::Reserved, CommandType::Request) => 0x0E,
            (NetFn::Reserved, CommandType::Response) => 0x2B,
            (NetFn::Unknown(fn_code), _) => *fn_code,
        }
    }
}
