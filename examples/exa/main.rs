use std::{env, time::Duration};

use rust_ipmi::{
    commands::{
        device_sdr::GetDeviceSdr,
        reading::{GetRawSensorReading, ThresholdReading},
        repository::GetSDRRepositoryInfo,
        reserve_repository::GetReserveSDRRepository,
        GetSelEntry, GetSelInfo,
    },
    storage::sdr::{RecordId, SensorRecord},
    IPMIClient, SessionActived,
};
use tokio::time::sleep;

fn init_log() {
    use std::io::Write;
    let _ = env_logger::builder()
        .default_format()
        .format(|f, r| {
            writeln!(
                f,
                "{} {} {}:{} - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                r.level(),
                r.file().unwrap(),
                r.line().unwrap(),
                r.args(),
            )
        })
        .parse_env("LOG")
        .try_init();
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_log();
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
    if ev.contains(&"sel".into()) {
        get_sel(&mut c).await;
    } else {
        get_sdr(&mut c, retain).await;
    }
    Ok(())
}

const RETAIN_SENSORS: [&str; 10] = [
    "PSU1_Status",
    "PSU2_Status",
    "PS1 Status",
    "PS2 Status",
    "Inlet Temp",
    "Outlet Temp",
    "FAN0_F_Speed",
    "FAN0_R_Speed",
    "FAN1_F_Speed",
    "FAN1_R_Speed",
];

fn retain(v: &str) -> bool {
    RETAIN_SENSORS.contains(&v)
}

async fn get_sel(c: &mut IPMIClient<SessionActived>) {
    let res = c.send_ipmi_cmd(&GetSelInfo).await.unwrap();

    let counter = 10;

    let first_offset_id = if res.entries > counter {
        let first_record = c.send_ipmi_cmd(&GetSelEntry::new(0, 0, 0)).await.unwrap();
        let delta = u16::from_le_bytes(first_record.next_record_id) - first_record.entry.id();
        (res.entries - counter + 1) * delta
    } else {
        0
    };

    let mut next = first_offset_id;

    while next != u16::from_le_bytes([0xff, 0xff]) {
        let res = c
            .send_ipmi_cmd(&GetSelEntry::new(0, next, 0))
            .await
            .unwrap();
        next = u16::from_le_bytes(res.next_record_id);
        println!("{:?}", res.entry.description_with_assetion(),);
    }
}

async fn get_sdr<F: Fn(&str) -> bool>(c: &mut IPMIClient<SessionActived>, retain: F) {
    let sdr_repo_info = c.send_ipmi_cmd(&GetSDRRepositoryInfo).await.unwrap();
    println!("sdr repo info: {:?}", sdr_repo_info);
    let sdr_repo = c.send_ipmi_cmd(&GetReserveSDRRepository).await.unwrap();
    println!("reserv sdr repo info: {:?}", sdr_repo);

    let mut cmds = Vec::new();
    let mut next_id = RecordId::FIRST;
    while next_id != RecordId::LAST {
        let sdr_cmd = GetDeviceSdr::new(None, next_id);
        let sdr_entry = c.send_ipmi_cmd(&sdr_cmd).await.unwrap();
        next_id = sdr_entry.next_entry;

        if let Some(full) = sdr_entry.record.full_sensor() {
            cmds.push((
                full.clone(),
                GetRawSensorReading::form_sensor_key(full.key_data()),
            ));
        };
    }
    let mut idx = cmds.len();
    while idx > 0 {
        idx -= 1;
        if !retain(&format!("{}", cmds[idx].0.id_string())) {
            cmds.remove(idx);
        }
    }

    loop {
        for (full, cmd) in &cmds {
            let value = c.send_ipmi_cmd(cmd).await.unwrap();
            let Some(reading) = ThresholdReading::from(&value).reading else {
                continue;
            };
            let Some(display) = full.h_value(reading) else {
                continue;
            };
            println!(
                "{} \t\t| {} \t| {:?}",
                full.id_string(),
                display,
                full.common().event_reading_type_code
            );
        }
        println!("\n\n\n\n");
        sleep(Duration::from_secs(10)).await;
    }
}
