#[derive(Debug, Clone, Copy)]
pub struct SensorCapabilities {
    pub ignore: bool,
    pub auto_rearm: bool,
    // TODO: make a type
    pub event_message_control: u8,
    pub hysteresis: HysteresisCapability,
    pub threshold_access: ThresholdAccessCapability,
    pub assertion_threshold_events: ThresholdAssertEventMask,
    pub deassertion_threshold_events: ThresholdAssertEventMask,
}

impl SensorCapabilities {
    pub fn new(
        caps: u8,
        assert_lower_thrsd: u16,
        deassert_upper_thrshd: u16,
        discrete_rd_thrsd_set_thrshd_read: u16,
    ) -> Self {
        let ignore = (caps & 0x80) == 0x80;
        let auto_rearm = (caps & 0x40) == 0x40;
        let hysteresis = match caps & 0x30 >> 4 {
            0b00 => HysteresisCapability::NoneOrUnspecified,
            0b01 => HysteresisCapability::Readable,
            0b10 => HysteresisCapability::ReadableAndSettable,
            0b11 => HysteresisCapability::FixedAndUnreadable,
            _ => unreachable!(),
        };
        let event_message_control = caps & 0b11;

        let assertion_event_mask = ThresholdAssertEventMask::from_bits_truncate(assert_lower_thrsd);
        let deassertion_event_mask =
            ThresholdAssertEventMask::from_bits_truncate(deassert_upper_thrshd);

        let threshold_read_value_mask = Thresholds {
            lower_non_recoverable: ((assert_lower_thrsd >> 14) & 0x1) == 1,
            lower_critical: ((assert_lower_thrsd >> 13) & 0x1) == 1,
            lower_non_critical: ((assert_lower_thrsd >> 12) & 0x1) == 1,
            upper_non_recoverable: ((deassert_upper_thrshd >> 14) & 0x1) == 1,
            upper_critical: ((deassert_upper_thrshd >> 14) & 0x1) == 1,
            upper_non_critical: ((deassert_upper_thrshd >> 14) & 0x1) == 1,
        };

        let threshold_set_mask = Thresholds {
            upper_non_recoverable: ((discrete_rd_thrsd_set_thrshd_read >> 13) & 0x1) == 1,
            upper_critical: ((discrete_rd_thrsd_set_thrshd_read >> 12) & 0x1) == 1,
            upper_non_critical: ((discrete_rd_thrsd_set_thrshd_read >> 11) & 0x1) == 1,
            lower_non_recoverable: ((discrete_rd_thrsd_set_thrshd_read >> 10) & 0x1) == 1,
            lower_critical: ((discrete_rd_thrsd_set_thrshd_read >> 9) & 0x1) == 1,
            lower_non_critical: ((discrete_rd_thrsd_set_thrshd_read >> 8) & 0x1) == 1,
        };

        let threshold_read_mask = Thresholds {
            upper_non_recoverable: ((discrete_rd_thrsd_set_thrshd_read >> 5) & 0x1) == 1,
            upper_critical: ((discrete_rd_thrsd_set_thrshd_read >> 4) & 0x1) == 1,
            upper_non_critical: ((discrete_rd_thrsd_set_thrshd_read >> 3) & 0x1) == 1,
            lower_non_recoverable: ((discrete_rd_thrsd_set_thrshd_read >> 2) & 0x1) == 1,
            lower_critical: ((discrete_rd_thrsd_set_thrshd_read >> 1) & 0x1) == 1,
            lower_non_critical: (discrete_rd_thrsd_set_thrshd_read & 0x1) == 1,
        };

        let threshold_access_support = match (caps & 0xC) >> 2 {
            0b00 => ThresholdAccessCapability::None,
            0b01 => ThresholdAccessCapability::Readable {
                readable: threshold_read_mask,
                values: threshold_read_value_mask,
            },
            0b10 => ThresholdAccessCapability::ReadableAndSettable {
                readable: threshold_read_mask,
                values: threshold_read_value_mask,
                settable: threshold_set_mask,
            },
            0b11 => ThresholdAccessCapability::FixedAndUnreadable {
                supported: threshold_read_mask,
            },
            _ => unreachable!(),
        };

        Self {
            ignore,
            auto_rearm,
            hysteresis,
            event_message_control,
            threshold_access: threshold_access_support,
            assertion_threshold_events: assertion_event_mask,
            deassertion_threshold_events: deassertion_event_mask,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HysteresisCapability {
    NoneOrUnspecified,
    Readable,
    ReadableAndSettable,
    FixedAndUnreadable,
}

#[derive(Debug, Clone, Copy)]
pub enum ThresholdAccessCapability {
    None,
    Readable {
        readable: Thresholds,
        values: Thresholds,
    },
    ReadableAndSettable {
        readable: Thresholds,
        values: Thresholds,
        settable: Thresholds,
    },
    FixedAndUnreadable {
        supported: Thresholds,
    },
}

impl ThresholdAccessCapability {
    pub fn readable(&self, kind: ThresholdKind) -> bool {
        match self {
            ThresholdAccessCapability::Readable { readable, .. } => readable.for_kind(kind),
            ThresholdAccessCapability::ReadableAndSettable { readable, .. } => {
                readable.for_kind(kind)
            }
            _ => false,
        }
    }

    pub fn settable(&self, kind: ThresholdKind) -> bool {
        match self {
            ThresholdAccessCapability::ReadableAndSettable { settable, .. } => {
                settable.for_kind(kind)
            }
            _ => false,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ThresholdAssertEventMask: u16 {
        const UPPER_NON_RECOVERABLE_GOING_HIGH = 1 << 11;
        const UPPER_NON_RECOVERABLE_GOING_LOW = 1 << 10;
        const UPPER_CRITICAL_GOING_HIGH = 1 << 9;
        const UPPER_CRITICAL_GOING_LOW = 1 << 8;
        const UPPER_NON_CRITICAL_GOING_HIGH = 1 << 7;
        const UPPER_NON_CRITICAL_GOING_LOW = 1 << 6;
        const LOWER_NON_RECOVERABLE_GOING_HIGH = 1 << 5;
        const LOWER_NON_RECOVERABLE_GOING_LOW = 1 << 4;
        const LOWER_CRITICAL_GOING_HIGH = 1 << 3;
        const LOWER_CRITICAL_GOING_LOW = 1 << 2;
        const LOWER_NON_CRITICAL_GOING_HIGH = 1 << 1;
        const LOWER_NON_CRITICAL_GOING_LOW = 1 << 0;

    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    GoingHigh,
    GoingLow,
}

impl ThresholdAssertEventMask {
    pub fn for_kind(&self, kind: ThresholdKind) -> &[EventKind] {
        static BOTH: [EventKind; 2] = [EventKind::GoingHigh, EventKind::GoingLow];
        static HIGH: [EventKind; 1] = [EventKind::GoingHigh];
        static LOW: [EventKind; 1] = [EventKind::GoingLow];
        static NONE: [EventKind; 0] = [];

        let (low, high) = match kind {
            ThresholdKind::LowerNonCritical => (
                self.contains(Self::LOWER_NON_CRITICAL_GOING_LOW),
                self.contains(Self::LOWER_NON_CRITICAL_GOING_HIGH),
            ),
            ThresholdKind::LowerCritical => (
                self.contains(Self::LOWER_CRITICAL_GOING_LOW),
                self.contains(Self::LOWER_CRITICAL_GOING_HIGH),
            ),
            ThresholdKind::LowerNonRecoverable => (
                self.contains(Self::LOWER_NON_RECOVERABLE_GOING_LOW),
                self.contains(Self::LOWER_NON_RECOVERABLE_GOING_HIGH),
            ),
            ThresholdKind::UpperNonCritical => (
                self.contains(Self::UPPER_NON_CRITICAL_GOING_LOW),
                self.contains(Self::UPPER_NON_CRITICAL_GOING_HIGH),
            ),
            ThresholdKind::UpperCritical => (
                self.contains(Self::UPPER_CRITICAL_GOING_LOW),
                self.contains(Self::UPPER_CRITICAL_GOING_HIGH),
            ),
            ThresholdKind::UpperNonRecoverable => (
                self.contains(Self::UPPER_NON_RECOVERABLE_GOING_LOW),
                self.contains(Self::UPPER_NON_RECOVERABLE_GOING_HIGH),
            ),
        };

        if low && high {
            &BOTH
        } else if low {
            &LOW
        } else if high {
            &HIGH
        } else {
            &NONE
        }
    }
}

#[derive(Debug, Clone, Copy)]

pub struct Thresholds {
    pub lower_non_recoverable: bool,
    pub lower_critical: bool,
    pub lower_non_critical: bool,
    pub upper_non_recoverable: bool,
    pub upper_critical: bool,
    pub upper_non_critical: bool,
}

impl Thresholds {
    pub fn for_kind(&self, kind: ThresholdKind) -> bool {
        match kind {
            ThresholdKind::LowerNonCritical => self.lower_non_critical,
            ThresholdKind::LowerCritical => self.lower_critical,
            ThresholdKind::LowerNonRecoverable => self.lower_non_recoverable,
            ThresholdKind::UpperNonCritical => self.upper_non_critical,
            ThresholdKind::UpperCritical => self.upper_critical,
            ThresholdKind::UpperNonRecoverable => self.upper_non_recoverable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThresholdKind {
    LowerNonCritical,
    LowerCritical,
    LowerNonRecoverable,
    UpperNonCritical,
    UpperCritical,
    UpperNonRecoverable,
}

impl ThresholdKind {
    pub fn variants() -> impl Iterator<Item = Self> {
        [
            Self::LowerNonCritical,
            Self::LowerCritical,
            Self::LowerNonRecoverable,
            Self::UpperNonCritical,
            Self::UpperCritical,
            Self::UpperNonRecoverable,
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Threshold {
    pub kind: ThresholdKind,
    pub readable: bool,
    pub settable: bool,
    pub event_assert_going_high: bool,
    pub event_assert_going_low: bool,
    pub event_deassert_going_high: bool,
    pub event_deassert_going_low: bool,
}
