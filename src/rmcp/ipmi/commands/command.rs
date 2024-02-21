use core::fmt;

use crate::rmcp::netfn::NetfnLun;

pub trait IpmiCommand: std::fmt::Display + for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>> {
    fn name(&self) -> &str;
    fn code(&self) -> u8;
    fn netfn_rslun(&self) -> NetfnLun;
}

// pub const GET_CHANNEL_AUTH_CAPABILITIES: u8 = 0x38;
#[derive(Clone, Copy, Debug)]
pub enum Command {
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
    // CloseSession,
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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Raw(x) => write!(f, "Unknown: 0x{:X}", x),
            Command::GetChannelAuthCapabilities => write!(f, "Get Channel Auth Capabilities"),
            Command::SetSessionPrivilegeLevel => write!(f, "Set Session Privilege Level"),
            Command::GetChannelCipherSuites => write!(f, "Get Channel Cipher Suites"),
        }
    }
}
impl From<u8> for Command {
    fn from(val: u8) -> Self {
        match val {
            0x38 => Command::GetChannelAuthCapabilities,
            0x54 => Command::GetChannelCipherSuites,
            0x3b => Command::SetSessionPrivilegeLevel,
            x => Command::Raw(x),
        }
    }
}

impl From<Command> for u8 {
    fn from(val: Command) -> Self {
        match val {
            Command::GetChannelAuthCapabilities => 0x38,
            Command::GetChannelCipherSuites => 0x54,
            Command::SetSessionPrivilegeLevel => 0x3b,
            Command::Raw(x) => x,
        }
    }
}