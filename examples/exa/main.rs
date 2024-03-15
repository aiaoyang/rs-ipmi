use std::env;

use rust_ipmi::{
    commands::{
        device_sdr::GetDeviceSdrCommand,
        reading::{GetSensorReading, ThresholdReading},
        repository::GetSDRRepositoryInfoCommand,
        reserve_repository::ReserveSDRRepositoryCommand,
        GetSelEntry, GetSelInfo,
    },
    storage::sdr::{
        record::{RecordContents, SensorRecord},
        RecordId,
    },
    IPMIClient, SessionActived,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut ev = env::args().collect::<Vec<String>>();
    if ev.len() < 2 {
        ev = vec![
            "".into(),
            "172.18.10.25".into(),
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

    get_sdr(&mut c).await;
    Ok(())
}

async fn get_sel(c: &mut IPMIClient<SessionActived>) {
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
}

async fn get_sdr(c: &mut IPMIClient<SessionActived>) {
    let sdr_repo_info = c.send_ipmi_cmd(GetSDRRepositoryInfoCommand).await.unwrap();
    println!("sdr repo info: {:?}", sdr_repo_info);
    let sdr_repo = c.send_ipmi_cmd(ReserveSDRRepositoryCommand).await.unwrap();
    println!("reserv sdr repo info: {:?}", sdr_repo);

    let mut next_id = RecordId::FIRST;
    while next_id != RecordId::LAST {
        let sdr_cmd = GetDeviceSdrCommand::new(None, next_id);
        let sdr_entry = c.send_ipmi_cmd(sdr_cmd).await.unwrap();
        next_id = sdr_entry.next_entry;

        if let RecordContents::FullSensor(full) = sdr_entry.record.contents {
            let value = c
                .send_ipmi_cmd(GetSensorReading::for_sensor_key(full.key_data()))
                .await
                .unwrap();
            let Some(reading) = ThresholdReading::from(&value).reading else {
                continue;
            };
            if let Some(display) = full.display_reading(reading) {
                println!("{} \t\t| {}", full.id_string(), display);
            }
        }
    }
}
