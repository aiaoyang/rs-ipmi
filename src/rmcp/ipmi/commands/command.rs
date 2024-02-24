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
