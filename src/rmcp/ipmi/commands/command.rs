use core::fmt;

use crate::rmcp::netfn::NetfnLun;

pub trait IpmiCommand: std::fmt::Display + for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>> {
    fn name(&self) -> &str;
    fn code(&self) -> u8;
    fn netfn_rslun(&self) -> NetfnLun;
}

// pub const GET_CHANNEL_AUTH_CAPABILITIES: u8 = 0x38;
#[derive(Clone, Copy, Debug)]
pub enum CommandType {
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

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::Raw(x) => write!(f, "Unknown: 0x{:X}", x),
            CommandType::GetChannelAuthCapabilities => write!(f, "Get Channel Auth Capabilities"),
            CommandType::SetSessionPrivilegeLevel => write!(f, "Set Session Privilege Level"),
            CommandType::GetChannelCipherSuites => write!(f, "Get Channel Cipher Suites"),
        }
    }
}
impl From<u8> for CommandType {
    fn from(val: u8) -> Self {
        match val {
            0x38 => CommandType::GetChannelAuthCapabilities,
            0x54 => CommandType::GetChannelCipherSuites,
            0x3b => CommandType::SetSessionPrivilegeLevel,
            x => CommandType::Raw(x),
        }
    }
}

impl From<CommandType> for u8 {
    fn from(val: CommandType) -> Self {
        match val {
            CommandType::GetChannelAuthCapabilities => 0x38,
            CommandType::GetChannelCipherSuites => 0x54,
            CommandType::SetSessionPrivilegeLevel => 0x3b,
            CommandType::Raw(x) => x,
        }
    }
}
