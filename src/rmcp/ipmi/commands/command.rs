use crate::{
    err::Error,
    rmcp::{request::ReqPayload, Payload},
    IpmiCommand,
};

#[derive(Clone, Copy, Debug)]
pub enum CommandCode {
    Raw(u8),
    // *APP Commands*
    // Reserved,
    // GetDeviceId,
    // ColdReset,
    // WarmReset,
    // GetSelfTestResults,
    // ManufacturingTestOn,
    // SetACPIPowerState,
    // GetACPIPowerState,
    // GetDeviceGUID,
    // GetNetFnSupport,
    // GetCommandSupport,
    // GetCommandSubfunctionSupport,
    // GetConfigurableCommandSubfunctions,
    // Unassigned,
    // SetCommandEnables,
    // GetCommandEnables,
    // SetCommandSubfunctionEnables,
    // GetCommandSubfunctionEnables,
    // GetOEMNetFnIANASupport,
    // ResetWatchdogTimer,
    // SetWatchdogTimer,
    // GetWatchdogTimer,
    // SetBMCGlobalEnables,
    // GetBMCGlobalEnables,
    // ClearMessageFlags,
    // GetMessageFlags,
    // EnableMessageChannelReceive,
    // GetMessage,
    // SendMessage,
    // ReadEventMessageBuffer,
    // GetBTInterfaceCapabilities,
    // GetSystemGUID,
    // SetSystemInfoParameters,
    // GetSystemInfoParameters,
    GetChannelAuthCapabilities,
    // GetSessionChallenge,
    // ActivateSession,
    SetSessionPrivilegeLevel,
    CloseSession,
    // GetAuthCode,
    // SetChannelAccess,
    // GetChannelAccess,
    // GetChannelInfoCommand,
    // SetUserAccessCommand,
    // GetUserAccessCommand,
    // SetUserName,
    // GetUserNameCommand,
    // SetUserPasswordCommand,
    // ActivatePayload,
    // DeactivatePayload,
    // GetPayloadActivationStatus,
    // GetPayloadInstanceInfo,
    // SetUserPayloadAccess,
    // GetUserPayloadAccess,
    // GetChannelPayloadSupport,
    // GetChannelPayloadVersion,
    // GetChannelOEMPayloadInfo,
    // MasterWriteRead,
    GetChannelCipherSuites,
    // SuspendResumePayloadEncryption,
    // SetChannelSecurityKeys,
    // GetSystemInterfaceCapabilities,
    // FirmwareFirewallConfiguration,
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
    type Error = Error;

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::App
    }

    fn commnad(&self) -> CommandCode {
        0x3c.into()
    }

    fn payload(self) -> Payload {
        Payload::IpmiReq(ReqPayload::new(
            self.netfn(),
            self.commnad(),
            vec![
                self.0 as u8,
                (self.0 >> 8) as u8,
                (self.0 >> 16) as u8,
                (self.0 >> 24) as u8,
            ],
        ))
    }

    fn parse(_data: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
