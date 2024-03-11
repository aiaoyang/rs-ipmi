use crate::err::{ERMCPHeader, Error};

#[derive(Clone, Debug)]
pub struct RmcpHeader {
    pub version: u8,         // 0x06 for RMCP Version 1.0
    pub reserved: u8,        // 0x00
    pub sequence_number: u8, // 255 if no RMCP ACK; 0-254 if RMCP ACK desired
    pub rmcp_ack: bool,
    pub message_class: MessageClass,
}

impl TryFrom<&[u8]> for RmcpHeader {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            Err(ERMCPHeader::WrongLength)?
        }

        Ok(RmcpHeader {
            version: value[0],
            reserved: value[1],
            sequence_number: value[2],
            rmcp_ack: (value[3] >> 7) != 0,
            message_class: (value[3] << 4 >> 4).try_into()?,
        })
    }
}

impl From<&RmcpHeader> for [u8; 4] {
    fn from(val: &RmcpHeader) -> Self {
        [val.version, val.reserved, val.sequence_number, {
            ((val.rmcp_ack as u8) << 6) | (u8::from(&val.message_class))
        }]
    }
}

impl From<&RmcpHeader> for Vec<u8> {
    fn from(val: &RmcpHeader) -> Self {
        [val.version, val.reserved, val.sequence_number, {
            ((val.rmcp_ack as u8) << 6) | (u8::from(&val.message_class))
        }]
        .to_vec()
    }
}

impl Default for RmcpHeader {
    fn default() -> RmcpHeader {
        RmcpHeader {
            version: 0x06,
            reserved: 0,
            sequence_number: 0xff,
            rmcp_ack: false,
            message_class: MessageClass::Ipmi,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageClass {
    Asf,
    Ipmi,
    Oem,
}

impl TryFrom<u8> for MessageClass {
    type Error = ERMCPHeader;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x6 => Ok(MessageClass::Asf),
            0x7 => Ok(MessageClass::Ipmi),
            0x8 => Ok(MessageClass::Oem),
            _ => Err(ERMCPHeader::UnsupportedMessageClass(value)),
        }
    }
}

impl From<&MessageClass> for u8 {
    fn from(val: &MessageClass) -> Self {
        match val {
            MessageClass::Asf => 6,
            MessageClass::Ipmi => 7,
            MessageClass::Oem => 8,
        }
    }
}
