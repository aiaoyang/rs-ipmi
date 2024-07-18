use crate::{
    commands::CommandCode,
    rmcp::{Address, Lun, NetfnLun, SlaveAddress, SoftwareType},
    NetFn,
};

#[derive(Clone, Debug)]
pub struct IpmiRequestPayload<'a> {
    pub rs_addr: Address,
    pub netfn_rslun: NetfnLun,
    // checksum 1
    pub rq_addr: Address,
    pub rqseq_rqlun: NetfnLun,
    pub command: CommandCode,
    pub data: &'a [u8],
    // checksum 2
}

const BMC_SLAVE_ADDRESS: u8 = 0x20;
const REMOTE_SOFTWARE_ID: u8 = 0x81;

impl<'a> Default for IpmiRequestPayload<'a> {
    fn default() -> Self {
        Self {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            netfn_rslun: NetfnLun::new(NetFn::App, Lun::Bmc),
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rqseq_rqlun: NetfnLun::new(0x00, Lun::Bmc),
            command: CommandCode::GetChannelAuthCapabilities,
            data: &[],
        }
    }
}

impl<'a> IpmiRequestPayload<'a> {
    pub fn with_data(&mut self, data: &'a [u8]) {
        self.data = data;
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.data.len() + 7
    }

    pub fn write_to(&self, buf: &mut [u8]) -> usize {
        let data_len = self.data.len();

        buf[0] = BMC_SLAVE_ADDRESS;
        buf[1] = self.netfn_rslun.0;
        buf[2] = checksum(&buf[..2]);
        buf[3] = REMOTE_SOFTWARE_ID;
        buf[4] = self.rqseq_rqlun.0;
        buf[5] = (self.command).into();

        buf[6..6 + data_len].copy_from_slice(self.data);
        buf[6 + data_len] = checksum(&buf[3..6 + data_len]);
        7 + data_len
    }
    pub fn new(net_fn: NetFn, command: CommandCode, data: Option<&'a [u8]>) -> Self {
        Self {
            rs_addr: Address::Slave(SlaveAddress::Bmc),
            netfn_rslun: NetfnLun::new(net_fn, Lun::Bmc),
            rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
            rqseq_rqlun: NetfnLun::new(0x08, Lun::Bmc),
            command,
            data: data.unwrap_or_default(),
        }
    }
}

//
// #[derive(Clone, Debug)]
// pub struct IpmiBridgeRequestPayload {
//     pub rs_addr: Address,
//     pub netfn_rslun: NetfnLun,
//     // checksum 1
//     pub rq_addr: Address,
//     pub rqseq_rqlun: NetfnLun,
//     pub command: CommandCode,
//     pub channel: u8,
//     pub data: IpmiRequestPayload, // checksum 2
// }
//
// impl From<&IpmiBridgeRequestPayload> for IpmiRequest {
//     fn from(value: &IpmiBridgeRequestPayload) -> Self {
//         Self::Bridge(value.clone())
//     }
// }
//
// impl From<&IpmiBridgeRequestPayload> for Vec<u8> {
//     fn from(val: &IpmiBridgeRequestPayload) -> Self {
//         let mut ret = <Vec<u8>>::new();
//         ret.extend(&[
//             BMC_SLAVE_ADDRESS,
//             val.netfn_rslun.0,
//             checksum(&[BMC_SLAVE_ADDRESS, val.netfn_rslun.0]),
//             REMOTE_SOFTWARE_ID,
//             val.rqseq_rqlun.0,
//             (val.command).into(),
//             val.channel,
//         ]);
//         ret.extend(&<Vec<u8>>::from(&val.data));
//         ret.push(checksum(&ret[3..]));
//         println!("bridge data: {ret:x?}");
//         ret
//     }
// }
//
// impl IpmiBridgeRequestPayload {
//     pub fn new(bridge_addr: u8, channel: u8, mut data: IpmiRequestPayload) -> Self {
//         data.rs_addr = bridge_addr.into();
//         Self {
//             rs_addr: Address::Slave(SlaveAddress::Bmc),
//             netfn_rslun: NetfnLun::new(NetFn::App, Lun::Bmc),
//             rq_addr: Address::Software(SoftwareType::RemoteConsoleSoftware(1)),
//             rqseq_rqlun: NetfnLun::new(0x08, Lun::Bmc),
//             command: CommandCode::SendMessage,
//             channel: 0x40 | channel,
//             data,
//         }
//     }
// }

// two's complement sum
pub fn checksum(input: &[u8]) -> u8 {
    let mut res = 0_i32;
    for val in input {
        res += *val as i32
    }
    (-res) as u8
}
