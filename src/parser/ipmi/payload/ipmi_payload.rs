use core::fmt;

pub enum AddrType {
    SlaveAddress,
    SoftwareId,
}

impl From<bool> for AddrType {
    fn from(value: bool) -> Self {
        match value {
            false => AddrType::SlaveAddress,
            true => AddrType::SoftwareId,
        }
    }
}

impl From<AddrType> for u8 {
    fn from(val: AddrType) -> Self {
        match val {
            AddrType::SlaveAddress => 0,
            AddrType::SoftwareId => 2,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum SoftwareType {
    Bios,
    SmiHandler,
    SystemManagementSoftware,
    Oem,
    RemoteConsoleSoftware(u8),
    TerminalModeRemoteConsole,
    Reserved(u8),
}

impl fmt::Display for SoftwareType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoftwareType::Bios => write!(f, "BIOS"),
            SoftwareType::SmiHandler => write!(f, "SMI Handler"),
            SoftwareType::SystemManagementSoftware => write!(f, "System Management Software"),
            SoftwareType::Oem => write!(f, "OEM"),
            SoftwareType::RemoteConsoleSoftware(x) => write!(f, "Remote Console Software: {}", x),
            SoftwareType::TerminalModeRemoteConsole => write!(f, "Terminal Mode Remote Console"),
            SoftwareType::Reserved(x) => write!(f, "Reserved: {}", x),
        }
    }
}

impl From<u8> for SoftwareType {
    fn from(value: u8) -> Self {
        match value {
            0x00..=0x0F => SoftwareType::Bios,
            0x10..=0x1F => SoftwareType::SmiHandler,
            0x20..=0x2F => SoftwareType::SystemManagementSoftware,
            0x30..=0x3F => SoftwareType::Oem,
            0x40..=0x46 => SoftwareType::RemoteConsoleSoftware(value - 0x3F),
            0x47 => SoftwareType::TerminalModeRemoteConsole,
            _ => SoftwareType::Reserved(value),
        }
    }
}

impl From<SoftwareType> for u8 {
    fn from(val: SoftwareType) -> Self {
        match val {
            SoftwareType::Bios => 0x00,
            SoftwareType::SmiHandler => 0x10,
            SoftwareType::SystemManagementSoftware => 0x20,
            SoftwareType::Oem => 0x30,
            SoftwareType::RemoteConsoleSoftware(a) => a,
            SoftwareType::TerminalModeRemoteConsole => 0x47,
            SoftwareType::Reserved(a) => a,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum SlaveAddress {
    Bmc,
    Unknown(u8),
}

impl fmt::Display for SlaveAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlaveAddress::Bmc => write!(f, "BMC"),
            SlaveAddress::Unknown(x) => write!(f, "Unknown: {}", x),
        }
    }
}

impl From<u8> for SlaveAddress {
    fn from(value: u8) -> Self {
        match value {
            0x20 => SlaveAddress::Bmc,
            _ => SlaveAddress::Unknown(value),
        }
    }
}

impl From<SlaveAddress> for u8 {
    fn from(val: SlaveAddress) -> Self {
        match val {
            SlaveAddress::Bmc => 0x20,
            SlaveAddress::Unknown(a) => a,
        }
    }
}
