use crate::{
    commands::Command,
    helpers::utils::checksum,
    parser::{AuthType, IpmiHeader, IpmiV2Header, Packet, Payload, PayloadType},
};

use super::{
    ipmi_payload::{SlaveAddress, SoftwareType},
    lun::Lun,
    netfn::{CommandType, NetFn, NetfnLun},
    response::Address,
};

pub struct IpmiRawRequest {
    pub netfn: NetFn,
    pub command_code: Command,
    pub data: Option<Vec<u8>>,
}

impl IpmiRawRequest {
    const PAYLOAD_LENGTH: u16 = 32;
    pub fn new(netfn: NetFn, command_code: Command, data: Option<Vec<u8>>) -> IpmiRawRequest {
        IpmiRawRequest {
            netfn,
            command_code,
            data,
        }
    }

    pub fn create_packet(&self, rmcp_plus_session_id: u32, session_seq_number: u32) -> Packet {
        let netfn = self.netfn;
        let cmd = self.command_code;
        let data = self.data.clone();
        Packet::new(
            IpmiHeader::V2_0(IpmiV2Header {
                auth_type: AuthType::RmcpPlus,
                payload_enc: true,
                payload_auth: true,
                payload_type: PayloadType::Ipmi,
                oem_iana: None,
                oem_payload_id: None,
                rmcp_plus_session_id,
                session_seq_number,
                payload_length: Self::PAYLOAD_LENGTH,
            }),
            Payload::IpmiReq(ReqPayload::new(netfn, cmd, data)),
        )
    }
}

#[derive(Clone, Debug)]
pub struct ReqPayload {
    pub rs_addr: Address,
    pub net_fn: NetFn,
    pub rs_lun: Lun,
    // checksum 1
    pub rq_addr: Address,
    pub rq_sequence: u8,
    pub rq_lun: Lun,
    pub command: Command,
    pub data: Option<Vec<u8>>,
    // checksum 2
}

const BMC_SLAVE_ADDRESS: u8 = 0x20;
const REMOTE_SOFTWARE_ID: u8 = 0x81;

impl From<ReqPayload> for Vec<u8> {
    fn from(val: ReqPayload) -> Self {
        let mut result: Vec<u8> = vec![];
        let rs_addr = BMC_SLAVE_ADDRESS;
        let netfn_rslun = NetfnLun::new(val.net_fn.to_u8(CommandType::Request), val.rs_lun.into());

        let checksum1 = checksum(&[rs_addr, netfn_rslun.0]);
        let rq_addr = REMOTE_SOFTWARE_ID;
        let rq_seq_rq_lun = NetfnLun::new(val.rq_sequence, val.rs_lun.into());
        let command_code: u8 = (val.command).into();
        result.push(rs_addr);
        result.push(netfn_rslun.0);
        result.push(checksum1);
        result.push(rq_addr);
        result.push(rq_seq_rq_lun.0);
        result.push(command_code);
        if let Some(data) = &val.data {
            result.extend(data);
        }
        result.push(checksum(&result[3..]));
        result
    }
}

impl ReqPayload {
    pub fn new(net_fn: NetFn, command: Command, data: Option<Vec<u8>>) -> ReqPayload {
        ReqPayload {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            net_fn,
            rs_lun: Lun::Bmc,
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rq_sequence: 0x8,
            rq_lun: Lun::Bmc,
            command,
            data,
        }
    }
}

impl Default for ReqPayload {
    fn default() -> Self {
        Self {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            net_fn: NetFn::App,
            rs_lun: Lun::Bmc,
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rq_sequence: 0x00,
            rq_lun: Lun::Bmc,
            command: Command::GetChannelAuthCapabilities,
            data: None,
        }
    }
}
