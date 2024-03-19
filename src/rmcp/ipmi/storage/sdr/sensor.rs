use nonmax::NonMaxU8;

use crate::{ECommand, Error, Lun};

use super::Channel;

#[derive(Debug, Clone, PartialEq)]
pub enum SensorId {
    Unicode(String),
    BCDPlus(Vec<u8>),
    Ascii6BPacked(Vec<u8>),
    Ascii8BAndLatin1(String),
}

type TypeLengthRaw<'a> = (u8, &'a [u8]);
impl<'a> From<TypeLengthRaw<'a>> for SensorId {
    fn from(value: TypeLengthRaw<'a>) -> Self {
        let (value, data) = value;
        let type_code = (value >> 6) & 0x3;

        let length = value & 0x1F;

        let data = &data[..(length as usize).min(data.len())];

        let str = core::str::from_utf8(data).map(ToString::to_string);

        match type_code {
            0b00 => SensorId::Unicode(str.unwrap()),
            0b01 => SensorId::BCDPlus(data.to_vec()),
            0b10 => SensorId::Ascii6BPacked(data.to_vec()),
            0b11 => SensorId::Ascii8BAndLatin1(str.unwrap()),
            _ => unreachable!(),
        }
    }
}

impl core::fmt::Display for SensorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorId::Unicode(v) => write!(f, "{}", v),
            SensorId::Ascii8BAndLatin1(v) => write!(f, "{}", v),
            _ => todo!(),
        }
    }
}

impl Default for SensorId {
    fn default() -> Self {
        Self::Unicode("".into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorNumber(pub NonMaxU8);

impl SensorNumber {
    pub fn new(value: NonMaxU8) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u8 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorOwner {
    I2C(u8),
    System(u8),
}

impl From<u8> for SensorOwner {
    fn from(value: u8) -> Self {
        let id = (value & 0xFE) >> 1;

        if (value & 1) == 1 {
            Self::System(id)
        } else {
            Self::I2C(id)
        }
    }
}

impl From<SensorOwner> for u8 {
    fn from(val: SensorOwner) -> Self {
        match val {
            SensorOwner::I2C(id) => (id << 1) & 0xFE,
            SensorOwner::System(id) => ((id << 1) & 0xFE) | 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorKey {
    pub owner_id: SensorOwner,
    pub owner_channel: Channel,
    pub fru_inv_device_owner_lun: Lun,
    pub owner_lun: Lun,
    pub sensor_number: SensorNumber,
}

impl SensorKey {
    pub fn parse(record_data: &[u8]) -> Result<Self, Error> {
        if record_data.len() != 3 {
            Err(ECommand::Parse("SensorKey NotEnoughData: 3".into()))?
        }

        let owner_id = SensorOwner::from(record_data[0]);
        let owner_channel_fru_lun = record_data[1];
        let owner_channel = Channel::new((owner_channel_fru_lun & 0xF0) >> 4).unwrap();
        let fru_inv_device_owner_lun = Lun::try_from((owner_channel_fru_lun >> 2) & 0x3).unwrap();
        let owner_lun = Lun::try_from(owner_channel_fru_lun & 0x3).unwrap();

        let sensor_number = SensorNumber(
            NonMaxU8::new(record_data[2])
                .ok_or(ECommand::Parse("Create SensorNumber get 0xff".into()))?,
        );

        Ok(Self {
            owner_id,
            owner_channel,
            fru_inv_device_owner_lun,
            owner_lun,
            sensor_number,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Linearization {
    Linear,
    Ln,
    Log10,
    Log2,
    E,
    Exp10,
    Exp2,
    OneOverX,
    Sqr,
    Cube,
    Sqrt,
    CubeRoot,
    Oem(u8),
    Unknown(u8),
}

impl From<u8> for Linearization {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::Ln,
            2 => Self::Log10,
            3 => Self::Log2,
            4 => Self::E,
            5 => Self::Exp10,
            6 => Self::Exp2,
            7 => Self::OneOverX,
            8 => Self::Sqr,
            9 => Self::Sqrt,
            10 => Self::Cube,
            11 => Self::Sqrt,
            12 => Self::CubeRoot,
            0x71..=0x7F => Self::Oem(value),
            v => Self::Unknown(v),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    UnspecifiedNotApplicable,
    Input,
    Output,
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let dir = match value {
            0b00 => Self::UnspecifiedNotApplicable,
            0b01 => Self::Input,
            0b10 => Self::Output,
            _ => return Err(()),
        };
        Ok(dir)
    }
}
