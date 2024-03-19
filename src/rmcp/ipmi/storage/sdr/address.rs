use std::num::NonZeroU8;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Address(pub u8);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChannelNumber(NonZeroU8);

impl ChannelNumber {
    /// Create a new `ChannelNumber`.
    ///
    /// This function returns `None` if `value > 0xB`
    pub fn new(value: NonZeroU8) -> Option<Self> {
        if value.get() <= 0xB {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value of this `ChannelNumber`.
    ///
    /// It is guaranteed that values returned by
    /// this function are less than or equal to `0xB`
    pub fn value(&self) -> NonZeroU8 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Channel {
    Primary,
    Numbered(ChannelNumber),
    System,
    Current,
}

impl Channel {
    /// Create a new `Channel`.
    ///
    /// This function returns `None` if `value == 0xC` || value == 0xD || value > 0xF`
    pub fn new(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Primary),
            0xE => Some(Self::Current),
            0xF => Some(Self::System),
            v => Some(Self::Numbered(ChannelNumber::new(NonZeroU8::new(v)?)?)),
        }
    }

    /// The number of this channel.
    ///
    /// This value is guaranteed to be less than or
    /// equal to 0xF, and will be 0xE if `self` is
    /// [`Channel::Current`].
    pub fn value(&self) -> u8 {
        match self {
            Channel::Primary => 0x0,
            Channel::Numbered(v) => v.value().get(),
            Channel::Current => 0xE,
            Channel::System => 0xF,
        }
    }
}
