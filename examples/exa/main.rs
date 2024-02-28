// use std::io::Read;

use std::env;

use rust_ipmi::{GetSelEntry, GetSelInfo, IPMIClient, SelEntry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut ev = env::args().collect::<Vec<String>>();
    if ev.len() < 2 {
        ev = vec![
            "".into(),
            "172.18.10.25".into(),
            "admin".into(),
            "W129404@106-2U".into(),
        ];
    }
    let client_inactived = IPMIClient::new(format!("{}:623", ev[1])).await.unwrap();
    let mut c = client_inactived
        .activate(&ev[2], &ev[3])
        .await
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    // let get_sel_cmd = GetSelEntry::new(0, 0, 0);
    // let payload = get_sel_cmd.payload();
    // let packet = Packet::new(
    //     RmcpHeader::default(),
    //     IpmiHeader::V2_0(IpmiV2Header::new_est(32)),
    //     payload,
    // );
    let res = c
        .send_ipmi_cmd(GetSelInfo)
        .await
        // .send_packet(packet)
        // .send_raw_request(&[0x0A, 0x23, 0x00, 0x00, 0xff, 0x0ff, 0x00, 0x0e])
        // .send_raw_request(&[0x0A, 0x43, 0, 0, 0, 0, 0, 0xff])
        .unwrap();
    println!("res: {res:?}");

    let mut next = 0;

    let mut records: Vec<SelEntry> = Vec::new();

    while next != u16::from_le_bytes([0xff, 0xff]) {
        let res = c.send_ipmi_cmd(GetSelEntry::new(0, next, 0)).await.unwrap();
        next = u16::from_le_bytes(res.next_record_id);
        records.push(res.entry);
    }

    println!("entry: {:#?}", records);
    Ok(())
}
