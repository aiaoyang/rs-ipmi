use std::collections::HashMap;
lazy_static::lazy_static! {
 pub static ref SENSOR_GENERIC_EVENT_DESC: HashMap<u32,&'static str>  = {
  HashMap::from([
    // Event Type, Offset
    ((0x01 << 8), "Lower Non-critical - going low"),
    ((0x01 << 8) | 0x01,"Lower Non-critical - going high"),
    ((0x01 << 8) | 0x02,"Lower Critical - going low"),
    ((0x01 << 8) | 0x03,"Lower Critical - going high"),
    ((0x01 << 8) | 0x04,"Lower Non-recoverable - going low"),
    ((0x01 << 8) | 0x05,"Lower Non-recoverable - going high"),
    ((0x01 << 8) | 0x06,"Upper Non-critical - going low"),
    ((0x01 << 8) | 0x07,"Upper Non-critical - going high"),
    ((0x01 << 8) | 0x08,"Upper Critical - going low"),
    ((0x01 << 8) | 0x09,"Upper Critical - going high"),
    ((0x01 << 8) | 0x0a,"Upper Non-recoverable - going low"),
    ((0x01 << 8) | 0x0b,"Upper Non-recoverable - going high"),

    ((0x02 << 8),"Transition to Idle"),
    ((0x02 << 8) | 0x01,"Transition to Active"),
    ((0x02 << 8) | 0x02,"Transition to Busy"),

    ((0x03 << 8),"State Deasserted"),
    ((0x03 << 8) | 0x01,"State Asserted"),

    ((0x04 << 8),"Predictive Failure deasserted"),
    ((0x04 << 8) | 0x01,"Predictive Failure asserted"),

    ((0x05 << 8),"Limit Not Exceeded"),
    ((0x05 << 8) | 0x01,"Limit Exceeded"),

    ((0x06 << 8),"Performance Met"),
    ((0x06 << 8) | 0x01,"Performance Lags"),

    ((0x07 << 8),"transition to OK"),
    ((0x07 << 8) | 0x01,"transition to Non-Critical from OK"),
    ((0x07 << 8) | 0x02,"transition to Critical from less severe"),
    ((0x07 << 8) | 0x03,"transition to Non-recoverable from less severe"),
    ((0x07 << 8) | 0x04,"transition to Non-Critical from more severe"),
    ((0x07 << 8) | 0x05,"transition to Critical from Non-recoverable"),
    ((0x07 << 8) | 0x06,"transition to Non-recoverable"),
    ((0x07 << 8) | 0x07,"Monitor"),
    ((0x07 << 8) | 0x08,"Informational"),

    ((0x08 << 8),"Device Removed/Device Absent"),
    ((0x08 << 8) | 0x01,"Device Inserted/Device Present"),

    ((0x09 << 8),"Device Disabled"),
    ((0x09 << 8) | 0x01,"Device Enabled"),

    ((0x0a << 8),"transition to Running"),
    ((0x0a << 8) | 0x01,"transition to In Test"),
    ((0x0a << 8) | 0x02,"transition to Power Off"),
    ((0x0a << 8) | 0x03,"transition to On Line"),
    ((0x0a << 8) | 0x04,"transition to Off Line"),
    ((0x0a << 8) | 0x05,"transition to Off Duty"),
    ((0x0a << 8) | 0x06,"transition to Degraded"),
    ((0x0a << 8) | 0x07,"transition to Power Save"),
    ((0x0a << 8) | 0x08,"install Error"),

    ((0x0b << 8),"Fully Redundant (formerly \"Redundancy Regained\")"),
    ((0x0b << 8) | 0x01,"Redundancy Lost"),
    ((0x0b << 8) | 0x02,"Redundancy Degraded"),
    ((0x0b << 8) | 0x03,"Non-redundant:Sufficient Resources from Redundant"),
    ((0x0b << 8) | 0x04,"Non-redundant:Sufficient Resources from Insufficient Resources"),
    ((0x0b << 8) | 0x05,"Non-redundant:Insufficient Resources"),
    ((0x0b << 8) | 0x06,"Redundancy Degraded from Fully Redundant"),
    ((0x0b << 8) | 0x07,"Redundancy Degraded from Non-redundant"),

    ((0x0c << 8),"D0 Power State"),
    ((0x0c << 8) | 0x01,"D1 Power State"),
    ((0x0c << 8) | 0x02,"D2 Power State"),
    ((0x0c << 8) | 0x03,"D3 Power State"),
  ])
 };

 pub static ref SENSOR_SPECIFIC_EVENT_DESC: HashMap<u32, &'static str> = {
    HashMap::from([
    // Sensor Type, Offset, Event Data2, Event Data3
    ((0x05 << 24) | (0xff << 8) | 0xff,"General Chassis Intrusion"),
    ((0x05 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Drive Bay intrusion"),
    ((0x05 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"I/O Card area intrusion"),
    ((0x05 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Processor area intrusion"),
    ((0x05 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"LAN Leash Lost (system is unplugged from LAN)"),
    ((0x05 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Unauthorized dock"),
    ((0x05 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"FAN area intrusion (supports detection of hot plug fan tampering)"),

    ((0x06 << 24) | (0xff << 8) | 0xff,"Secure Mode (Front Panel Lockout) Violation attempt"),
    ((0x06 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Pre-boot Password Violation - user password"),
    ((0x06 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Pre-boot Password Violation attempt - setup password"),
    ((0x06 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Pre-boot Password Violation - network boot password"),
    ((0x06 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Other pre-boot Password Violation"),
    ((0x06 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Out-of-band Access Password Violation"),

    ((0x07 << 24) | (0xff << 8) | 0xff,"IERR"),
    ((0x07 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Thermal Trip"),
    ((0x07 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"FRB1/BIST failure"),
    ((0x07 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"FRB2/Hang in POST failure"),
    ((0x07 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"FRB3/Processor Startup/Initialization failure"),
    ((0x07 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Configuration Error"),
    ((0x07 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"SM BIOS `Uncorrectable CPU-complex Error'"),
    ((0x07 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Processor Presence detected"),
    ((0x07 << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Processor disabled"),
    ((0x07 << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"Terminator Presence Detected"),
    ((0x07 << 24) | (0x0a << 16) | (0xff << 8) | 0xff,"Processor Automatically Throttled"),
    ((0x07 << 24) | (0x0b << 16) | (0xff << 8) | 0xff,"Machine Check Exception (Uncorrectable)"),
    ((0x07 << 24) | (0x0c << 16) | (0xff << 8) | 0xff,"Correctable Machine Check Error"),

    ((0x08 << 24) | (0xff << 8) | 0xff,"Presence detected"),
    ((0x08 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Power Supply Failure detected"),
    ((0x08 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Predictive Failure"),
    ((0x08 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Power Supply input lost (AC/DC)"),
    ((0x08 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Power Supply input lost or out-of-range"),
    ((0x08 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Power Supply input out-of-range, but present"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8),"Configuration error : Vendor mismatch"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8) | 0x01,"Configuration error : Revision mismatch"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8) | 0x02,"Configuration error : Processor missing"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8) | 0x03,"Configuration error : Power Supply rating mismatch"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8) | 0x04,"Configuration error : Voltage rating mismatch"),
    ((0x08 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Configuration error"),
    ((0x08 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Power Supply Inactive (in standby state)"),

    ((0x09 << 24) | (0xff << 8) | 0xff,"Power Off / Power Down"),
    ((0x09 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Power Cycle"),
    ((0x09 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"240VA Power Down"),
    ((0x09 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Interlock Power Down"),
    ((0x09 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"AC lost / Power input lost"),
    ((0x09 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Soft Power Control Failure"),
    ((0x09 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Power Unit Failure detected"),
    ((0x09 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Predictive Failure"),

    ((0x0c << 24) | (0xff << 8) | 0xff,"Correctable ECC / other correctable memory error"),
    ((0x0c << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Uncorrectable ECC / other uncorrectable memory error"),
    ((0x0c << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Parity"),
    ((0x0c << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Memory Scrub Failed (stuck bit)"),
    ((0x0c << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Memory Device Disabled"),
    ((0x0c << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Correctable ECC / other correctable memory error logging limit reached"),
    ((0x0c << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Presence detected"),
    ((0x0c << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Configuration error"),
    ((0x0c << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Spare"),
    ((0x0c << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"Memory Automatically Throttled"),
    ((0x0c << 24) | (0x0a << 16) | (0xff << 8) | 0xff,"Critical Overtemperature"),

    ((0x0d << 24) | (0xff << 8) | 0xff,"Drive Presence"),
    ((0x0d << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Drive Fault"),
    ((0x0d << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Predictive Failure"),
    ((0x0d << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Hot Spare"),
    ((0x0d << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Consistency Check / Parity Check in progress"),
    ((0x0d << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"In Critical Array"),
    ((0x0d << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"In Failed Array"),
    ((0x0d << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Rebuild/Remap in progress"),
    ((0x0d << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Rebuild/Remap Aborted (was not completed normally)"),

    ((0x0f << 24) | 0xff,"System Firmware Error : Unspecified"),
    ((0x0f << 24) | (0x01 << 8) | 0xff,"System Firmware Error : No system memory is physically installed in the system"),
    ((0x0f << 24) | (0x02 << 8) | 0xff,"System Firmware Error : No usable system memory"),
    ((0x0f << 24) | (0x03 << 8) | 0xff,"System Firmware Error : Unrecoverable hard-disk/ATAPI IDE device failure"),
    ((0x0f << 24) | (0x04 << 8) | 0xff,"System Firmware Error : Unrecoverable system-board failure"),
    ((0x0f << 24) | (0x05 << 8) | 0xff,"System Firmware Error : Unrecoverable diskettesubsystem failure"),
    ((0x0f << 24) | (0x06 << 8) | 0xff,"System Firmware Error : Unrecoverable hard-disk controller failure"),
    ((0x0f << 24) | (0x07 << 8) | 0xff,"System Firmware Error : Unrecoverable PS/2 or USB keyboard failure"),
    ((0x0f << 24) | (0x08 << 8) | 0xff,"System Firmware Error : Removable boot media not),found"),
    ((0x0f << 24) | (0x09 << 8) | 0xff,"System Firmware Error : Unrecoverable video controller failure"),
    ((0x0f << 24) | (0x0a << 8) | 0xff,"System Firmware Error : No video device detected"),
    ((0x0f << 24) | (0x0b << 8) | 0xff,"System Firmware Error : Firmware (BIOS) ROM corruption detected"),
    ((0x0f << 24) | (0x0c << 8) | 0xff,"System Firmware Error : CPU voltage mismatch"),
    ((0x0f << 24) | (0x0d << 8) | 0xff,"System Firmware Error : CPU speed matching failure"),
    ((0x0f << 24) | (0xff << 8) | 0xff,"System Firmware Error"),
    ((0x0f << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"System Firmware Hang"),
    ((0x0f << 24) | (0x02 << 16) | 0xff,"System Firmware Progress : Unspecified"),
    ((0x0f << 24) | (0x02 << 16) | (0x01 << 8) | 0xff,"System Firmware Progress : Memory initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x02 << 8) | 0xff,"System Firmware Progress : Hard-disk initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x03 << 8) | 0xff,"System Firmware Progress : Secondary processor(s) initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x04 << 8) | 0xff,"System Firmware Progress : User authentication"),
    ((0x0f << 24) | (0x02 << 16) | (0x05 << 8) | 0xff,"System Firmware Progress : User-initiated system),setup"),
    ((0x0f << 24) | (0x02 << 16) | (0x06 << 8) | 0xff,"System Firmware Progress : USB resource configuration"),
    ((0x0f << 24) | (0x02 << 16) | (0x07 << 8) | 0xff,"System Firmware Progress : PCI resource configuration"),
    ((0x0f << 24) | (0x02 << 16) | (0x08 << 8) | 0xff,"System Firmware Progress : Option ROM initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x09 << 8) | 0xff,"System Firmware Progress : Video initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x0a << 8) | 0xff,"System Firmware Progress : Cache initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x0b << 8) | 0xff,"System Firmware Progress : SM Bus initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x0c << 8) | 0xff,"System Firmware Progress : Keyboard controller initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x0d << 8) | 0xff,"System Firmware Progress : Embedded controller management controller initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x0e << 8) | 0xff,"System Firmware Progress : Docking station attachment"),
    ((0x0f << 24) | (0x02 << 16) | (0x0f << 8) | 0xff,"System Firmware Progress : Enabling docking station"),
    ((0x0f << 24) | (0x02 << 16) | (0x10 << 8) | 0xff,"System Firmware Progress : Docking station ejection"),
    ((0x0f << 24) | (0x02 << 16) | (0x11 << 8) | 0xff,"System Firmware Progress : Disabling docking station"),
    ((0x0f << 24) | (0x02 << 16) | (0x12 << 8) | 0xff,"System Firmware Progress : Calling operating system wake-up vector"),
    ((0x0f << 24) | (0x02 << 16) | (0x13 << 8) | 0xff,"System Firmware Progress : Starting operating system boot process"),
    ((0x0f << 24) | (0x02 << 16) | (0x14 << 8) | 0xff,"System Firmware Progress : Baseboard or motherboard initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x15 << 8) | 0xff,"System Firmware Progress : reserved"),
    ((0x0f << 24) | (0x02 << 16) | (0x16 << 8) | 0xff,"System Firmware Progress : Floppy initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0x17 << 8) | 0xff,"System Firmware Progress : Keyboard test"),
    ((0x0f << 24) | (0x02 << 16) | (0x18 << 8) | 0xff,"System Firmware Progress : Pointing device test"),
    ((0x0f << 24) | (0x02 << 16) | (0x19 << 8) | 0xff,"System Firmware Progress : Primary processor initialization"),
    ((0x0f << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"System Firmware Progress"),

    ((0x10 << 24) | (0xff << 8) | 0xff,"Correctable Memory Error Logging Disabled"),
    ((0x10 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Event 'Type' Logging Disabled"),
    ((0x10 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Log Area Reset/Cleared"),
    ((0x10 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"All Event Logging Disabled"),
    ((0x10 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"SEL Full"),
    ((0x10 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"SEL Almost Full"),
    ((0x10 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Correctable Machine Check Error Logging Disabled"),

    ((0x11 << 24) | (0xff << 8) | 0xff,"BIOS Watchdog Reset"),
    ((0x11 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"OS Watchdog Reset"),
    ((0x11 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"OS Watchdog Shut Down"),
    ((0x11 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"OS Watchdog Power Down"),
    ((0x11 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"OS Watchdog Power Cycle"),
    ((0x11 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"OS Watchdog NMI / Diagnostic Interrupt"),
    ((0x11 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"OS Watchdog Expired, status only"),
    ((0x11 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"OS Watchdog pre-timeout Interrupt, non-NMI"),

    ((0x12 << 24) | (0xff << 8) | 0xff,"System Reconfigured"),
    ((0x12 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"OEM System Boot Event"),
    ((0x12 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Undetermined system hardware failure"),
    ((0x12 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Entry added to Auxiliary Log"),
    ((0x12 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"PEF Action"),
    ((0x12 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Timestamp Clock Synch"),

    ((0x13 << 24) | (0xff << 8) | 0xff,"Front Panel NMI / Diagnostic Interrupt"),
    ((0x13 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Bus Timeout"),
    ((0x13 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"I/O channel check NMI"),
    ((0x13 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Software NMI"),
    ((0x13 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"PCI PERR"),
    ((0x13 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"PCI SERR"),
    ((0x13 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"EISA Fail Safe Timeout"),
    ((0x13 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Bus Correctable Error"),
    ((0x13 << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Bus Uncorrectable Error"),
    ((0x13 << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"Fatal NMI"),
    ((0x13 << 24) | (0x0a << 16) | (0xff << 8) | 0xff,"Bus Fatal Error"),
    ((0x13 << 24) | (0x0b << 16) | (0xff << 8) | 0xff,"Bus Degraded"),

    ((0x14 << 24) | (0xff << 8) | 0xff,"Power Button pressed"),
    ((0x14 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Sleep Button pressed"),
    ((0x14 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Reset Button pressed"),
    ((0x14 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"FRU latch open"),
    ((0x14 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"FRU service request button"),

    ((0x19 << 24) | (0xff << 8) | 0xff,"Soft Power Control Failure"),
    ((0x19 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Thermal Trip"),

    ((0x1b << 24) | (0xff << 8) | 0xff,"Cable/Interconnect is connected"),
    ((0x1b << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Configuration Error"),

    ((0x1d << 24) | (0xff << 8) | 0xff,"Initiated by power up"),
    ((0x1d << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Initiated by hard reset"),
    ((0x1d << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Initiated by warm reset"),
    ((0x1d << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"User requested PXE boot"),
    ((0x1d << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Automatic boot to diagnostic"),
    ((0x1d << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"OS / run-time software initiated hard reset"),
    ((0x1d << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"OS / run-time software initiated warm reset"),
    ((0x1d << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"System Restart"),

    ((0x1e << 24) | (0xff << 8) | 0xff,"No bootable media"),
    ((0x1e << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Non-bootable diskette left in drive"),
    ((0x1e << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"PXE Server not found"),
    ((0x1e << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Invalid boot sector"),
    ((0x1e << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Timeout waiting for user selection of boot source"),

    ((0x1f << 24) | (0xff << 8) | 0xff,"A: boot completed"),
    ((0x1f << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"C: boot completed"),
    ((0x1f << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"PXE boot completed"),
    ((0x1f << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Diagnostic boot completed"),
    ((0x1f << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"CD-ROM boot completed"),
    ((0x1f << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"ROM boot completed"),
    ((0x1f << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"boot completed - boot device not specified"),
    ((0x1f << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Base OS/Hypervisor Installation started"),
    ((0x1f << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Base OS/Hypervisor Installation completed"),
    ((0x1f << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"Base OS/Hypervisor Installation aborted"),
    ((0x1f << 24) | (0x0a << 16) | (0xff << 8) | 0xff,"Base OS/Hypervisor Installation failed"),

    ((0x20 << 24) | (0xff << 8) | 0xff,"Critical stop during OS load / initialization"),
    ((0x20 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Run-time Critical Stop"),
    ((0x20 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"OS Graceful Stop"),
    ((0x20 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"OS Graceful Shutdown"),
    ((0x20 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Soft Shutdown initiated by PEF"),
    ((0x20 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Agent Not Responding"),

    ((0x21 << 24) | (0xff << 8) | 0xff,"Fault Status asserted"),
    ((0x21 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Identify Status asserted"),
    ((0x21 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Slot/Connector Device installed/attached"),
    ((0x21 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Slot/Connector Ready for Device Installation"),
    ((0x21 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Slot/Connector Ready for Device Removal"),
    ((0x21 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Slot Power is Off"),
    ((0x21 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Slot/Connector Device Removal Request"),
    ((0x21 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Interlock asserted"),
    ((0x21 << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Slot is Disabled"),
    ((0x21 << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"Slot holds spare device"),

    ((0x22 << 24) | (0xff << 8) | 0xff,"S0/G0 \"working\""),
    ((0x22 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"S1 \"sleeping with system h/w & processor context maintained\""),
    ((0x22 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"S2 \"sleeping, processor context lost\""),
    ((0x22 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"S3 \"sleeping, processor & h/w context lost, memory retained\""),
    ((0x22 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"S4 \"non-volatile sleep / suspend-to disk\""),
    ((0x22 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"S5/G2 \"soft-off\""),
    ((0x22 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"S4/S5 soft-off, particular S4 / S5 state cannot be determined"),
    ((0x22 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"G3/Mechanical Off"),
    ((0x22 << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Sleeping in an S1, S2, or S3 states"),
    ((0x22 << 24) | (0x09 << 16) | (0xff << 8) | 0xff,"G1 sleeping"),
    ((0x22 << 24) | (0x0a << 16) | (0xff << 8) | 0xff,"S5 entered by override"),
    ((0x22 << 24) | (0x0b << 16) | (0xff << 8) | 0xff,"Legacy ON state"),
    ((0x22 << 24) | (0x0c << 16) | (0xff << 8) | 0xff,"Legacy OFF state"),
    ((0x22 << 24) | (0x0e << 16) | (0xff << 8) | 0xff,"Unknown"),

    ((0x23 << 24) | (0xff << 8) | 0xff,"Timer expired, status only"),
    ((0x23 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Hard Reset"),
    ((0x23 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Power Down"),
    ((0x23 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Power Cycle"),
    ((0x23 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"reserved"),
    ((0x23 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"reserved"),
    ((0x23 << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"reserved"),
    ((0x23 << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"reserved"),
    ((0x23 << 24) | (0x08 << 16) | (0xff << 8) | 0xff,"Timer interrupt"),

    ((0x24 << 24) | (0xff << 8) | 0xff,"platform generated page"),
    ((0x24 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"platform generated LAN alert"),
    ((0x24 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Platform Event Trap generated"),
    ((0x24 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"platform generated SNMP trap, OEM format"),

    ((0x25 << 24) | (0xff << 8) | 0xff,"Entity Present"),
    ((0x25 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Entity Absent"),
    ((0x25 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Entity Disabled"),

    ((0x27 << 24) | (0xff << 8) | 0xff,"LAN Heartbeat Lost"),
    ((0x27 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"LAN Heartbeat"),

    ((0x28 << 24) | (0xff << 8) | 0xff,"sensor access degraded or unavailable"),
    ((0x28 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"controller access degraded or unavailable"),
    ((0x28 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"management controller off-line"),
    ((0x28 << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"management controller unavailable"),
    ((0x28 << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"sensor failure"),
    ((0x28 << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"FRU failure"),

    ((0x29 << 24) | (0xff << 8) | 0xff,"battery low"),
    ((0x29 << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"battery failed"),
    ((0x29 << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"battery presence detected"),

    ((0x2a << 24) | (0xff << 8) | 0xff,"Session Activated"),
    ((0x2a << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Session Deactivated"),
    ((0x2a << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Invalid Username or Password"),
    ((0x2a << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Invalid Password Disable"),

    ((0x2b << 24) | (0xff << 8) | 0xff,"Hardware change detected with associated Entity"),
    ((0x2b << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"Firmware or software change detected with associated Entity"),
    ((0x2b << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"Hardware incompatibility detected with associated Entity"),
    ((0x2b << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"Firmware or software incompatibility detected with associated Entity"),
    ((0x2b << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"Entity is of an invalid or unsupported hardware version"),
    ((0x2b << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"Entity contains an invalid or unsupported firmware or software version"),
    ((0x2b << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"Hardware Change detected with associated Entity was successful"),
    ((0x2b << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"Software or F/W Change detected with associated Entity was successful"),

    ((0x2c << 24) | (0xff << 8) | 0xff,"FRU Not Installed"),
    ((0x2c << 24) | (0x01 << 16) | (0xff << 8) | 0xff,"FRU Inactive"),
    ((0x2c << 24) | (0x02 << 16) | (0xff << 8) | 0xff,"FRU Activation Requested"),
    ((0x2c << 24) | (0x03 << 16) | (0xff << 8) | 0xff,"FRU Activation In Progress"),
    ((0x2c << 24) | (0x04 << 16) | (0xff << 8) | 0xff,"FRU Active"),
    ((0x2c << 24) | (0x05 << 16) | (0xff << 8) | 0xff,"FRU Deactivation Requested"),
    ((0x2c << 24) | (0x06 << 16) | (0xff << 8) | 0xff,"FRU Deactivation In Progress"),
    ((0x2c << 24) | (0x07 << 16) | (0xff << 8) | 0xff,"FRU Communication Lost")])
  };
}
