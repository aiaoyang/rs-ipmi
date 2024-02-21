use crate::err::RMCPHeaderError;

#[derive(Clone, Debug)]
pub struct RmcpHeader {
    pub version: u8,         // 0x06 for RMCP Version 1.0
    pub reserved: u8,        // 0x00
    pub sequence_number: u8, // 255 if no RMCP ACK; 0-254 if RMCP ACK desired
    pub rmcp_ack: bool,
    pub message_class: MessageClass,
}

impl TryFrom<&[u8]> for RmcpHeader {
    type Error = RMCPHeaderError;

    fn try_from(value: &[u8]) -> Result<Self, RMCPHeaderError> {
        if value.len() != 4 {
            Err(RMCPHeaderError::WrongLength)?
        }

        // let third_byte_slice = BitSlice::<u8, Msb0>::from_element(&value[3]);

        Ok(RmcpHeader {
            version: value[0],
            reserved: value[1],
            sequence_number: value[2],
            // rmcp_ack: third_byte_slice[0],
            rmcp_ack: (value[3] >> 7) != 0,
            // message_class: third_byte_slice[4..].load::<u8>().try_into()?,
            message_class: (value[3] << 4 >> 4).try_into()?,
        })
    }
}

impl From<RmcpHeader> for Vec<u8> {
    fn from(val: RmcpHeader) -> Self {
        let result = [val.version, val.reserved, val.sequence_number, {
            // let mut bv: BitVec<u8, Msb0> = bitvec![u8, Msb0; 0;8];
            ((val.rmcp_ack as u8) << 6) | (u8::from(val.message_class))
            // bv[0..1].store::<u8>(val.rmcp_ack as u8);
            // bv[4..].store::<u8>(val.message_class.into());

            // bv[..].load::<u8>()
        }];
        let mut vec_result = Vec::new();
        vec_result.extend_from_slice(result.as_slice());
        vec_result
    }
}

// Doesn't apply....yet
// impl RmcpHeader {
//     pub fn new(version: u8, sequence_number: u8, message_class: MessageClass) -> RmcpHeader {
//         RmcpHeader {
//             version,
//             reserved: 0x00,
//             sequence_number,
//             rmcp_ack: false,
//             message_class,
//         }
//     }
// }

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
    type Error = RMCPHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x6 => Ok(MessageClass::Asf),
            0x7 => Ok(MessageClass::Ipmi),
            0x8 => Ok(MessageClass::Oem),
            _ => Err(RMCPHeaderError::UnsupportedMessageClass(value)),
        }
    }
}

impl From<MessageClass> for u8 {
    fn from(val: MessageClass) -> Self {
        match val {
            MessageClass::Asf => 6,
            MessageClass::Ipmi => 7,
            MessageClass::Oem => 8,
        }
    }
}
