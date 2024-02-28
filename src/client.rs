use log::info;
use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::{
    err::IPMIClientError,
    rmcp::{
        crypto::{add_tailer, aes_128_cbc_encrypt, hash_hmac_sha1},
        ipmi::commands::Privilege,
        open_session::{
            AuthAlgorithm, ConfidentialityAlgorithm, IntegrityAlgorithm, RMCPPlusOpenSession,
            RMCPPlusOpenSessionRequest, StatusCode,
        },
        rakp::{RAKPMessage1, RAKPMessage3, Rakp},
        request::IpmiRawRequest,
        response::RespPayload,
        CompletionCode, Packet, Payload,
    },
    Command, GetChannelAuthCapabilitiesRequest, IpmiCommand, IpmiHeader, IpmiV2Header, NetFn,
    RmcpHeader,
};

pub type Result<T> = core::result::Result<T, IPMIClientError>;

pub enum State {
    Discovery,
    // Authentication,
    Established,
}

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
            // remote_console_random_number: rand::random::<u128>(),
            password_mac_key: Default::default(),
            sik: Default::default(),
            k1: Default::default(),
            k2: Default::default(),
        }
    }
}

pub struct SessionActived {
    seq_number: u32,
    managed_id: u32,
    k1: [u8; 20],
    k2: [u8; 20],
}

impl From<SessionInactived> for SessionActived {
    fn from(value: SessionInactived) -> Self {
        Self {
            seq_number: value.seq_number,
            managed_id: value.managed_id.unwrap(),
            k1: value.k1.unwrap(),
            k2: value.k2.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct IPMIClient<S> {
    client_socket: tokio::net::UdpSocket,
    session: S,
    privilege: Privilege,
    username: String,
    cipher_list_index: u8,
    buf: [u8; 8192],
}

impl IPMIClient<SessionInactived> {
    /// Creates client for running IPMI commands against a BMC.
    ///
    /// # Arguments
    /// * `ipmi_server_addr` - Socket address of the IPMI server (or BMC LAN controller). Default port for IPMI RMCP is 623 UDP.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_ipmi::ipmi_client::{IPMIClient, IPMIClientError};
    ///
    /// let ipmi_server = "192.168.1.10:623"
    /// let ipmi_client: Result<IPMIClient, IPMIClientError> = IPMIClient::new(ipmi_server)
    ///     .expect("Failed to connect to the IPMI server");
    /// ```
    pub async fn new<A: ToSocketAddrs>(
        ipmi_server_addr: A,
    ) -> Result<IPMIClient<SessionInactived>> {
        let client_socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(IPMIClientError::FailedBind)?;
        client_socket
            .connect(ipmi_server_addr)
            .await
            .map_err(IPMIClientError::ConnectToIPMIServer)?;
        Ok(IPMIClient {
            client_socket,
            session: SessionInactived::default(),
            username: String::new(),
            cipher_list_index: 0,
            privilege: Privilege::Callback,
            buf: [0; 8192],
        })
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
    ///     .map_err(|e: IPMIClientError| println!("Failed to establish session with BMC: {}", e));
    ///
    /// ```
    pub async fn activate(
        mut self,
        username: &str,
        password: &str,
    ) -> Result<IPMIClient<SessionActived>> {
        self.username = username.to_string();
        self.session.password_mac_key = Some(password.into());

        self.get_channel_auth_cap().await?; // Get channel auth capabilites and set cipher
        self.rmcpplus_session_authenticate().await // rmcp open session and authenticate
                                                   // c.set_privilege_level()?;
    }

    // fn handle_completion_code(&mut self, payload: &RespPayload) -> Result<()> {
    //     if !payload.completion_code.is_success() {
    //         return Ok(());
    //     }
    //     match payload.command {
    //         Command::GetChannelAuthCapabilities => {
    //             let response: GetChannelAuthCapabilitiesResponse =
    //                 GetChannelAuthCapabilitiesResponse::try_from(payload.data.as_slice())
    //                     .map_err(PacketError::IPMIPayload)?;
    //             // Currently don't support IPMI v1.5
    //             if let AuthVersion::IpmiV1_5 = response.auth_version {
    //                 return Err(IPMIClientError::UnsupportedVersion);
    //             }
    //             self.session.channel_auth_capabilities = Some(response);
    //         }
    //         _ => return Ok(()),
    //     }
    //     Ok(())
    // }

    async fn get_channel_auth_cap(&mut self) -> Result<()> {
        let get_channel_auth_cap_packet =
            GetChannelAuthCapabilitiesRequest::new(true, 0xE, Privilege::Administrator)
                .create_packet();
        self.send_unauth_packet(get_channel_auth_cap_packet).await?;

        Ok(())
    }

    fn gen_rk2_auth_code(&self) -> [u8; 20] {
        let mut buf: Vec<u8> = vec![0; 58 + self.username.len()];
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
        buf[57] = self.username.len().try_into().unwrap();
        buf[58..].copy_from_slice(self.username.as_bytes());

        hash_hmac_sha1(self.session.password_mac_key.as_ref().unwrap(), &buf)
    }

    fn gen_rk3_auth_code(&self) -> [u8; 20] {
        let mut buf = Vec::new();
        append_u128_to_vec(&mut buf, self.session.managed_system_random_number.unwrap());
        append_u32_to_vec(&mut buf, self.session.remote_console_session_id.unwrap());

        buf.push(Self::privilege(false, Privilege::Administrator));
        buf.push(self.username.len().try_into().unwrap());
        self.username
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

    async fn rmcpplus_session_authenticate(mut self) -> Result<IPMIClient<SessionActived>> {
        // RMCP+ Open Session Request
        let rmcp_plus_open_session_packet: Packet = RMCPPlusOpenSessionRequest::new(
            0,
            Privilege::Administrator,
            0xa0a2a3a4,
            AuthAlgorithm::RakpHmacSha1,
            IntegrityAlgorithm::HmacSha196,
            ConfidentialityAlgorithm::AesCbc128,
        )
        .into();
        let osr = self
            .send_unauth_packet(rmcp_plus_open_session_packet)
            .await?;
        let Payload::Rmcp(RMCPPlusOpenSession::Response(msg)) = osr.payload else {
            Err(IPMIClientError::MisformedResponse)?
        };

        let StatusCode::NoErrors = &msg.rmcp_plus_status_code else {
            Err(IPMIClientError::FailedToOpenSession(
                msg.rmcp_plus_status_code,
            ))?
        };
        self.session.managed_id = Some(msg.managed_system_session_id);

        // RAKP Message 1
        let rk1 = RAKPMessage1::new(
            0x0,
            self.session.managed_id.unwrap(),
            self.session.remote_console_random_number,
            false,
            Privilege::Administrator,
            self.username.clone(),
        );
        let rakp1_packet: Packet = rk1.into();

        // RAKP Message 2
        let r2_pkt = self.send_unauth_packet(rakp1_packet).await?;
        let Payload::Rakp(Rakp::Message2(msg2)) = &r2_pkt.payload else {
            Err(IPMIClientError::MisformedResponse)?
        };

        let StatusCode::NoErrors = &msg2.status_code else {
            Err(IPMIClientError::FailedToOpenSession(msg2.status_code))?
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
                Err(IPMIClientError::FailedToValidateRAKP2)?
            }
        }

        self.create_session_keys()?;

        // RAKP Message 3
        let rakp3_packet: Packet = RAKPMessage3::new(
            0x0,
            StatusCode::NoErrors,
            self.session.managed_id.unwrap(),
            Some(self.gen_rk3_auth_code().into()),
        )
        .into();

        // RAKP Message 4
        let r4_pkt = self.send_unauth_packet(rakp3_packet).await?;

        let Payload::Rakp(Rakp::Message4(msg4)) = r4_pkt.payload else {
            Err(IPMIClientError::MisformedResponse)?
        };

        let StatusCode::NoErrors = msg4.status_code else {
            Err(IPMIClientError::FailedToOpenSession(msg4.status_code))?
        };
        if let Some(auth_code) = msg4.integrity_auth_code {
            if auth_code != self.gen_rk4_auth_code() {
                Err(IPMIClientError::MismatchedKeyExchangeAuthCode)?
            }
            self.session.state = State::Established;
        }

        Ok(IPMIClient::from(self))
    }

    async fn send_unauth_packet(&mut self, request_packet: Packet) -> Result<Packet> {
        let x: Vec<u8> = request_packet.into();
        self.client_socket
            .send(&x)
            .await
            .map_err(IPMIClientError::FailedSend)?;
        let mut buf = [0_u8; 1024];
        let Ok((n_bytes, _)) = self.client_socket.recv_from(&mut buf).await else {
            info!("send_unauth_packet");
            Err(IPMIClientError::NoResponse)?
        };

        match self.session.k2 {
            Some(k2) => Ok(Packet::try_from((&buf[..n_bytes], &k2))?),
            None => Ok(Packet::try_from(&buf[..n_bytes])?),
        }
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
            self.username
                .len()
                .try_into()
                .map_err(IPMIClientError::UsernameOver255InLength)?,
        );
        self.username
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

impl From<IPMIClient<SessionInactived>> for IPMIClient<SessionActived> {
    fn from(inactive: IPMIClient<SessionInactived>) -> Self {
        Self {
            client_socket: inactive.client_socket,
            session: inactive.session.into(),
            privilege: inactive.privilege,
            username: inactive.username,
            cipher_list_index: inactive.cipher_list_index,
            buf: inactive.buf,
        }
    }
}

impl IPMIClient<SessionActived> {
    pub async fn send_packet(&mut self, mut request_packet: Packet) -> Result<Vec<u8>> {
        request_packet.set_session_id(self.session.managed_id);
        request_packet.set_session_seq_num(self.session.seq_number);

        let payload_bytes: Vec<u8> = request_packet.payload.into();
        let encrypted_payload =
            aes_128_cbc_encrypt(payload_bytes, self.session.k2[..16].try_into().unwrap());

        request_packet
            .ipmi_header
            .set_payload_len(encrypted_payload.len());
        let mut packet_vec: Vec<u8> = (&request_packet.rmcp_header).into();
        let mut session_payload_vec: Vec<u8> = request_packet.ipmi_header.into();
        session_payload_vec.extend(encrypted_payload);
        add_tailer(&mut session_payload_vec, self.session.k1);
        packet_vec.extend(session_payload_vec);
        self.client_socket
            .send(&packet_vec)
            .await
            .map_err(IPMIClientError::FailedSend)?;
        self.session.seq_number += 1;

        let mut buf = [0; 1024];

        let n_bytes = match self.client_socket.recv_from(&mut buf).await {
            Ok((nbytes, _)) => nbytes,
            Err(e) => {
                info!("send packet: {}", e);
                Err(IPMIClientError::NoResponse)?
            }
        };

        Ok(buf[..n_bytes].to_vec())
    }

    pub async fn send_ipmi_cmd<CMD: IpmiCommand>(&mut self, ipmi_cmd: CMD) -> Result<CMD::Output> {
        let payload = ipmi_cmd.payload();
        let packet = Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_est(32)),
            payload,
        );
        let resp: Packet = (self.send_packet(packet).await?.as_slice(), &self.session.k2)
            .try_into()
            .unwrap();

        let (data, code) = resp.payload.data_and_completion();

        if !matches!(code, CompletionCode::CompletedNormally) {
            Err(IPMIClientError::ParseResponse(code))
        } else {
            Ok(<CMD>::parse(data).unwrap())
        }
    }

    pub async fn send_raw_request(&mut self, data: &[u8]) -> Result<RespPayload> {
        let send_data = if data.len() > 2 {
            data[2..].to_vec()
        } else {
            Vec::new()
        };

        let raw_request: Packet = IpmiRawRequest::new(data[0], data[1], send_data).create_packet();
        let response = self.send_packet(raw_request).await?;
        let packet: Packet = (response.as_slice(), &self.session.k2).try_into().unwrap();
        let Payload::IpmiResp(payload) = packet.payload else {
            info!("send raw request");
            Err(IPMIClientError::NoResponse)?
        };
        Ok(payload)
    }

    #[allow(unused)]
    async fn set_privilege_level(&mut self) -> Result<()> {
        // Set session privilege level to ADMIN
        let pri = &self.privilege;
        if !matches!(pri, Privilege::Administrator) {
            let set_session_req = IpmiRawRequest::new(
                NetFn::App,
                Command::SetSessionPrivilegeLevel,
                vec![Privilege::Administrator as u8],
            )
            .create_packet();

            let set_session_response: Packet = (
                self.send_packet(set_session_req).await?.as_slice(),
                &self.session.k2,
            )
                .try_into()
                .unwrap();
            if let Payload::IpmiResp(RespPayload {
                completion_code: CompletionCode::CompletedNormally,
                ..
            }) = set_session_response.payload
            {
                self.privilege = Privilege::Administrator;
            }
        }
        Ok(())
    }
}
