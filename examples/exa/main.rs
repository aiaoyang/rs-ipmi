// use std::io::Read;

use rust_ipmi::IPMIClient;
fn main() {
    let mut c = IPMIClient::new("172.18.10.25:623").unwrap();
    c.establish_connection("admin", "admin")
        .unwrap_or_else(|e| println!("{e:?}"));

    let resp = c
        .send_raw_request_new(&[0x0A, 0x43, 0, 0, 0, 0, 0, 0xff])
        .unwrap();

    println!("resp: {}", resp);
    if let Some(data) = resp.data {
        // println!("respdata: {:x?},len: {}", data, data.len());
        let entry = rust_ipmi::Entry::parse(&data[2..]).unwrap();
        println!("entry: {entry}");
    } else {
        println!("no data")
    }
}

#[test]
fn t() {
    let a = u16::from_le_bytes([1, 4]);
    println!("{a}");
}
