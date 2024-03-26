use crate::{
    commands::CommandCode,
    err::{ECommand, Error},
    storage::sdr::SensorType,
    ECommandCode,
};

use super::event::{EventDirection, EventGenerator, EventMessageRevision, EventType};

#[derive(Debug, Clone, Copy)]
enum SelRecordType {
    System,
    TimestampedOem(u8),
    NonTimestampedOem(u8),
    Unknown(u8),
}

impl From<u8> for SelRecordType {
    fn from(value: u8) -> Self {
        match value {
            0x02 => Self::System,
            0xC0..=0xDF => Self::TimestampedOem(value),
            0xE0..=0xFF => Self::NonTimestampedOem(value),
            v => Self::Unknown(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RecordId(u16);

#[derive(Debug, Clone)]
pub enum Entry {
    System {
        record_id: RecordId,
        timestamp: u32,
        generator_id: EventGenerator,
        event_message_format: EventMessageRevision,
        sensor_type: SensorType,
        sensor_number: u8,
        event_direction: EventDirection,
        event_type: EventType,
        event_data: [u8; 3],
    },
    OemTimestamped {
        record_id: RecordId,
        ty: u8,
        timestamp: u32,
        manufacturer_id: u32,
        data: [u8; 6],
    },
    OemNotTimestamped {
        record_id: RecordId,
        ty: u8,
        data: [u8; 13],
    },
}

impl Entry {
    pub fn description_with_assetion(&self) -> (&str, bool, String) {
        match &self {
            Entry::System {
                sensor_type,
                event_type,
                event_data,
                event_direction,
                ..
            } => {
                let (s, d) = event_type.description(sensor_type, event_data);
                let reason = if d == event_direction {
                    "".into()
                } else {
                    format!("expect: {:?}, get: {:?}", d, event_direction)
                };
                (s, d == event_direction, reason)
            }
            _ => ("unreacheable", false, "".into()),
        }
    }
    pub fn id(&self) -> u16 {
        if let Entry::System { record_id, .. } = self {
            record_id.0
        } else {
            0
        }
    }
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        if data.len() < 16 {
            Err(ECommand::NotEnoughData(ECommandCode::new(
                CommandCode::Raw(0x43),
                16,
                data.len(),
                data.into(),
            )))?;
        }

        let record_id = RecordId(u16::from_le_bytes([data[0], data[1]]));
        let record_type = SelRecordType::from(data[2]);
        let timestamp = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);

        match record_type {
            SelRecordType::System => {
                let generator_id = EventGenerator::from((data[7], data[8]));
                let event_message_format = EventMessageRevision::from(data[9]);
                let sensor_type = data[10];
                let sensor_number = data[11];
                let event_direction = if (data[12] >> 7) == 0 {
                    EventDirection::Assert
                } else {
                    EventDirection::Deassert
                };
                let event_type = data[12] & 0x7F;
                let event_data = [data[13], data[14], data[15]];
                Ok(Self::System {
                    record_id,
                    timestamp,
                    generator_id,
                    event_message_format,
                    sensor_type: sensor_type.into(),
                    sensor_number,
                    event_direction,
                    event_type: event_type.into(),
                    event_data,
                })
            }
            SelRecordType::TimestampedOem(v) => Ok(Self::OemTimestamped {
                record_id,
                ty: v,
                timestamp,
                manufacturer_id: u32::from_le_bytes([data[7], data[8], data[9], 0]),
                data: [data[10], data[11], data[12], data[13], data[14], data[15]],
            }),
            SelRecordType::NonTimestampedOem(v) => Ok(Self::OemNotTimestamped {
                record_id,
                ty: v,
                data: [
                    data[3], data[4], data[5], data[6], data[7], data[8], data[9], data[10],
                    data[11], data[12], data[13], data[14], data[15],
                ],
            }),
            SelRecordType::Unknown(v) => Err(ECommand::UnknownRecordType(v))?,
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::System {
                record_id,
                timestamp,
                generator_id,
                event_message_format,
                sensor_type,
                sensor_number,
                event_direction,
                event_type,
                event_data,
            } => {
                f.write_fmt(format_args!("record_id: {record_id:?}, timestamp: {timestamp}, generator_id: {generator_id:?}, event_message_format: {event_message_format:?}, sensor_type: {sensor_type:?}, sensor_number: {sensor_number}, event_direction: {event_direction:?}, event_type: {:?}, event_data: 0b{:b}",event_type.description(sensor_type, event_data),event_type.data_to_u32(sensor_type, *event_data)))
            }
            _ => f.write_fmt(format_args!("{self:?}"))
        }
    }
}
