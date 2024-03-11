use crate::{
    commands::CommandCode,
    rmcp::{
        Address, IpmiHeader, IpmiV2Header, Lun, NetfnLun, Packet, Payload, RmcpHeader,
        SlaveAddress, SoftwareType,
    },
    NetFn,
};

pub struct IpmiRawRequest {
    pub netfn: NetFn,
    pub command: CommandCode,
    pub data: Vec<u8>,
}

impl IpmiRawRequest {
    // const PAYLOAD_LENGTH: u16 = 32;
    pub fn new(
        netfn: impl Into<NetFn>,
        command: impl Into<CommandCode>,
        data: Vec<u8>,
    ) -> IpmiRawRequest {
        IpmiRawRequest {
            netfn: netfn.into(),
            command: command.into(),
            data,
        }
    }

    pub fn create_packet(self) -> Packet {
        let netfn = self.netfn;
        let cmd = self.command;
        let data = self.data;
        Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_est(7 + data.len() as u16)),
            Payload::IpmiReq(ReqPayload::new(netfn, cmd, data)),
        )
    }
}

#[derive(Clone, Debug)]
pub struct ReqPayload {
    pub rs_addr: Address,
    pub netfn_rslun: NetfnLun,
    // checksum 1
    pub rq_addr: Address,
    pub rqseq_rqlun: NetfnLun,
    pub command: CommandCode,
    pub data: Vec<u8>,
    // checksum 2
}

const BMC_SLAVE_ADDRESS: u8 = 0x20;
const REMOTE_SOFTWARE_ID: u8 = 0x81;

impl From<ReqPayload> for Vec<u8> {
    fn from(val: ReqPayload) -> Self {
        let mut result: Vec<u8> = vec![];

        result.extend(&[
            BMC_SLAVE_ADDRESS,
            val.netfn_rslun.0,
            checksum(&[BMC_SLAVE_ADDRESS, val.netfn_rslun.0]),
            REMOTE_SOFTWARE_ID,
            val.rqseq_rqlun.0,
            (val.command).into(),
        ]);

        result.extend(&val.data);
        result.push(checksum(&result[3..]));
        result
    }
}

impl ReqPayload {
    pub fn new(net_fn: NetFn, command: CommandCode, data: Vec<u8>) -> ReqPayload {
        ReqPayload {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            netfn_rslun: NetfnLun::new(net_fn, Lun::Bmc),
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rqseq_rqlun: NetfnLun::new(0x08, Lun::Bmc),
            command,
            data,
        }
    }
}

impl Default for ReqPayload {
    fn default() -> Self {
        Self {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            netfn_rslun: NetfnLun::new(NetFn::App, Lun::Bmc),
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rqseq_rqlun: NetfnLun::new(0x00, Lun::Bmc),
            command: CommandCode::GetChannelAuthCapabilities,
            data: Vec::new(),
        }
    }
}

// two's complement sum
pub fn checksum(input: &[u8]) -> u8 {
    let mut res = 0_i32;
    for val in input {
        res += *val as i32
    }
    (-res) as u8
}
