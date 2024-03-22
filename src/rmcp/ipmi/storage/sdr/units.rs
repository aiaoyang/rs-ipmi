use super::units_macro::Unit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataFormat {
    Unsigned,
    OnesComplement,
    TwosComplement,
}

#[derive(Debug)]
pub struct Value {
    units: SensorUnits,
    value: f32,
}

impl Value {
    pub fn new(units: SensorUnits, value: f32) -> Self {
        Self { units, value }
    }

    pub fn display(&self, short: bool) -> String {
        if self.units.is_percentage {
            format!("{:.2} %", self.value)
        } else {
            // TODO: use Modifier unit and rate units
            // somehow here
            self.units.base_unit.display(short, self.value)
        }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RateUnit {
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierUnit {
    BasUnitDivByModifier(Unit),
    BaseUnitMulByModifier(Unit),
}

#[derive(Debug, Clone, Copy)]
pub struct SensorUnits {
    pub rate: Option<RateUnit>,
    pub modifier: Option<ModifierUnit>,
    pub is_percentage: bool,
    pub base_unit: Unit,
}

impl SensorUnits {
    pub fn from(sensor_units_1: u8, base_unit: u8, modifier_unit: u8) -> Self {
        let rate = match (sensor_units_1 >> 3) & 0b111 {
            0b000 => None,
            0b001 => Some(RateUnit::Microsecond),
            0b010 => Some(RateUnit::Millisecond),
            0b011 => Some(RateUnit::Second),
            0b100 => Some(RateUnit::Minute),
            0b101 => Some(RateUnit::Hour),
            0b110 => Some(RateUnit::Day),
            0b111 => None,
            _ => unreachable!(),
        };

        let base_unit = Unit::from(base_unit);

        let modifier_unit = Unit::from(modifier_unit);

        let modifier = match (sensor_units_1 >> 1) & 0b11 {
            0b00 => None,
            0b01 => Some(ModifierUnit::BasUnitDivByModifier(modifier_unit)),
            0b10 => Some(ModifierUnit::BaseUnitMulByModifier(modifier_unit)),
            0b11 => None,
            _ => unreachable!(),
        };

        let is_percentage = (sensor_units_1 & 0x1) == 0x1;

        Self {
            rate,
            modifier,
            base_unit,
            is_percentage,
        }
    }
}
