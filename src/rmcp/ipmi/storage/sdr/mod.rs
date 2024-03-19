pub use address::*;
pub use compact_sensor_record::CompactSensorRecord;
pub use full_sensor_record::FullSensorRecord;
pub use sensor::*;
pub use sensor_type::SensorType;

mod address;
mod capabilities;
mod common;
mod compact_sensor_record;
mod event_reading_type_code;
mod full_sensor_record;
mod sensor;
mod sensor_type;
mod units;
mod units_macro;

use self::{
    capabilities::SensorCapabilities,
    common::{IdSettled, SensorRecordCommon},
};
use crate::{commands::CommandCode, ECommand, Error};

pub trait SensorRecord {
    fn common(&self) -> &SensorRecordCommon<IdSettled>;

    fn capabilities(&self) -> &SensorCapabilities {
        &self.common().capabilities
    }

    fn id_string(&self) -> &SensorId {
        &self.common().sensor_id
    }

    fn direction(&self) -> Direction;

    fn sensor_number(&self) -> SensorNumber {
        self.common().key.sensor_number
    }

    fn entity_id(&self) -> u8 {
        self.common().entity_id
    }

    fn key_data(&self) -> &SensorKey {
        &self.common().key
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RecordId(u16);

impl RecordId {
    pub const FIRST: Self = Self(0);
    pub const LAST: Self = Self(0xFFFF);

    pub fn new_raw(value: u16) -> Self {
        Self(value)
    }

    pub fn is_first(&self) -> bool {
        self.0 == Self::FIRST.0
    }

    pub fn is_last(&self) -> bool {
        self.0 == Self::LAST.0
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RecordHeader {
    pub id: RecordId,

    pub sdr_version_major: u8,
    pub sdr_version_minor: u8,
}

#[derive(Debug, Clone)]
pub enum RecordContents {
    FullSensor(FullSensorRecord),
    CompactSensor(CompactSensorRecord),
    Unknown { ty: u8, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct SdrRecord {
    pub header: RecordHeader,
    pub contents: RecordContents,
}

impl SdrRecord {
    pub fn common_data(&self) -> Option<&SensorRecordCommon<IdSettled>> {
        match &self.contents {
            RecordContents::FullSensor(s) => Some(s.common()),
            RecordContents::CompactSensor(s) => Some(s.common()),
            RecordContents::Unknown { .. } => None,
        }
    }

    pub fn full_sensor(&self) -> Option<&FullSensorRecord> {
        if let RecordContents::FullSensor(full_sensor) = &self.contents {
            Some(full_sensor)
        } else {
            None
        }
    }

    pub fn compact_sensor(&self) -> Option<&CompactSensorRecord> {
        if let RecordContents::CompactSensor(compact_sensor) = &self.contents {
            Some(compact_sensor)
        } else {
            None
        }
    }

    pub fn parse(data: &[u8], cmd: CommandCode) -> Result<Self, Error> {
        if data.len() < 5 {
            return Err(ECommand::NotEnoughData {
                command: cmd,
                expected_len: 5,
                get_len: data.len(),
                data: data.into(),
            })?;
        }

        let record_id = RecordId::new_raw(u16::from_le_bytes([data[0], data[1]]));
        let sdr_version_min = (data[2] & 0xF0) >> 4;
        let sdr_version_maj = data[2] & 0x0F;
        let record_type = data[3];
        let record_length = data[4];

        let record_data = &data[5..];
        if record_data.len() != record_length as usize {
            return Err(ECommand::NotEnoughData {
                command: CommandCode::Raw(0x23),
                expected_len: record_length,
                get_len: data.len(),
                data: data.into(),
            })?;
        }

        let contents = if record_type == 0x01 {
            RecordContents::FullSensor(FullSensorRecord::parse(record_data)?)
        } else if record_type == 0x02 {
            RecordContents::CompactSensor(CompactSensorRecord::parse(record_data)?)
        } else {
            RecordContents::Unknown {
                ty: record_type,
                data: record_data.to_vec(),
            }
        };

        Ok(Self {
            header: RecordHeader {
                id: record_id,
                sdr_version_minor: sdr_version_min,
                sdr_version_major: sdr_version_maj,
            },
            contents,
        })
    }

    pub fn id(&self) -> Option<&SensorId> {
        match &self.contents {
            RecordContents::FullSensor(full) => Some(full.id_string()),
            RecordContents::CompactSensor(compact) => Some(compact.id_string()),
            RecordContents::Unknown { .. } => None,
        }
    }

    pub fn sensor_number(&self) -> Option<SensorNumber> {
        match &self.contents {
            RecordContents::FullSensor(full) => Some(full.sensor_number()),
            RecordContents::CompactSensor(compact) => Some(compact.sensor_number()),
            RecordContents::Unknown { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_owner_round_trip() {
        for x in 0u8..=255u8 {
            let o = SensorOwner::from(x);
            let value: u8 = o.into();
            assert_eq!(x, value);
        }
    }
}
