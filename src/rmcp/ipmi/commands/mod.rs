pub use app::channel::*;
pub use app::cipher::*;
pub use app::sol::*;
pub use chassis::*;
pub use sdr::*;
pub use sel::*;

mod app;
mod chassis;
mod sdr;
mod sel;

use crate::NetFn;
use crate::Packet;
use crate::{
    err::Error,
    rmcp::{request::IpmiRequestPayload, PacketPayload},
    IpmiCommand,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandCode {
    Raw(u8),
    Chassis,
    GetChannelAuthCapabilities,
    SetSessionPrivilegeLevel,
    CloseSession,
    GetChannelCipherSuites,
    ActivatePayload,
    DeactivatePayload,
    SendMessage,
}

impl From<u8> for CommandCode {
    fn from(val: u8) -> Self {
        match val {
            0x38 => CommandCode::GetChannelAuthCapabilities,
            0x54 => CommandCode::GetChannelCipherSuites,
            0x3b => CommandCode::SetSessionPrivilegeLevel,
            0x3c => CommandCode::CloseSession,
            0x01 => CommandCode::Chassis,
            0x48 => CommandCode::ActivatePayload,
            0x49 => CommandCode::DeactivatePayload,
            0x34 => CommandCode::SendMessage,
            x => CommandCode::Raw(x),
        }
    }
}

impl From<CommandCode> for u8 {
    fn from(val: CommandCode) -> Self {
        match val {
            CommandCode::Chassis => 0x01,
            CommandCode::GetChannelAuthCapabilities => 0x38,
            CommandCode::GetChannelCipherSuites => 0x54,
            CommandCode::SetSessionPrivilegeLevel => 0x3b,
            CommandCode::CloseSession => 0x3c,
            CommandCode::ActivatePayload => 0x48,
            CommandCode::DeactivatePayload => 0x49,
            CommandCode::SendMessage => 0x34,
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

    const NET_FN: NetFn = NetFn::App;
    const COMMAND: CommandCode = CommandCode::CloseSession;

    fn parse(&self, _data: &[u8]) -> Result<Self::Output, Error> {
        Ok(())
    }

    fn write_to(&self, buf: &mut [u8]) -> usize {
        let data = PacketPayload::IpmiReq(IpmiRequestPayload::new(
            Self::NET_FN,
            Self::COMMAND,
            vec![
                self.0 as u8,
                (self.0 >> 8) as u8,
                (self.0 >> 16) as u8,
                (self.0 >> 24) as u8,
            ],
        ));

        Packet::new_with_data(data).write_to(buf)
    }
}
