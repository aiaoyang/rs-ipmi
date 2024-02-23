// use std::io::Read;

use rust_ipmi::IPMIClient;
fn main() {
    let mut c = IPMIClient::new("172.18.10.25:623").unwrap();
    c.establish_connection("admin", "admin")
        .unwrap_or_else(|e| println!("{e:?}"));

    let resp = c
        .send_raw_request(&[0x0A, 0x23, 0x00, 0x00, 0xff, 0x0ff, 0x00, 0x0e])
        // .send_raw_request(&[0x0A, 0x43, 0, 0, 0, 0, 0, 0xff])
        .unwrap();

    println!("resp: {:?}", resp);
    let data = resp.data;
    println!("{:?}", data);
    // if !data.is_empty() {
    //     // println!("respdata: {:x?},len: {}", data, data.len());
    //     let entry = rust_ipmi::Entry::parse(&data[2..]).unwrap();
    //     println!("entry: {entry}");
    // } else {
    //     println!("{:?}", data);
    // }
}
