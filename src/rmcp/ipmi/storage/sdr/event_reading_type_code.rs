#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventReadingTypeCodes {
    Unspecified,
    Threshold,
    DiscreteGeneric(EventReading),
    SensorSpecific,
    Oem(u8),
    Reserved(u8),
}

impl From<u8> for EventReadingTypeCodes {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Unspecified,
            0x01 => Self::Threshold,
            0x02..=0x0C => Self::DiscreteGeneric(EventReading::try_from(value).unwrap()),
            0x6F => Self::SensorSpecific,
            0x70..=0x7F => Self::Oem(value),
            v => Self::Reserved(v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventReading {
    UsageState = 0x02,
    StateAssertion = 0x03,
    PredictiveFailure = 0x04,
    LimitExcess = 0x05,
    PerformanceMetric = 0x06,
    SeverityEvents = 0x07,
    DevicePresence = 0x08,
    DeviceEnabledStatus = 0x09,
    PowerState = 0x0A,
    RedundancyState = 0x0B,
    AcpiDevicePowerState = 0x0C,
}

impl TryFrom<u8> for EventReading {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        assert!((0x02..=0x0C).contains(&value));
        match value {
            0x02 => Ok(Self::UsageState),
            0x03 => Ok(Self::StateAssertion),
            0x04 => Ok(Self::PredictiveFailure),
            0x05 => Ok(Self::LimitExcess),
            0x06 => Ok(Self::PerformanceMetric),
            0x07 => Ok(Self::SeverityEvents),
            0x08 => Ok(Self::DevicePresence),
            0x09 => Ok(Self::DeviceEnabledStatus),
            0x0A => Ok(Self::PowerState),
            0x0B => Ok(Self::RedundancyState),
            0x0C => Ok(Self::AcpiDevicePowerState),
            _ => unreachable!(),
        }
    }
}
