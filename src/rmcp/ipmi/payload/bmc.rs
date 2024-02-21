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

#[derive(Clone, Debug, Copy)]
pub enum SlaveAddress {
    Bmc,
    Unknown(u8),
}

impl From<u8> for SlaveAddress {
    fn from(value: u8) -> Self {
        match value {
            0x20 => SlaveAddress::Bmc,
            _ => SlaveAddress::Unknown(value),
        }
    }
}
