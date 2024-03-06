use std::env;

use rust_ipmi::{GetSelEntry, GetSelInfo, IPMIClient};

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

    while next != u16::from_le_bytes([0xff, 0xff]) {
        let res = c.send_ipmi_cmd(GetSelEntry::new(0, next, 0)).await.unwrap();
        next = u16::from_le_bytes(res.next_record_id);
        println!("{:?}", res.entry.description_with_assetion(),);
    }

    Ok(())
}
