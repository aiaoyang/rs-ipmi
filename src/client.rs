use std::{fmt::Debug, sync::Arc, time::Duration};

use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::{
    commands::{GetChannelAuthCapabilitiesRequest, Privilege},
    crypto::aes_128_cbc_encrypt,
    err::{EClient, Error},
    rmcp::{
        crypto::hash_hmac_sha1,
        open_session::{
            AuthAlgorithm, ConfidentialityAlgorithm, IntegrityAlgorithm,
            RMCPPlusOpenSessionPayload, RMCPPlusOpenSessionRequest, StatusCode,
        },
        rakp::{RAKPMessage1, RAKPMessage3, RakpPayload},
        response::IpmiRespPayload,
        Packet, PacketPayload,
    },
    IpmiCommand, PacketBuilder,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone)]
pub enum State {
    Discovery,
    // Authentication,
    Established,
}

#[derive(Clone)]
pub struct SessionInactived {
    state: State,
    seq_number: u32,
    managed_id: Option<u32>,
    managed_system_random_number: Option<u128>,
    managed_system_guid: Option<u128>,
    remote_console_session_id: Option<u32>,
    remote_console_random_number: u128,

    password_mac_key: Option<Vec<u8>>,
    sik: Option<[u8; 20]>,
    k1: Option<[u8; 20]>,
    k2: Option<[u8; 20]>,
}

impl Default for SessionInactived {
    fn default() -> Self {
        Self {
            state: State::Discovery,
            seq_number: 0x0000_0001,
            managed_id: Default::default(),
            managed_system_random_number: Default::default(),
            managed_system_guid: Default::default(),
            remote_console_session_id: Default::default(),
            remote_console_random_number: 0,
            password_mac_key: Default::default(),
            sik: Default::default(),
            k1: Default::default(),
            k2: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct SessionActived {
    pub seq_number: u32,
    pub managed_id: u32,
    pub k1: [u8; 20],
    pub k2: [u8; 20],
}

/// default retry_duration is 300ms
#[derive(Debug, Clone)]
pub struct IPMIClient<SESSION: Clone, SOCKET, A: ToSocketAddrs = String> {
    conf: ClientOption<A>,
    socket: SOCKET,
    pub(crate) session: SESSION,

    snd_buf: [u8; 1024],
    recv_buf: [u8; 1024],
}

#[derive(Debug, Clone)]
pub(crate) struct ClientOption<A: ToSocketAddrs = String> {
    remote_addr: A,
    // privilege: Privilege,
    username: Box<str>,
    password: Box<str>,
    // cipher_list_index: u8,
    socket_recv_timeout: Duration,
    retry: u8,
    retry_duration: Duration,
    auto_reconnect: bool,
}

pub type ActiveClient = IPMIClient<SessionActived, Arc<UdpSocket>>;
pub type InActiveClient = IPMIClient<SessionInactived, ()>;

impl InActiveClient {
    /// Creates client for running IPMI commands against a BMC.
    ///
    /// # Arguments
    /// * `ipmi_server_addr` - Socket address of the IPMI server (or BMC LAN controller).
    /// Default port for IPMI RMCP is 623 UDP.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use rust_ipmi::{IPMIClient, Error, SessionInactived};
    ///     let ipmi_server = "192.168.1.10:623";
    ///     let username = "username";
    ///     let password = "password";
    ///     let ipmi_client: Result<IPMIClient<SessionInactived>, Error> = IPMIClient::new(
    ///          ipmi_server,
    ///          username,
    ///          password)
    ///         .await;
    /// # })
    /// ```
    pub async fn new(
        ipmi_server_addr: String,
        username: &str,
        password: &str,
    ) -> Result<InActiveClient> {
        let client_socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(EClient::FailedBind)?;
        client_socket
            .connect(&ipmi_server_addr)
            .await
            .map_err(|e| EClient::ConnectToIPMIServer(format!("{}: {:?}", e, &ipmi_server_addr)))?;
        Ok(IPMIClient {
            socket: (),
            session: SessionInactived::default(),
            conf: ClientOption {
                remote_addr: ipmi_server_addr,
                username: Box::from(username),
                password: Box::from(password),
                // cipher_list_index: 0,
                // privilege: Privilege::Callback,
                socket_recv_timeout: Duration::from_secs(5),
                retry: 0,
                retry_duration: Duration::from_millis(300),
                auto_reconnect: false,
            },
            snd_buf: [0; 1024],
            recv_buf: [0; 1024],
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.conf.socket_recv_timeout = timeout;
        self
    }

    pub fn retry(mut self, n: u8) -> Self {
        self.conf.retry = n;
        self
    }

    pub fn retry_duration(mut self, duration: Duration) -> Self {
        self.conf.retry_duration = duration;
        self
    }

    pub fn auto_reconnect(mut self, b: bool) -> Self {
        self.conf.auto_reconnect = b;
        self
    }
    /// Set the read timeout on the ipmi client UDP socket. Default timeout for the socket is set to 20 seconds
    ///
    /// # Arguments
    /// * `duration` - The timeout duration. If set to 5 seconds, the ipmi client will wait up to 5 seconds for a response from the BMC until timing out
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_ipmi::ipmi_client::IPMIClient;
    ///
    /// let ipmi_server = "192.168.1.10:623"
    /// let ipmi_client = IPMIClient::new(ipmi_server).expect("Failed to connect to the IPMI server");
    /// ipmi_client.set_read_timeout(Some(time::Duration::from_secs(10))).expect("Failed to set the timeout");
    /// ```
    // pub fn set_read_timeout(self, duration: Option<Duration>) -> Result<Self> {
    //     self.client_socket
    //         .set_read_timeout(duration)
    //         .map_err(IPMIClientError::SetReadTimeOutError)?;
    //     Ok(self)
    // }

    /// Authenticates and establishes a session with the BMC.
    ///
    /// # Arguments
    /// * `username` - username used to authenticate against the BMC.
    /// * `password` - password for the username provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_ipmi::{IPMIClient, IPMIClientError};
    ///
    /// let ipmi_server = "192.168.1.10:623";
    /// let mut ipmi_client: IPMIClient = IPMIClient::new(ipmi_server)
    ///     .expect("Failed to connect to the server");
    ///
    /// let username = "my-username";
    /// ipmi_client.establish_connection(username, "password")
    ///     .map_err(|e: IPMIClientError| {
    ///         println!("Failed to establish session with BMC: {}", e);
    ///     });
    ///
    /// ```
    pub async fn activate(mut self) -> Result<ActiveClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(EClient::FailedBind)?;
        socket.connect(&self.conf.remote_addr).await.map_err(|e| {
            EClient::ConnectToIPMIServer(format!("{}: {:?}", e, &self.conf.remote_addr))
        })?;
        self.session.password_mac_key = Some(self.conf.password.as_bytes().to_vec());

        self.get_channel_auth_cap(&socket).await?; // Get channel auth capabilites and set cipher
        self.rmcpplus_session_authenticate(socket).await // rmcp open session and authenticate
                                                         // c.set_privilege_level()?;
    }

    async fn get_channel_auth_cap(&mut self, socket: &UdpSocket) -> Result<()> {
        println!("get_channel_auth_cap");

        let get_channel_auth_cap_packet =
            GetChannelAuthCapabilitiesRequest::new(true, 0xE, Privilege::Administrator);
        let packet =
            PacketBuilder::ipmi().write_payload(&mut self.snd_buf, get_channel_auth_cap_packet);

        Self::send_unauth_packet_retry(
            socket,
            &mut self.session,
            packet,
            &self.conf,
            &mut self.recv_buf,
        )
        .await?;

        Ok(())
    }

    fn gen_rk2_auth_code(&self) -> [u8; 20] {
        let mut buf: Vec<u8> = vec![0; 58 + self.conf.username.len()];
        buf[..4].copy_from_slice(
            &self
                .session
                .remote_console_session_id
                .unwrap()
                .to_le_bytes(),
        );
        buf[4..8].copy_from_slice(&self.session.managed_id.unwrap().to_le_bytes());
        buf[8..24].copy_from_slice(&self.session.remote_console_random_number.to_le_bytes());
        buf[24..40].copy_from_slice(
            &self
                .session
                .managed_system_random_number
                .unwrap()
                .to_le_bytes(),
        );
        buf[40..56].copy_from_slice(&self.session.managed_system_guid.unwrap().to_le_bytes());
        buf[56] = Self::privilege(false, Privilege::Administrator);
        buf[57] = self.conf.username.len().try_into().unwrap();
        buf[58..].copy_from_slice(self.conf.username.as_bytes());

        hash_hmac_sha1(self.session.password_mac_key.as_ref().unwrap(), &buf)
    }

    fn gen_rk3_auth_code(&self) -> [u8; 20] {
        let mut buf = Vec::new();
        append_u128_to_vec(&mut buf, self.session.managed_system_random_number.unwrap());
        append_u32_to_vec(&mut buf, self.session.remote_console_session_id.unwrap());

        buf.push(Self::privilege(false, Privilege::Administrator));
        buf.push(self.conf.username.len().try_into().unwrap());
        self.conf
            .username
            .as_bytes()
            .iter()
            .for_each(|char| buf.push(*char));

        hash_hmac_sha1(self.session.password_mac_key.as_ref().unwrap(), &buf)
    }

    fn gen_rk4_auth_code(&self) -> [u8; 12] {
        let mut buf: Vec<u8> = Vec::new();
        append_u128_to_vec(&mut buf, self.session.remote_console_random_number);
        append_u32_to_vec(&mut buf, self.session.managed_id.unwrap());
        append_u128_to_vec(&mut buf, self.session.managed_system_guid.unwrap());
        hash_hmac_sha1(&self.session.sik.unwrap(), &buf)[..12]
            .try_into()
            .unwrap()
    }

    async fn rmcpplus_session_authenticate(mut self, socket: UdpSocket) -> Result<ActiveClient> {
        println!("session authenticate");
        // RMCP+ Open Session Request
        let rmcp_plus_open_session_req = RMCPPlusOpenSessionRequest::new(
            0,
            Privilege::Administrator,
            0xa0a2a3a4,
            AuthAlgorithm::RakpHmacSha1,
            IntegrityAlgorithm::HmacSha196,
            ConfidentialityAlgorithm::AesCbc128,
        );
        let packet = PacketBuilder::open_session()
            .write_payload(&mut self.snd_buf, rmcp_plus_open_session_req);
        let resp = Self::send_unauth_packet_retry(
            &socket,
            &mut self.session,
            packet,
            &self.conf,
            &mut self.recv_buf,
        )
        .await?;
        let PacketPayload::Rmcp(RMCPPlusOpenSessionPayload::Response(msg)) = resp.data else {
            Err(EClient::MisformedResponse)?
        };

        let StatusCode::NoErrors = &msg.rmcp_plus_status_code else {
            Err(EClient::FailedToOpenSession(msg.rmcp_plus_status_code))?
        };
        self.session.managed_id = Some(msg.managed_system_session_id);

        // RAKP Message 1
        let rk1 = RAKPMessage1::new(
            0x0,
            self.session.managed_id.unwrap(),
            self.session.remote_console_random_number,
            false,
            Privilege::Administrator,
            self.conf.username.to_string(),
        );
        let packet = PacketBuilder::rakp1().write_payload(&mut self.snd_buf, rk1);

        // RAKP Message 2
        println!("send rakp1");
        let r2_pkt = Self::send_unauth_packet_retry(
            &socket,
            &mut self.session,
            packet,
            &self.conf,
            &mut self.recv_buf,
        )
        .await?;
        let PacketPayload::Rakp(RakpPayload::Message2(msg2)) = &r2_pkt.data else {
            Err(EClient::MisformedResponse)?
        };

        let StatusCode::NoErrors = &msg2.status_code else {
            Err(EClient::FailedToOpenSession(msg2.status_code))?
        };

        self.session.managed_system_guid = Some(msg2.managed_guid);
        self.session.remote_console_session_id = Some(msg2.console_id);
        self.session.managed_system_random_number = Some(msg2.managed_rnd_number);
        // validate BMC auth code

        if let Some(auth_code) = msg2.key_exchange_auth_code {
            if auth_code != self.gen_rk2_auth_code() {
                println!(
                    "calc: {:?}\ncome: {:?}",
                    self.gen_rk2_auth_code(),
                    msg2.key_exchange_auth_code.unwrap()
                );
                Err(EClient::FailedToValidateRAKP2)?
            }
        }
        self.create_session_keys()?;

        // RAKP Message 3
        let rakp3_packet = RAKPMessage3::new(
            0x0,
            StatusCode::NoErrors,
            self.session.managed_id.unwrap(),
            Some(self.gen_rk3_auth_code().into()),
        );
        let packet = PacketBuilder::rakp3().write_payload(&mut self.snd_buf, rakp3_packet);

        // RAKP Message 4
        println!("send rakp3");
        let r4_pkt = Self::send_unauth_packet_retry(
            &socket,
            &mut self.session,
            packet,
            &self.conf,
            &mut self.recv_buf,
        )
        .await?;

        let PacketPayload::Rakp(RakpPayload::Message4(msg4)) = r4_pkt.data else {
            Err(EClient::MisformedResponse)?
        };

        let StatusCode::NoErrors = msg4.status_code else {
            Err(EClient::FailedToOpenSession(msg4.status_code))?
        };
        if let Some(auth_code) = msg4.integrity_auth_code {
            if auth_code != self.gen_rk4_auth_code() {
                Err(EClient::MismatchedKeyExchangeAuthCode)?
            }
            self.session.state = State::Established;
        }

        Ok(ActiveClient {
            socket: Arc::new(socket),
            conf: self.conf,
            session: SessionActived {
                seq_number: self.session.seq_number,
                managed_id: self.session.managed_id.unwrap(),
                k1: self.session.k1.unwrap(),
                k2: self.session.k2.unwrap(),
            },
            snd_buf: self.snd_buf,
            recv_buf: self.recv_buf,
        })
        // Ok(IPMIClient::from(self))
    }

    // async fn send_unauth_packet<'a>(
    //     socket: &UdpSocket,
    //     session: &mut SessionInactived,
    //     request_packet: Packet<'a>,
    //     buf: &'a mut [u8],
    // ) -> Result<Packet<'a>> {
    //     let x = request_packet.write_to(buf);
    //     socket.send(&buf[..x]).await.map_err(EClient::FailedSend)?;
    //     let Ok((n_bytes, _)) = socket.recv_from(buf).await else {
    //         info!("send_unauth_packet");
    //         Err(EClient::NoResponse)?
    //     };
    //
    //     match session.k2 {
    //         Some(k2) => Ok(Packet::decode_from(buf, &k2, n_bytes)?),
    //         None => Ok(Packet::try_from(&buf[..n_bytes])?),
    //     }
    // }

    async fn send_unauth_packet_retry<'a>(
        socket: &UdpSocket,
        session: &mut SessionInactived,
        request_packet: &[u8],
        conf: &ClientOption,
        recv_buf: &'a mut [u8],
    ) -> Result<Packet<'a>> {
        let mut retry: i32 = conf.retry as i32;
        // println!("packet: {:x?}", &buf[..n]);

        while retry >= 0 {
            let Ok(n_bytes) =
                send_recv(socket, request_packet, recv_buf, conf.socket_recv_timeout).await
            else {
                if retry <= 0 {
                    // info!("send_unauth_packet");
                    Err(EClient::NoResponse)?
                } else {
                    tokio::time::sleep(conf.retry_duration).await;
                    retry -= 1;
                    // warn!("retry packet");
                    continue;
                }
            };

            if let Some(k2) = session.k2 {
                return Packet::decode_from(recv_buf, &k2, n_bytes);
            } else {
                return Packet::try_from(&recv_buf[..n_bytes]);
            };
        }
        Err(EClient::NoResponse)?
    }

    fn privilege(nameonly_lookup: bool, privilege: Privilege) -> u8 {
        if !nameonly_lookup {
            privilege as u8 | 0x10
        } else {
            privilege as u8
        }
    }

    fn create_session_keys(&mut self) -> Result<()> {
        let mut sik_input = Vec::new();
        append_u128_to_vec(&mut sik_input, self.session.remote_console_random_number);
        append_u128_to_vec(
            &mut sik_input,
            self.session.managed_system_random_number.unwrap(),
        );
        sik_input.push(0x14);
        sik_input.push(
            self.conf
                .username
                .len()
                .try_into()
                .map_err(EClient::UsernameOver255InLength)?,
        );
        self.conf
            .username
            .as_bytes()
            .iter()
            .for_each(|char| sik_input.push(*char));

        self.session.sik = Some(hash_hmac_sha1(
            self.session.password_mac_key.as_ref().unwrap(),
            &sik_input,
        ));
        self.session.k1 = Some(hash_hmac_sha1(&self.session.sik.unwrap(), &[1; 20]));
        self.session.k2 = Some(hash_hmac_sha1(&self.session.sik.unwrap(), &[2; 20]));

        Ok(())
    }
}

pub fn append_u32_to_vec(main_vec: &mut Vec<u8>, append: u32) {
    append.to_le_bytes().map(|byte| main_vec.push(byte));
}

pub fn append_u128_to_vec(main_vec: &mut Vec<u8>, append: u128) {
    append.to_le_bytes().map(|byte| main_vec.push(byte));
}

impl ActiveClient {
    pub async fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv(buf).await
    }

    pub async fn send_ipmi_cmd<CMD: IpmiCommand>(&mut self, ipmi_cmd: CMD) -> Result<CMD::Output> {
        let packet =
            PacketBuilder::ipmi_encrypted(self.session.managed_id, &mut self.session.seq_number)
                .write_payload(
                    &mut self.snd_buf,
                    ipmi_cmd,
                    &self.session.k2[..16].try_into().unwrap(),
                    &self.session.k1,
                );
        let n = send_recv(
            &self.socket,
            packet,
            &mut self.recv_buf,
            Duration::from_secs(1),
        )
        .await?;
        let resp_packet = Packet::decode_from(&mut self.recv_buf, &self.session.k2, n)?;
        println!("resp: {resp_packet:?}");

        // self.buffer.1 = ipmi_cmd.write_to(&mut self.buffer.0);
        //
        // let resp = match self.send_and_decrypt_packet(&mut packet).await {
        //     Ok(resp) => resp,
        //     Err(e) => {
        //         if self.conf.auto_reconnect {
        //             self.re_connect().await?;
        //         }
        //         return Err(e);
        //     }
        // };
        // let (data, code) = resp.data.data_and_completion();
        // if let Some(resp_cmd) = resp.data.command() {
        //     if <CMD as IpmiCommand>::COMMAND != resp_cmd {
        //         let addr = self.socket.peer_addr();
        //         let err = format!(
        //             "peer: {addr:?}, req packet command: {:?}, seq: {}, resp command: {:x?}, seq: {}",
        //             packet.data.command(),
        //             packet.ipmi_session.seq_num(),
        //             resp.data.command(),
        //             resp.ipmi_session.seq_num(),
        //         );
        //         if self.auto_reconnect {
        //             self.re_connect().await?;
        //         }
        //         return Err(Error::RawString(err));
        //     }
        // }
        //
        // let CompletionCode::CompletedNormally = code else {
        //     Err(EClient::CompletionCode((
        //         <CMD as IpmiCommand>::COMMAND,
        //         code,
        //     )))?
        // };
        //
        // ipmi_cmd.parse(data)
        todo!()
    }

    pub async fn send_and_decrypt_packet<'a>(
        &mut self,
        packet: &mut Packet<'a>,
    ) -> Result<Packet<'a>> {
        // Packet::try_from((
        //     self.send_packet_retry(packet).await?.as_slice(),
        //     &self.session.k2,
        // ))
        todo!()
    }

    pub async fn send_packet_retry<'a>(&'a mut self) -> Result<&'a [u8]> {
        // let mut n: i32 = self.conf.retry as i32;
        // let mut retry_counter = 0;
        // let retry_duration = self.conf.retry_duration;
        // let (buffer, len) = &mut self.buffer;
        // while n >= 0 {
        //     let res = send_packet(&self.socket, &self.conf, buffer, len, &mut self.session).await;
        //     if res.is_ok() || n <= 0 {
        //         return res;
        //     }
        //     retry_counter += 1;
        //     // warn!(
        //     //     "retry packet[{retry_counter}]time : {:?}",
        //     //     &self.buffer.0[..self.buffer.1]
        //     // );
        //     tokio::time::sleep(retry_duration).await;
        //     n -= 1;
        // }
        unreachable!()
    }

    pub async fn loop_sol_data(&mut self) -> Result<()> {
        // let mut recv_buf = Vec::with_capacity(1024);
        // let buf: &mut [u8] = &mut [0u8; 255];
        // loop {
        //     let n = self.socket.recv(buf).await.unwrap();
        //     let b = &buf[..n];
        //     let packet = Packet::try_from((b, &self.session.k2)).unwrap();
        //     println!("data : {:?}", packet);
        //     if let PacketPayload::Sol(sol) = packet.data {
        //         println!("resp ==>{:?}", String::from_utf8_lossy(&sol.data));
        //     };
        //
        //     // tokio::time::sleep(Duration::from_millis(10)).await;
        // }
        todo!()
    }

    pub async fn send_raw_request<'a>(&mut self) -> Result<IpmiRespPayload> {
        // send_recv(&self.socket, &mut self.buffer.0, &mut self.buffer.1)
        //     .await
        //     .unwrap();
        // let packet: Packet = Packet::from(
        //     &self.session.k2,
        //     &self.buffer.0[..self.buffer.1],
        //     &self.session.k2,
        // )
        // .try_into()
        // .unwrap();
        // let PacketPayload::IpmiResp(payload) = packet.data else {
        //     info!("send raw request");
        //     Err(EClient::NoResponse)?
        // };
        // Ok(payload)
        todo!()
    }

    pub async fn deactivated(&mut self) {
        // let cmd = CloseSessionCMD::new(self.session.managed_id);
        // let _ = self.send_packet(&mut cmd.gen_packet()).await;
    }

    // bug: inspur and dell bmc sometimes response with different request commandcode,
    // when this situation happened, we need reconnect
    pub async fn re_connect(&mut self) -> Result<()> {
        // self.deactivated().await;
        // let addr = self.socket.peer_addr().unwrap();
        // warn!("reconnect: {}", addr);
        // *self = IPMIClient::new(addr.to_string(), &self.username, &self.password)
        //     .await?
        //     .activate()
        //     .await?;
        Ok(())
    }

    // #[allow(unused)]
    // async fn set_privilege_level(&mut self) -> Result<()> {
    //     // Set session privilege level to ADMIN
    //     let pri = &self.privilege;
    //     if !matches!(pri, Privilege::Administrator) {
    //         let mut set_session_req = IpmiRawRequest::new(
    //             None,
    //             None,
    //             NetFn::App,
    //             CommandCode::SetSessionPrivilegeLevel,
    //             vec![Privilege::Administrator as u8],
    //         )
    //         .create_packet();
    //
    //         let set_session_response: Packet = (
    //             self.send_packet(&mut set_session_req).await?.as_slice(),
    //             &self.session.k2,
    //         )
    //             .try_into()
    //             .unwrap();
    //         if let PacketData::IpmiResp(IpmiRespPayload {
    //             completion_code: CompletionCode::CompletedNormally,
    //             ..
    //         }) = set_session_response.data
    //         {
    //             self.privilege = Privilege::Administrator;
    //         }
    //     }
    //     Ok(())
    // }
}

// pub async fn send_packet<'a>(
//     socket: &UdpSocket,
//     buffer: &'a mut [u8],
//     packet: Packet<'a>,
//     opt: &ClientOption,
//     session: &mut SessionActived,
// ) -> Result<Packet<'a>> {
//     let mut n = packet.write_to(buffer);
//     // request_packet.set_session_id(session.managed_id);
//     // request_packet.set_session_seq_num(session.seq_number);
//     session.seq_number = session.seq_number.wrapping_add(1);
//     encrypt_packet(session, (buffer, &mut n));
//
//     let n = send_recv(socket, buffer, n, opt.socket_recv_timeout).await?;
//     let resp_packet = Packet::decode_from(buffer, &session.k2, n)?;
//     Ok(resp_packet)
// }

pub async fn send_recv(
    socket: &UdpSocket,
    snd: &[u8],
    mut recv_buf: &mut [u8],
    timeout: Duration,
) -> Result<usize> {
    tokio::time::timeout(timeout, socket.send(&snd))
        .await?
        .map_err(EClient::FailedSend)?;

    let n = tokio::time::timeout(timeout, socket.recv_buf(&mut recv_buf))
        .await?
        .map_err(EClient::FailedSetSocketReadTimeout)?;
    Ok(n)
}

fn encrypt_packet(session: &SessionActived, data: (&mut [u8], &mut usize)) {
    aes_128_cbc_encrypt(data.0, data.1, session.k2[..16].try_into().unwrap());

    // req_packet
    //     .ipmi_session
    //     .set_payload_len(encrypted_payload.len());
    // let mut packet_vec: Vec<u8> = (&req_packet.rmcp_header).into();
    // let mut session_payload_slice: Vec<u8> = req_packet.ipmi_session.into();
    // session_payload_slice.extend(encrypted_payload);
    // add_tailer(&mut session_payload_slice, session.k1);
    // packet_vec.extend(session_payload_slice);
    // packet_vec
}
