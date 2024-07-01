use std::env;

use rust_ipmi::{commands::ActivatePayloadCommand, IPMIClient};

// fn init_log() {
//     use std::io::Write;
//     let _ = env_logger::builder()
//         .default_format()
//         .format(|f, r| {
//             writeln!(
//                 f,
//                 "{} {} {}:{} - {}",
//                 chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
//                 r.level(),
//                 r.file().unwrap(),
//                 r.line().unwrap(),
//                 r.args(),
//             )
//         })
//         .parse_env("LOG")
//         .try_init();
// }

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
    let mut client = IPMIClient::new(format!("{}:623", ev[1]), &ev[2], &ev[2])
        .await
        .unwrap()
        .activate()
        .await
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    let cmd = ActivatePayloadCommand {};
    let resp = client.send_ipmi_cmd(&cmd).await;
    match resp {
        Ok(_) => (),
        Err(e) => println!("ERROR: {e:#?}"),
    }
    Ok(())
}
