pub mod event_reading_type_code;
pub mod record;
pub mod sensor_type;

mod units_macro;
pub use units_macro::Unit;

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
