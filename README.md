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
use rust_ipmi::{IPMIClient, NetFn, GetSelInfo, IpmiHeader, IpmiV2Header, RmcpHeader};

fn main() {
    // create the client for the server you want to execute IPMI commands against
    let mut client = IPMIClient::new("192.168.1.100:623")
        .unwrap()
        .activate("admin", "admin")
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    // let packet = Packet::new(
    //     RmcpHeader::default(),
    //     IpmiHeader::V2_0(IpmiV2Header::new_est(32)),
    //     payload,
    // );

    let resp = client
        // get first sel record raw command
        // .send_ipmi_cmd(GetSelInfo)
        // .send_packet(packet)
        .send_raw_request(&[0x0A, 0x43, 0, 0, 0, 0, 0, 0xff])
        .unwrap();

    println!("resp: {:?}", resp);
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
