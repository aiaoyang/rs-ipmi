use std::marker::PhantomData;

use crate::{ECommand, Error};

use super::{
    capabilities::SensorCapabilities, event_reading_type_code::EventReadingTypeCodes,
    units::SensorUnits, SensorId, SensorKey, SensorType,
};

pub struct IdUnset;
#[derive(Debug, Clone)]
pub struct IdSettled;

#[derive(Debug, Clone)]
pub struct SensorRecordCommon<T> {
    pub key: SensorKey,
    // TODO: make a type EntityId
    pub entity_id: u8,
    pub entity_instance: EntityInstance,
    pub initialization: SensorInitialization,
    pub capabilities: SensorCapabilities,
    pub sensor_type: SensorType,
    pub event_reading_type_code: EventReadingTypeCodes,
    pub sensor_units: SensorUnits,
    pub sensor_id: SensorId,
    _p: PhantomData<T>,
}

impl SensorRecordCommon<IdUnset> {
    /// Parse common sensor record data, but set the SensorID to an empty UTF-8 String.
    ///
    /// You _must_ remember to [`SensorRecordCommon::set_id`] once the ID of the
    /// record has been parsed.
    pub(crate) fn parse_without_id(record_data: &[u8]) -> Result<(Self, &[u8]), Error> {
        if record_data.len() < 17 {
            Err(ECommand::Parse(
                "SensorRecordCommon NotEnoughData: 17".into(),
            ))?
        }

        let sensor_key = SensorKey::parse(&record_data[..3])?;
        let entity_id = record_data[3];
        let entity_instance = EntityInstance::from(record_data[4]);
        let initialization = SensorInitialization::from(record_data[5]);
        let sensor_capabilities = record_data[6];
        let sensor_type = record_data[7].into();
        let event_reading_type_code = record_data[8].into();

        let assertion_event_mask_lower_thrsd_reading_mask =
            u16::from_le_bytes([record_data[9], record_data[10]]);
        let deassertion_event_mask_upper_thrsd_reading_mask =
            u16::from_le_bytes([record_data[11], record_data[12]]);
        let settable_thrsd_readable_thrsd_mask =
            u16::from_le_bytes([record_data[13], record_data[14]]);

        let capabilities = SensorCapabilities::new(
            sensor_capabilities,
            assertion_event_mask_lower_thrsd_reading_mask,
            deassertion_event_mask_upper_thrsd_reading_mask,
            settable_thrsd_readable_thrsd_mask,
        );

        let sensor_units_1 = record_data[15];
        let base_unit = record_data[16];
        let modifier_unit = record_data[17];

        let sensor_units = SensorUnits::from(sensor_units_1, base_unit, modifier_unit);

        Ok((
            Self {
                key: sensor_key,
                entity_id,
                entity_instance,
                initialization,
                capabilities,
                sensor_type,
                event_reading_type_code,
                sensor_units,
                sensor_id: Default::default(),
                _p: Default::default(),
            },
            &record_data[18..],
        ))
    }

    pub(crate) fn set_id(self, id: SensorId) -> SensorRecordCommon<IdSettled> {
        let sensor_id = id;
        let Self {
            key,
            entity_id,
            entity_instance,
            initialization,
            capabilities,
            sensor_type: ty,
            event_reading_type_code,
            sensor_units,
            ..
        } = self;
        SensorRecordCommon {
            key,
            entity_id,
            entity_instance,
            initialization,
            capabilities,
            sensor_type: ty,
            event_reading_type_code,
            sensor_units,
            sensor_id,
            _p: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorInitialization {
    pub settable: bool,
    pub scanning: bool,
    pub events: bool,
    pub thresholds: bool,
    pub hysteresis: bool,
    pub sensor_type: bool,
    pub event_generation_enabled_on_startup: bool,
    pub sensor_scanning_enabled_on_startup: bool,
}

bitflags::bitflags! {
    pub struct Flags: u8 {
        const SETTABLE = 1 << 7;
        const SCANNING = 1 << 6;
        const EVENTS = 1 << 5;
        const THRESHOLDS = 1 << 4;
        const HYSTERESIS = 1 << 3;
        const TYPE = 1 << 2;
        const EVENTGEN_ON_STARTUP = 1 << 1;
        const SCANNING_ON_STARTUP = 1 << 0;
    }
}

impl From<u8> for SensorInitialization {
    fn from(value: u8) -> Self {
        let flags = Flags::from_bits_truncate(value);

        Self {
            settable: flags.contains(Flags::SETTABLE),
            scanning: flags.contains(Flags::SCANNING),
            events: flags.contains(Flags::EVENTS),
            thresholds: flags.contains(Flags::THRESHOLDS),
            hysteresis: flags.contains(Flags::THRESHOLDS),
            sensor_type: flags.contains(Flags::TYPE),
            event_generation_enabled_on_startup: flags.contains(Flags::EVENTGEN_ON_STARTUP),
            sensor_scanning_enabled_on_startup: flags.contains(Flags::SCANNING_ON_STARTUP),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EntityRelativeTo {
    System,
    Device,
}

#[derive(Debug, Clone, Copy)]
pub enum EntityInstance {
    Physical {
        relative: EntityRelativeTo,
        instance_number: u8,
    },
    LogicalContainer {
        relative: EntityRelativeTo,
        instance_number: u8,
    },
}

impl From<u8> for EntityInstance {
    fn from(value: u8) -> Self {
        let instance_number = value & 0x7F;
        let relative = match instance_number {
            0x00..=0x5F => EntityRelativeTo::System,
            0x60..=0x7F => EntityRelativeTo::Device,
            _ => unreachable!(),
        };

        if (value & 0x80) == 0x80 {
            Self::LogicalContainer {
                relative,
                instance_number,
            }
        } else {
            Self::Physical {
                relative,
                instance_number,
            }
        }
    }
}
