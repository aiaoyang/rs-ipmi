use crate::{commands::CommandCode, request::ReqPayload, IpmiCommand, Payload};

pub enum BootOpt {
    Pxe,
    HardDisk,
}

///    Parameter valid
///  [7] - 1b = mark parameter invalid/locked
///        0b = mark parameter valid/unlocked
///[6:0] - boot option parameter selector
///        0h = set in progress
///        1h = service partition selector
///        2h = service partition scan
///        3h = BMC boot flag valid bit clearing
///        4h = boot info acknowledge
///        5h = boot flags
///        6h = boot initiator info
///        7h = boot initiator mailbox
///        96:127 = OEM parameters
/// parameter_valid: u8,

///    Parameter Data 1
///  [7] - 1b = boot flags valid
///  [6] - 0b = options apply to next boot only
///        1b = options requested to be persistent for all future boots
///  [5] - BIOS boot type
///        0b = "PC compatible" boot (legacy)
///        1b = Extensible Firmware Interface Boot (EFI)
///[4:0] - reserved
/// data1: u8,

///    Parameter Data 2
///  [7] - 1b = CMOS clear
///  [6] - 1b = lock keyboard
///[5:2] - Boot device selector
///        0000b = no override
///        0001b = force PXE
///        0010b = force boot from default hard-drive
///        0011b = force boot from default hard-drive, request safe mode
///        0100b = force boot from default diagnostic partition
///        0101b = force boot from default CD/DVD
///        0110b = force boot into BIOS setup
///        0111b = force boot from remotely connected floppy/primary removable media
///        1001b = force boot from primary remote media
///        1000b = force boot from remotely connected CD/DVD
///        1010b = reserved
///        1011b = force boot from remotely connected hard drive
///        1100-1110b = reserved
///        1111b = force boot from floppy/primary removable media
///  [1] - 1b = screen blank
///  [0] - 1b = lock out reset button
/// data2: u8,

///    Parameter Data 3
///  [7] - 1b = lock out via power button
///[6:5] - firmware (BIOS) verbotisty
///        00b = system default
///        01b = request quiet display
///        10b = request verbose display
///        11b = reserved
///  [4] - 1b = force progress event traps for [IPMI 2.0]
///  [3] - 1b = user password bypass
///  [2] - 1b = lock out sleep button
///[1:0] - console redirection control
/// data3: u8,

///    Parameter Data 4
///[7:4] - reserved
///  [3] - BIOS shared mode override
///        1b = request BIOS to temporarily set the access mode for the channel specified in parameter #6 to 'Shared'
///        0b = no request to BIOS to change present access mode setting
///[2:0] - BIOS Mux control override
///        000b = BIOS uses recommended setting of the mux at the end of POST
///        001b = requests BIOS to force mux to BMC at conclusion of POST/start of OS boot
///        010b = Requests BIOS to force mux to system at conclusion of POST/start of OS boot
/// data4: u8,

/// Parameter Data 5
/// [7:5] - reserved
/// [4:0] - device instance selector
///         0001b = force PXE
///         0010b = force boot from default hard-drive
///         0011b = force boot from default hard-drive, request safe mode
///         0101b = force boot from default CD/DVD
///         0111b = force boot from remotely connected floppy/primary removable media
///         1000b = force boot from remotely connected CD/DVD
///         1001b = force boot from primary remote media
///         1011b = force boot from remotely connected hard drive
///         1111b = force boot from floppy/primary removable media
///         0000b = no specific device instance requested
/// data5: u8,
type SetSystemBootOptions = [u8; 6];

impl From<&BootOpt> for SetSystemBootOptions {
    fn from(value: &BootOpt) -> Self {
        match value {
            BootOpt::Pxe => [0x05, 0x80, 0x04, 0x00, 0x00, 0x00],
            BootOpt::HardDisk => [0x05, 0x80, 0xc0, 0x00, 0x00, 0x00],
        }
    }
}

impl IpmiCommand for BootOpt {
    type Output = ();

    fn netfn(&self) -> crate::NetFn {
        crate::NetFn::Chassis
    }

    fn command(&self) -> crate::commands::CommandCode {
        CommandCode::Raw(0x08)
    }

    fn payload(&self) -> crate::Payload {
        Payload::IpmiReq(ReqPayload::new(
            self.netfn(),
            self.command(),
            <SetSystemBootOptions>::from(self).to_vec(),
        ))
    }

    fn parse(&self, _data: &[u8]) -> Result<Self::Output, crate::Error> {
        Ok(())
    }
}
