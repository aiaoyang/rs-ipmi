// use std::io::Read;

use rust_ipmi::IPMIClient;
fn main() {
    let client_inactived = IPMIClient::new("172.18.10.25:623").unwrap();
    let mut client_actived = client_inactived
        .activate("admin", "admin")
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    let resp = client_actived
        // .send_raw_request(&[0x0A, 0x23, 0x00, 0x00, 0xff, 0x0ff, 0x00, 0x0e])
        .send_raw_request(&[0x0A, 0x43, 0, 0, 0, 0, 0, 0xff])
        .unwrap();

    println!("resp: {:?}", resp);
}
