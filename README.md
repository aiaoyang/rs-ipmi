# rust-ipmi
<!-- ![Crates.io Version](https://img.shields.io/crates/v/rust-ipmi?style=flat) -->
<a href="https://crates.io/crates/rust-ipmi"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/rust-ipmi"></a>
<a href="https://docs.rs/rust-ipmi/latest/rust_ipmi/"><img alt="docs.rs" src="https://img.shields.io/docsrs/rust-ipmi"></a>

[Website](https://crates.io/crates/rust-ipmi) | [API Docs](https://docs.rs/rust-ipmi/latest/rust_ipmi/)

rust-ipmi is a native rust client for remotely managing/monitoring systems with hardware support for IPMI. IPMI is a specification which allows software to interact and communicate with systems through the BMC (Baseboard Management Controller). BMC is a hardware component which enables interaction with a computer's chassis, motherboard, and storage through LAN and serial.


### Recent Changes
- v0.1.1 is live on crates.io 🥳. See release notes [here](https://github.com/htemuri/rust-ipmi/releases/tag/v0.1.1)!

###  Preface
This is a hobby project to learn some rust, and is NOT a library meant for production use. If you would like a stable, well featured IPMI LAN client, look into ipmi-tool - a CLI tool which has been maintained for over a decade.

### ⚠️ Security WARNING ⚠️

IPMI through LAN has multiple relatively well-documented security vulnerabilities. Here are a few suggestions to harden your security:
- Change the default IPMI username
- Keep port 623/UDP to the servers under a restrictive firewall
- Do not directly expose your servers to the WAN

### Example

Creating an ipmi client, authenticating against the BMC, and running a raw request
```rs
use rust_ipmi::{IPMIClient, NetFn};

fn main() {
    // create the client for the server you want to execute IPMI commands against
    let client_inactived = IPMIClient::new("192.168.1.100:623").unwrap();
    let mut client_actived = client_inactived
        .activate("admin", "admin")
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    let resp = client_actived
        // get first sel record raw command
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
