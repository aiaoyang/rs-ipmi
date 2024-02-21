use std::fmt;

use crate::err::LunError;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Lun {
    Bmc,
    Oem1,
    Sms,
    Oem2,
}

impl fmt::Display for Lun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lun::Bmc => write!(f, "BMC"),
            Lun::Oem1 => write!(f, "OEM1"),
            Lun::Sms => write!(f, "SMS"),
            Lun::Oem2 => write!(f, "OEM2"),
        }
    }
}

impl TryFrom<u8> for Lun {
    type Error = LunError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Lun::Bmc),
            0b01 => Ok(Lun::Oem1),
            0b10 => Ok(Lun::Sms),
            0b11 => Ok(Lun::Oem2),
            _ => Err(LunError::UnknownLun(value)),
        }
    }
}

impl From<Lun> for u8 {
    fn from(val: Lun) -> Self {
        match val {
            Lun::Bmc => 0b00,
            Lun::Oem1 => 0b01,
            Lun::Sms => 0b10,
            Lun::Oem2 => 0b11,
        }
    }
}
