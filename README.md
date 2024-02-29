# rust-ipmi
## this project inspired by
1. rust-ipmi(main by this)
2. ipmi-rs
3. ipmigo
### ⚠️ Security WARNING ⚠️

IPMI through LAN has multiple relatively well-documented security vulnerabilities. Here are a few suggestions to harden your security:
- Change the default IPMI username
- Keep port 623/UDP to the servers under a restrictive firewall
- Do not directly expose your servers to the WAN

### Example

Creating an ipmi client, authenticating against the BMC, and running a raw request
```rs
use std::env;

use rust_ipmi::{GetSelEntry, GetSelInfo, IPMIClient, SelEntry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut ev = env::args().collect::<Vec<String>>();
    if ev.len() < 2 {
        ev = vec![
            "".into(),
            "192.168.1.100".into(),
            "admin".into(),
            "admin".into(),
        ];
    }
    let client_inactived = IPMIClient::new(format!("{}:623", ev[1])).await.unwrap();
    let mut c = client_inactived
        .activate(&ev[2], &ev[3])
        .await
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    let res = c.send_ipmi_cmd(GetSelInfo).await.unwrap();

    let counter = 10;

    let first_offset_id = if res.entries > counter {
        let first_record = c.send_ipmi_cmd(GetSelEntry::new(0, 0, 0)).await.unwrap();
        let delta = u16::from_le_bytes(first_record.next_record_id) - first_record.entry.id();
        (res.entries - counter + 1) * delta
    } else {
        0
    };

    let mut next = first_offset_id;

    let mut records: Vec<SelEntry> = Vec::new();
    while next != u16::from_le_bytes([0xff, 0xff]) {
        let res = c.send_ipmi_cmd(GetSelEntry::new(0, next, 0)).await.unwrap();
        next = u16::from_le_bytes(res.next_record_id);
        records.push(res.entry);
    }

    println!("entry: {:#?}", records);
    Ok(())
}

```

<!-- ## Design documentation for rust-ipmi -->
<!--
## Background

rust-ipmi is a native rust client for remotely managing/monitoring systems with hardware support for IPMI. IPMI is a specification which allows software to interact and communicate with systems through the BMC (Baseboard Management Controller). BMC is a hardware component which enables interaction with a computer's chassis, motherboard, and storage through LAN and serial.
-->
<!-- ![IPMI Block diagram](/images/ipmi.png) -->

<!-- This library is focusing on the IPMI LAN protocol. Some general information on IPMI through LAN:
1. This is a network-based implementation of IPMI so network packets will be sent to and received from the BMC LAN controller on port 623 through UDP.
2. The packet structure generally looks like Ethernet frame -> IP/UDP -> RMCP header -> IPMI header -> IPMI payload
3. Intel came out with a IPMI v2 and RMCP+ which introduced encrypted payloads
-->
<!-- ## Requirements for this library

- Support IPMI v1.5/2 RMCP/RMCP+
- Support most common APP and CHASSIS commands -->
