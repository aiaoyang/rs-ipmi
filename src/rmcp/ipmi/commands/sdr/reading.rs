use crate::commands::CommandCode;
use crate::request::ReqPayload;
use crate::storage::sdr::{Address, Channel, SensorKey};
use crate::{storage::sdr::SensorNumber, IpmiCommand};
use crate::{ECommand, Error, Payload};

pub struct GetSensorReading {
    sensor_number: SensorNumber,
    #[allow(unused)]
    address: Address,
    #[allow(unused)]
    channel: Channel,
}
impl GetSensorReading {
    pub fn new(sensor_number: SensorNumber, address: Address, channel: Channel) -> Self {
        Self {
            sensor_number,
            address,
            channel,
        }
    }

    pub fn form_sensor_key(sensor_key: &SensorKey) -> Self {
        Self {
            sensor_number: sensor_key.sensor_number,
            address: Address(sensor_key.owner_id.into()),
            channel: sensor_key.owner_channel,
        }
    }
}

impl IpmiCommand for GetSensorReading {
    type Output = RawSensorReading;

    type Error = Error;

    fn netfn() -> crate::NetFn {
        crate::NetFn::SensorEvent
    }

    fn command() -> crate::commands::CommandCode {
        CommandCode::Raw(0x2d)
    }

    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(
            Self::netfn(),
            Self::command(),
            vec![self.sensor_number.get()],
        ))
    }

    fn parse(&self, data: &[u8]) -> Result<Self::Output, Self::Error> {
        if data.len() < 2 {
            Err(ECommand::Parse("RawSensorReading: 2".into()))?
        }

        let reading = data[0];

        // Bit indicates that all event messages are enabled => must negate result
        let all_event_messages_disabled = (data[1] & 0x80) != 0x80;

        // Bit indicates that sensor scanning is enabled => must negate result
        let scanning_disabled = (data[1] & 0x40) != 0x40;

        let reading_or_state_unavailable = (data[1] & 0x20) == 0x20;

        let offset_data_1 = data.get(2).copied();
        let map = data.get(3).copied();
        let offset_data_2 = map;

        Ok(RawSensorReading {
            reading,
            all_event_messages_disabled,
            scanning_disabled,
            reading_or_state_unavailable,
            offset_data_1,
            offset_data_2,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RawSensorReading {
    reading: u8,
    all_event_messages_disabled: bool,
    scanning_disabled: bool,
    reading_or_state_unavailable: bool,
    offset_data_1: Option<u8>,
    #[allow(unused)]
    offset_data_2: Option<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThresholdReading {
    pub all_event_messages_disabled: bool,
    pub scanning_disabled: bool,
    pub reading: Option<u8>,
    pub threshold_status: Option<ThresholdStatus>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThresholdStatus {
    pub at_or_above_non_recoverable: bool,
    pub at_or_above_upper_critical: bool,
    pub at_or_above_upper_non_critical: bool,
    pub at_or_below_lower_non_recoverable: bool,
    pub at_or_below_lower_critical: bool,
    pub at_or_below_lower_non_critical: bool,
}

impl From<&RawSensorReading> for ThresholdReading {
    fn from(in_reading: &RawSensorReading) -> Self {
        let threshold_status = if in_reading.reading_or_state_unavailable {
            None
        } else {
            in_reading.offset_data_1.map(|d| ThresholdStatus {
                at_or_above_non_recoverable: (d & 0x20) == 0x20,
                at_or_above_upper_critical: (d & 0x10 == 0x10),
                at_or_above_upper_non_critical: (d & 0x08) == 0x08,
                at_or_below_lower_non_recoverable: (d & 0x04) == 0x04,
                at_or_below_lower_critical: (d & 0x20) == 0x20,
                at_or_below_lower_non_critical: (d & 0x01) == 0x01,
            })
        };

        let reading = if in_reading.reading_or_state_unavailable {
            None
        } else {
            Some(in_reading.reading)
        };

        Self {
            all_event_messages_disabled: in_reading.all_event_messages_disabled,
            scanning_disabled: in_reading.scanning_disabled,
            reading,
            threshold_status,
        }
    }
}
