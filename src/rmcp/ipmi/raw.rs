use crate::{
    err::{IpmiPayloadRequestError, ParseError},
    Command,
};

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

pub struct IpmiRawResponse<'a> {
    pub rq_addr: Address,
    pub netfn_rqlun: NetfnLun,
    // checksum 1
    pub rs_addr: Address,
    pub rqseq_rslun: NetfnLun,
    pub command: Command,
    pub completion_code: CompletionCode,
    pub data: &'a [u8],
}

impl IpmiCommand for IpmiRawCommand {
    type Output<'a> = IpmiRawResponse<'a>;
    type Error = IpmiPayloadRequestError;

    fn netfn(&self) -> crate::NetFn {
        todo!()
    }

    fn commnad(&self) -> Command {
        todo!()
    }

    fn to_vec(self) -> Vec<u8> {
        todo!()
    }

    fn parse(data: &[u8]) -> Result<Self::Output<'_>, Self::Error> {
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
                    &[]
                } else {
                    &data[7..len]
                }
            },
        })
    }

    fn check_cc_success(cc: CompletionCode) -> Result<(), ParseError> {
        if cc.is_success() {
            Ok(())
        } else {
            Err(ParseError::UnknownCompletionCode)
        }
    }
}
