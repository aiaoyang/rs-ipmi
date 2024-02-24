use crate::rmcp::Lun;

use super::constant::{SENSOR_GENERIC_EVENT_DESC, SENSOR_SPECIFIC_EVENT_DESC};

#[derive(Debug, Clone)]
pub enum EventType {
    Unspecified(u8),
    Threshold(u8),
    Generic(u8),
    SensorSpecific(u8),
    Oem(u8),
    Unknown(u8),
}
impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Unspecified(0x00),
            0x01 => Self::Threshold(0x01),
            0x02..=0x0c => Self::Generic(value),
            0x6f => Self::SensorSpecific(0x6f),
            0x70..=0x7f => Self::Oem(value),
            _ => Self::Unknown(value),
        }
    }
}

impl EventType {
    pub fn description(
        &self,
        sensor_type: impl Into<u8> + std::marker::Copy,
        data: [u8; 3],
    ) -> &'static str {
        match self {
            EventType::Threshold(event_type) | EventType::Generic(event_type) => {
                let offset = data[0] & 0x0f;
                SENSOR_GENERIC_EVENT_DESC
                    .get(&((*event_type as u32) << 8 | offset as u32))
                    .unwrap_or(&"generic unknown")
            }
            EventType::SensorSpecific(_) => {
                let d1: u8 = data[0];
                let mut d2: u8 = if d1 & 0xc0 != 0 { data[1] } else { 0xff };
                let mut d3: u8 = if d1 & 0x30 != 0 { data[2] } else { 0xff };
                let offset = d1 & 0x0f;

                for _ in 0..2 {
                    if let Some(desc) = SENSOR_SPECIFIC_EVENT_DESC.get(
                        &((sensor_type.into() as u32) << 24
                            | (offset as u32) << 16
                            | (d2 as u32) << 8
                            | (d3 as u32)),
                    ) {
                        return desc;
                    }
                    if d2 != 0xff || d3 != 0xff {
                        (d2, d3) = (0xff, 0xff);
                    }
                }
                "specific not found"
            }
            EventType::Oem(_) => "oem",
            EventType::Unknown(_) | EventType::Unspecified(_) => "other unknown",
        }
    }

    pub fn data_to_u32(
        &self,
        sensor_type: impl Into<u8> + std::marker::Copy,
        data: [u8; 3],
    ) -> u32 {
        match self {
            EventType::Threshold(event_type) | EventType::Generic(event_type) => {
                let offset = data[0] & 0x0f;
                (*event_type as u32) << 8 | offset as u32
            }
            EventType::SensorSpecific(_) => {
                let d1: u8 = data[0];
                let mut d2: u8 = if d1 & 0xc0 != 0 { data[1] } else { 0xff };
                let mut d3: u8 = if d1 & 0x30 != 0 { data[2] } else { 0xff };
                let offset = d1 & 0x0f;

                for _ in 0..2 {
                    if SENSOR_SPECIFIC_EVENT_DESC
                        .get(
                            &((sensor_type.into() as u32) << 24
                                | (offset as u32) << 16
                                | (d2 as u32) << 8
                                | (d3 as u32)),
                        )
                        .is_some()
                    {
                        return (sensor_type.into() as u32) << 24
                            | (offset as u32) << 16
                            | (d2 as u32) << 8
                            | (d3 as u32);
                    }
                    if d2 != 0xff || d3 != 0xff {
                        (d2, d3) = (0xff, 0xff);
                    }
                }
                0
            }
            EventType::Oem(oem) => *oem as u32,
            EventType::Unknown(v) | EventType::Unspecified(v) => *v as u32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventMessageRevision {
    V2_0,
    V1_0,
    Unknown(u8),
}

impl From<u8> for EventMessageRevision {
    fn from(value: u8) -> Self {
        match value {
            0x04 => Self::V2_0,
            0x03 => Self::V1_0,
            v => Self::Unknown(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventDirection {
    Assert,
    Deassert,
}

#[derive(Debug, Clone, Copy)]
pub enum EventGenerator {
    RqSAAndLun {
        i2c_addr: u8,
        channel_number: u8,
        lun: Lun,
    },
    SoftwareId {
        software_id: u8,
        channel_number: u8,
    },
}

impl From<(u8, u8)> for EventGenerator {
    fn from(value: (u8, u8)) -> Self {
        let is_software_id = (value.0 & 0x1) == 0x1;
        let i2c_or_sid = (value.0 >> 1) & 0x7F;
        let channel_number = (value.1 >> 4) & 0xF;

        if is_software_id {
            Self::SoftwareId {
                software_id: i2c_or_sid,
                channel_number,
            }
        } else {
            let lun = Lun::new(value.1 & 0x3).unwrap();

            Self::RqSAAndLun {
                i2c_addr: i2c_or_sid,
                channel_number,
                lun,
            }
        }
    }
}
