use crate::err::IpmiPayloadError;
use crate::rmcp::request::ReqPayload;
use crate::rmcp::{IpmiHeader, IpmiV2Header, Packet, Payload, PayloadType, RmcpHeader};
use crate::{Command, NetFn};

#[derive(Clone)]
pub struct GetChannelCipherSuitesRequest {
    pub channel_number: u8,
    pub payload_type: PayloadType,
    pub list_algo_cipher_suite: bool,
    pub list_index: u8,
}

impl From<GetChannelCipherSuitesRequest> for Vec<u8> {
    fn from(val: GetChannelCipherSuitesRequest) -> Self {
        vec![
            (val.channel_number << 4 >> 4),
            (std::convert::Into::<u8>::into(val.payload_type) << 3 >> 3),
            (((val.list_algo_cipher_suite as u8) << 7) | (val.list_index << 2 >> 2)),
        ]
    }
}

impl GetChannelCipherSuitesRequest {
    pub fn new(
        channel_number: u8,
        payload_type: PayloadType,
        list_algo_cipher_suite: bool,
        list_index: u8,
    ) -> GetChannelCipherSuitesRequest {
        GetChannelCipherSuitesRequest {
            channel_number,
            payload_type,
            list_algo_cipher_suite,
            list_index,
        }
    }

    pub fn create_packet(self) -> Packet {
        let data_bytes: Vec<u8> = self.into();

        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_pre(
                PayloadType::Ipmi,
                ((data_bytes.len() as u8) + 7).into(),
            )),
            Payload::IpmiReq(ReqPayload::new(
                NetFn::App,
                Command::GetChannelCipherSuites,
                Some(data_bytes),
            )),
        )
    }
}

impl Default for GetChannelCipherSuitesRequest {
    fn default() -> Self {
        GetChannelCipherSuitesRequest {
            channel_number: 0xe,
            payload_type: PayloadType::Ipmi,
            list_algo_cipher_suite: true,
            list_index: 0x0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetChannelCipherSuitesResponse {
    /*
    2 bytes Channel Number
    Channel number that the Authentication Algorithms are being returned
    for. If the channel number in the request was set to Eh, this will return
    the channel number for the channel that the request was received on.

    (3:18) bytes Cipher Suite Record data bytes, per Table 22-19, Cipher Suite Record
    Format. Record data is ‘packed’; there are no pad bytes between records. It is
    possible that record data will span across multiple List Index values.
    The BMC returns sixteen (16) bytes at a time per index, starting from index
    00h, until the list data is exhausted, at which point it will 0 bytes or <16 bytes
    of list data.
     */
    pub channel_number: u8,
    pub cypher_suite_record_data_bytes: Vec<u8>,
}

impl TryFrom<&[u8]> for GetChannelCipherSuitesResponse {
    type Error = IpmiPayloadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            Err(IpmiPayloadError::WrongLength)?
        }
        Ok(GetChannelCipherSuitesResponse {
            channel_number: value[0],
            cypher_suite_record_data_bytes: value[1..].to_vec(),
        })
    }
}

impl TryFrom<Vec<u8>> for GetChannelCipherSuitesResponse {
    type Error = IpmiPayloadError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl GetChannelCipherSuitesResponse {
    pub fn is_last(&self) -> bool {
        self.cypher_suite_record_data_bytes.len() < 16
    }
}
