use crate::{
    err::{EIpmiPayload, Error},
    rmcp::{commands::CommandCode, Address, CompletionCode, NetfnLun},
};

#[derive(Clone, Debug)]
pub struct RespPayload {
    pub rq_addr: Address,
    pub netfn_rqlun: NetfnLun,
    // checksum 1
    pub rs_addr: Address,
    pub rqseq_rslun: NetfnLun,
    pub command: CommandCode,
    pub completion_code: CompletionCode,
    pub data: Vec<u8>,
    // checksum 2
}

impl TryFrom<&[u8]> for RespPayload {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(EIpmiPayload::WrongLength)?
        }
        let netfn_rqlun = NetfnLun::from(value[1]);
        let rqseq_rslun = NetfnLun::from(value[4]);
        let ret = RespPayload {
            rq_addr: value[0].into(),
            netfn_rqlun,
            rs_addr: value[3].into(),
            rqseq_rslun,
            command: value[5].into(),
            completion_code: value[6].into(),
            data: {
                let len = value.len() - 1;
                if len == 7 {
                    Vec::new()
                } else {
                    value[7..len].into()
                }
            },
        };
        log::debug!("resp: {:?}", ret);
        Ok(ret)
    }
}

impl RespPayload {
    pub fn payload_length(&self) -> usize {
        if self.data.is_empty() {
            8
        } else {
            self.data.len() + 8
        }
    }
}
