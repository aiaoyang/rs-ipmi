use std::{env, time::Duration};

use rust_ipmi::{commands::ActivatePayloadCommand, IPMIClient, Packet};

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
        .with_timeout(Duration::from_secs(10))
        .activate()
        .await
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    let cmd = ActivatePayloadCommand { data: None };
    let resp = client.send_ipmi_cmd(&cmd).await;
    match resp {
        Ok(_) => (),
        Err(e) => println!("ERROR: {e:#?}"),
    }
    // uint8_t data[IPMI_BUF_SIZE];
    // uint16_t character_count;
    // uint8_t packet_sequence_number;
    // uint8_t acked_packet_number;
    // uint8_t accepted_character_count;
    // uint8_t is_nack;	/* bool */
    // uint8_t assert_ring_wor;	/* bool */
    // uint8_t generate_break;	/* bool */
    // uint8_t deassert_cts;	/* bool */
    // uint8_t deassert_dcd_dsr;	/* bool */
    // uint8_t flush_inbound;	/* bool */
    // uint8_t flush_outbound;	/*

    // msg[i++] = payload->payload.sol_packet.packet_sequence_number;
    // msg[i++] = payload->payload.sol_packet.acked_packet_number;
    // msg[i++] = payload->payload.sol_packet.accepted_character_count;
    //
    // msg[i] = payload->payload.sol_packet.is_nack ? 0x40 : 0;
    // msg[i] |= payload->payload.sol_packet.assert_ring_wor ? 0x20 : 0;
    // msg[i] |= payload->payload.sol_packet.generate_break ? 0x10 : 0;
    // msg[i] |= payload->payload.sol_packet.deassert_cts ? 0x08 : 0;
    // msg[i] |= payload->payload.sol_packet.deassert_dcd_dsr ? 0x04 : 0;
    // msg[i] |= payload->payload.sol_packet.flush_inbound ? 0x02 : 0;
    // msg[i++] |= payload->payload.sol_packet.flush_outbound ? 0x01 : 0;

    let data = &[b'l', b's'];
    let mut packet = Packet::new_sol_packet(data);
    let resp = client.send_and_decrypt_packet(&mut packet).await;
    match resp {
        Ok(packet) => {
            if let rust_ipmi::PacketPayload::Sol(sol) = packet.data {
                println!("resp ==>{:?}", String::from_utf8_lossy(&sol.data));
            };
        }
        Err(e) => println!("ERROR: {e:#?}"),
    }
    client.loop_sol_data().await.unwrap();
    // client.loop_sol_data().await.unwrap();
    Ok(())
}
