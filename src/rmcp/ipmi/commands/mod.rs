
pub use app::channel::*;
pub use app::cipher::*;
pub use sdr::*;
pub use sel::*;

mod app;
mod sdr;
mod sel;

use crate::{
    err::Error,
    rmcp::{request::ReqPayload, Payload},
    IpmiCommand,
};

#[derive(Clone, Copy, Debug)]
pub enum CommandCode {
    Raw(u8),
    GetChannelAuthCapabilities,
    SetSessionPrivilegeLevel,
    CloseSession,
    GetChannelCipherSuites,
}

impl From<u8> for CommandCode {
    fn from(val: u8) -> Self {
        match val {
            0x38 => CommandCode::GetChannelAuthCapabilities,
            0x54 => CommandCode::GetChannelCipherSuites,
            0x3b => CommandCode::SetSessionPrivilegeLevel,
            0x3c => CommandCode::CloseSession,
            x => CommandCode::Raw(x),
        }
    }
}

impl From<CommandCode> for u8 {
    fn from(val: CommandCode) -> Self {
        match val {
            CommandCode::GetChannelAuthCapabilities => 0x38,
            CommandCode::GetChannelCipherSuites => 0x54,
            CommandCode::SetSessionPrivilegeLevel => 0x3b,
            CommandCode::CloseSession => 0x3c,
            CommandCode::Raw(x) => x,
        }
    }
}


pub struct CloseSessionCMD(u32);
impl CloseSessionCMD {
    pub fn new(session_id: u32) -> Self {
        Self(session_id)
    }
}
impl IpmiCommand for CloseSessionCMD {
    type Output = ();

    fn netfn() -> crate::NetFn {
        crate::NetFn::App
    }

    fn command() -> CommandCode {
        0x3c.into()
    }

    fn payload(&self) -> Payload {
        Payload::IpmiReq(ReqPayload::new(
            Self::netfn(),
            Self::command(),
            vec![
                self.0 as u8,
                (self.0 >> 8) as u8,
                (self.0 >> 16) as u8,
                (self.0 >> 24) as u8,
            ],
        ))
    }

    fn parse(&self, _data: &[u8]) -> Result<Self::Output, Error> {
        Ok(())
    }
}
