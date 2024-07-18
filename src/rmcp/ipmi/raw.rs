pub struct IpmiRawRequest(&'static [u8]);

impl IpmiRawRequest {
    pub fn new(data: &'static [u8]) -> IpmiRawRequest {
        IpmiRawRequest(data)
    }

    // pub fn create_packet(&self) -> Packet {
    //     Packet::new_raw(
    //         RmcpHeader::default(),
    //         IpmiSession::V2_0(IpmiV2Session::new_est(32)),
    //         self.0,
    //     )
    // }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    use crate::request::checksum;

    #[tokio::test]
    async fn gen_raw_command() {
        // let mut client = IPMIClient::new("172.18.10.25:623".to_string(), "admin", "admin")
        //     .await
        //     .unwrap()
        //     .activate()
        //     .await
        //     .unwrap();
        //
        // let mut buf = [0_u8; 1024];
        // let n = GetSelInfo::write_to(&GetSelInfo, &mut buf);
        //
        // let mut buf = [0_u8; 1024];
        // // build_v2_payload(channel, bridge, &mut buf);
        // build_test(&mut buf);
        //
        // let raw_req = IpmiRawRequest::new(&buf[..7]);
        // assert_eq!(
        //     &buf[0..n],
        //     <Vec<u8>>::from(&raw_req.clone().create_packet())
        // );
        //
        // let res = client.send_raw_request().await.unwrap();
        // println!("resp: {res:#x?}");
        // let n = client.recv(&mut buf).await.unwrap();
        // println!("data: {:x?}", &buf[..n]);
        // let packet = Packet::try_from((&buf[..n], &client.session.k2)).unwrap();
        // println!("resp: {packet:#x?}");
    }

    #[cfg(test)]
    #[allow(unused)]
    fn build_v2_payload(channel: u8, target_addr: u8, msg: &mut [u8]) {
        use crate::request::checksum;

        const IPMI_BMC_SLAVE_ADDR: u8 = 0x20;
        const IPMI_NETFN_APP: u8 = 0x06;
        const IPMI_REMOTE_SWID: u8 = 0x81;
        const SEND_MSG_COMMAND: u8 = 0x34;

        msg[0] = IPMI_BMC_SLAVE_ADDR;
        msg[1] = IPMI_NETFN_APP << 2;
        msg[2] = checksum(&msg[0..=1]);
        msg[3] = IPMI_REMOTE_SWID;
        msg[4] = 0;
        msg[5] = SEND_MSG_COMMAND;
        msg[6] = 0x40 | channel;

        // origin_payload
        msg[7] = target_addr;
        msg[8] = 0x2e << 2;
        msg[9] = checksum(&msg[7..=8]);
        msg[10] = IPMI_REMOTE_SWID;
        msg[11] = 0;
        msg[12] = 0x65;
        msg[13..=16].copy_from_slice(&[0x57, 0x01, 0x00, 0x01]);
        msg[17] = checksum(&msg[10..=16]);
        //
        msg[18] = checksum(&msg[3..=17]);
    }

    #[cfg(test)]
    fn build_test(msg: &mut [u8]) {
        const IPMI_BMC_SLAVE_ADDR: u8 = 0x20;
        const IPMI_NETFN_STORAGE: u16 = 0x0A;
        const IPMI_REMOTE_SWID: u8 = 0x81;
        const SEL_COMMAND: u8 = 0x40;

        msg[0] = IPMI_BMC_SLAVE_ADDR;
        msg[1] = (IPMI_NETFN_STORAGE << 2) as u8;
        msg[2] = checksum(&msg[0..=1]);
        msg[3] = IPMI_REMOTE_SWID;
        msg[4] = 0x08 << 2;
        msg[5] = SEL_COMMAND;
        msg[6] = checksum(&msg[3..=5]);
    }
}
