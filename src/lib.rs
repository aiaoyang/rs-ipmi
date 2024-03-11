//!
//! rust-ipmi is a client library for remotely managing/monitoring systems with hardware support for IPMI.
//! IPMI is a specification which allows software to interact and communicate with systems through the BMC
//! (Baseboard Management Controller). BMC is a hardware component which enables interaction with a computer's
//! chassis, motherboard, and storage through LAN and serial.This library currently supports the following:
//!
//! * IPMI over LAN using IPMI v2 / RMCP+ (it does NOT currently support v1.5)
//!
//! # Examples
//!
//! Creating an ipmi client, authenticating against the BMC, and running a raw request
//!
//! ```no_run
//! use rust_ipmi::{IPMIClient, NetFn};
//!
//!     // create the client for the server you want to execute IPMI commands against
//!     let mut client: IPMIClient =
//!         IPMIClient::new("192.168.88.10:623").expect("Failed to create ipmi client");
//!
//!     // establish a session with the BMC using the credentials specified
//!     client
//!         .establish_connection("billybob123", "superpassword")
//!         .expect("Failed to establish the session with the BMC");
//!     
//!     // send a command to the BMC using raw values
//!     let response = client.send_raw_request(NetFn::App, 0x3b, Some(vec![0x04]));
//!
//!     match response {
//!         Err(err) => println!("Failed to send the raw request; err = {:?}", err),
//!         Ok(n) => println!("{}", n), // print the response
//!     }
//!
//! ```

pub use client::{IPMIClient, SessionActived, SessionInactived};
pub use err::*;
pub use rmcp::*;

mod client;
mod err;
mod rmcp;

pub const fn u8_ms_bit(value: u8, index: u8) -> bool {
    if index > 7 {
        panic!("unsupported position")
    }
    (value << (index) >> 7) != 0
}
#[test]
fn position() {
    assert!(u8_ms_bit(0b1, 7));
    assert!(!u8_ms_bit(0b1, 6));
    assert!(!u8_ms_bit(0b10, 7));
    assert!(u8_ms_bit(0b10, 6));

    assert!(u8_ms_bit(0b1000_0000, 0));
    assert!(!u8_ms_bit(0b1000_0000, 7));
}
