use std::rc::Rc;

use crate::{err::IpmiPayloadRequestError, rmcp::Payload, Command};

use super::{Address, CompletionCode, IpmiCommand, NetfnLun};

pub struct IpmiRawCommand {
    pub rs_addr: Address,
    pub netfn_rslun: NetfnLun,
    // checksum 1
    pub rq_addr: Address,
    pub rqseq_rqlun: NetfnLun,
    pub command: Command,
    pub data: Option<Vec<u8>>,
    // checksum 2
}

pub struct IpmiRawResponse {
    pub rq_addr: Address,
    pub netfn_rqlun: NetfnLun,
    // checksum 1
    pub rs_addr: Address,
    pub rqseq_rslun: NetfnLun,
    pub command: Command,
    pub completion_code: CompletionCode,
    pub data: Rc<[u8]>,
}

impl IpmiCommand for IpmiRawCommand {
    type Output = IpmiRawResponse;
    type Error = IpmiPayloadRequestError;

    fn netfn(&self) -> crate::NetFn {
        todo!()
    }

    fn commnad(&self) -> Command {
        todo!()
    }

    fn payload(self) -> Payload {
        todo!()
    }

    fn parse(data: &[u8]) -> Result<Self::Output, Self::Error> {
        if data.len() < 8 {
            Err(IpmiPayloadRequestError::WrongLength)?
        }
        let netfn_rqlun = NetfnLun::from(data[1]);
        let rqseq_rslun = NetfnLun::from(data[4]);
        Ok(IpmiRawResponse {
            rq_addr: data[0].into(),
            netfn_rqlun,
            rs_addr: data[3].into(),
            rqseq_rslun,
            command: data[5].into(),
            completion_code: data[6].into(),
            data: {
                let len = data.len() - 1;
                if len == 7 {
                    Rc::from([])
                } else {
                    Rc::from(&data[7..len])
                }
            },
        })
    }
}
