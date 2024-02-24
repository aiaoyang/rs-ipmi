use core::time;
use std::{
    collections::HashMap,
    net::{ToSocketAddrs, UdpSocket},
    time::Duration,
};

use crate::{
    err::{IPMIClientError, PacketError},
    rmcp::{
        crypto::hash_hmac_sha_256,
        ipmi::commands::{
            AuthVersion, Command, GetChannelAuthCapabilitiesRequest,
            GetChannelAuthCapabilitiesResponse, GetChannelCipherSuitesRequest,
            GetChannelCipherSuitesResponse, Privilege,
        },
        open_session::{
            AuthAlgorithm, ConfidentialityAlgorithm, IntegrityAlgorithm, RMCPPlusOpenSession,
            RMCPPlusOpenSessionRequest, StatusCode,
        },
        rakp::{RAKPMessage1, RAKPMessage2, RAKPMessage3, Rakp},
        request::IpmiRawRequest,
        response::RespPayload,
        CompletionCode, Packet, Payload, PayloadType,
    },
    IpmiCommand, IpmiHeader, IpmiV2Header, NetFn, RmcpHeader,
};

pub type Result<T> = core::result::Result<T, IPMIClientError>;

pub enum State {
    Discovery,
    Authentication,
    Established,
}

pub struct SessionInactived {
    state: State,
    seq_number: u32,
    auth_algorithm: Option<AuthAlgorithm>,
    integrity_algorithm: Option<IntegrityAlgorithm>,
    confidentiality_algorithm: Option<ConfidentialityAlgorithm>,
    managed_id: Option<u32>,
    managed_system_random_number: Option<u128>,
    managed_system_guid: Option<u128>,
    remote_console_session_id: Option<u32>,
    remote_console_random_number: u128,

    password_mac_key: Option<Vec<u8>>,
    channel_auth_capabilities: Option<GetChannelAuthCapabilitiesResponse>,
    cipher_suite_bytes: Option<Vec<u8>>,

    sik: Option<[u8; 32]>,
    k1: Option<[u8; 32]>,
    k2: Option<[u8; 32]>,
}

impl Default for SessionInactived {
    fn default() -> Self {
        Self {
            state: State::Discovery,
            seq_number: 0x0000_0001,
            auth_algorithm: Default::default(),
            integrity_algorithm: Default::default(),
            confidentiality_algorithm: Default::default(),
            managed_id: Default::default(),
            managed_system_random_number: Default::default(),
            managed_system_guid: Default::default(),
            remote_console_session_id: Default::default(),
            remote_console_random_number: rand::random::<u128>(),
            password_mac_key: Default::default(),
            channel_auth_capabilities: Default::default(),
            cipher_suite_bytes: Default::default(),
            sik: Default::default(),
            k1: Default::default(),
            k2: Default::default(),
        }
    }
}

pub struct SessionActived {
    seq_number: u32,
    managed_id: u32,
    k1: [u8; 32],
    k2: [u8; 32],
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
    client_socket: UdpSocket,
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
    pub fn new<A: ToSocketAddrs>(ipmi_server_addr: A) -> Result<IPMIClient<SessionInactived>> {
        let client_socket = UdpSocket::bind("0.0.0.0:0").map_err(IPMIClientError::FailedBind)?;
        let _ = client_socket
            .set_read_timeout(Some(time::Duration::from_secs(20)))
            .map_err(IPMIClientError::FailedSetSocketReadTimeout);
        client_socket
            .connect(ipmi_server_addr)
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
    pub fn set_read_timeout(&self, duration: Option<Duration>) -> Result<()> {
        self.client_socket
            .set_read_timeout(duration)
            .map_err(IPMIClientError::SetReadTimeOutError)
    }

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
    pub fn activate<S: ToString>(
        mut self,
        username: S,
        password: S,
    ) -> Result<IPMIClient<SessionActived>> {
        self.username = username.to_string();
        let binding = password.to_string();
        let rakp2_mac_key = binding.as_bytes();
        self.session.password_mac_key = Some(rakp2_mac_key.into());

        self.get_channel_auth_cap()?; // Get channel auth capabilites and set cipher
        let mut c = self.rmcpplus_session_authenticate()?; // rmcp open session and authenticate
        c.set_privilege_level()?;
        Ok(c)
    }

    fn handle_completion_code(&mut self, payload: &RespPayload) -> Result<()> {
        if !payload.completion_code.is_success() {
            return Ok(());
        }
        match payload.command {
            Command::GetChannelAuthCapabilities => {
                let response: GetChannelAuthCapabilitiesResponse =
                    GetChannelAuthCapabilitiesResponse::try_from(payload.data.as_slice())
                        .map_err(PacketError::IPMIPayload)?;
                // Currently don't support IPMI v1.5
                if let AuthVersion::IpmiV1_5 = response.auth_version {
                    return Err(IPMIClientError::UnsupportedVersion);
                }
                self.session.channel_auth_capabilities = Some(response);
            }
            Command::GetChannelCipherSuites => {
                while let State::Discovery = self.session.state {
                    self.cipher_list_index += 1;
                    self.handle_cipher_suites(payload.clone(), self.cipher_list_index)?;
                }
            }
            _ => return Ok(()),
        }
        Ok(())
    }

    fn get_channel_auth_cap(&mut self) -> Result<()> {
        let get_channel_auth_cap_packet =
            GetChannelAuthCapabilitiesRequest::new(true, 0xE, Privilege::Administrator)
                .create_packet();
        self.send_unauth_packet(get_channel_auth_cap_packet)?;

        // Get the Channel Cipher Suites
        let get_channel_cipher_suites_packet =
            GetChannelCipherSuitesRequest::default().create_packet();
        self.send_unauth_packet(get_channel_cipher_suites_packet)?;
        Ok(())
    }

    fn rmcpplus_session_authenticate(mut self) -> Result<IPMIClient<SessionActived>> {
        // RMCP+ Open Session Request
        let rmcp_open_packet: Packet = RMCPPlusOpenSessionRequest::new(
            0,
            Privilege::Administrator,
            0xa0a2a3a4,
            self.session.auth_algorithm.clone().unwrap(),
            self.session.integrity_algorithm.clone().unwrap(),
            self.session.confidentiality_algorithm.clone().unwrap(),
        )
        .into();
        self.send_unauth_packet(rmcp_open_packet)?;

        // RAKP Message 1
        let rakp1_packet: Packet = RAKPMessage1::new(
            0x0,
            self.session.managed_id.unwrap(),
            self.session.remote_console_random_number,
            true,
            Privilege::Administrator,
            self.username.clone(),
        )
        .into();
        self.send_unauth_packet(rakp1_packet)?;
        self.create_session_keys()?;

        // RAKP Message 3
        let mut rakp3_input_buffer = Vec::new();
        append_u128_to_vec(
            &mut rakp3_input_buffer,
            self.session.managed_system_random_number.unwrap(),
        );
        append_u32_to_vec(
            &mut rakp3_input_buffer,
            self.session.remote_console_session_id.unwrap(),
        );
        rakp3_input_buffer.push(0x14);
        rakp3_input_buffer.push(self.username.len().try_into().unwrap());
        self.username
            .as_bytes()
            .iter()
            .for_each(|char| rakp3_input_buffer.push(*char));

        let rakp3_auth_code = hash_hmac_sha_256(
            self.session.password_mac_key.clone().unwrap(),
            rakp3_input_buffer,
        );
        let rakp3_packet: Packet = RAKPMessage3::new(
            0x0,
            StatusCode::NoErrors,
            self.session.managed_id.unwrap(),
            Some(rakp3_auth_code.into()),
        )
        .into();
        self.send_unauth_packet(rakp3_packet)?;

        Ok(IPMIClient::from(self))
    }

    fn send_unauth_packet(&mut self, request_packet: Packet) -> Result<Packet> {
        let x: Vec<u8> = request_packet.into();
        self.client_socket
            .send(&x)
            .map_err(IPMIClientError::FailedSend)?;

        let Ok((n_bytes, _)) = self.client_socket.recv_from(&mut self.buf) else {
            Err(IPMIClientError::NoResponse)?
        };

        let response_packet: Packet = match self.session.k2 {
            Some(k2) => Packet::try_from((&self.buf[..n_bytes], &k2))?,
            None => Packet::try_from(&self.buf[..n_bytes])?,
        };

        match &response_packet.payload {
            Payload::IpmiResp(payload) => self.handle_completion_code(payload)?,
            Payload::Rmcp(_) | Payload::Rakp(Rakp::Message2(_) | Rakp::Message4(_)) => {
                self.handle_status_code(&response_packet.payload)?
            }
            Payload::None => {}
            _ => Err(IPMIClientError::MisformedResponse)?,
        }
        Ok(response_packet)
    }

    fn handle_status_code(&mut self, payload: &Payload) -> Result<()> {
        match payload {
            Payload::Rmcp(RMCPPlusOpenSession::Response(response)) => {
                match &response.rmcp_plus_status_code {
                    StatusCode::NoErrors => {
                        self.session.managed_id = Some(response.managed_system_session_id);
                    }
                    _ => Err(IPMIClientError::FailedToOpenSession(
                        response.rmcp_plus_status_code,
                    ))?,
                }
            }
            Payload::Rakp(rakp) => match rakp {
                Rakp::Message2(response) => {
                    match &response.rmcp_plus_status_code {
                        StatusCode::NoErrors => {
                            self.session.managed_system_guid = Some(response.managed_system_guid);
                            self.session.remote_console_session_id =
                                Some(response.remote_console_session_id);
                            self.session.managed_system_random_number =
                                Some(response.managed_system_random_number);
                            // validate BMC auth code
                            self.validate_rakp2(response)?;
                        }
                        _ => Err(IPMIClientError::FailedToOpenSession(
                            response.rmcp_plus_status_code,
                        ))?,
                    }
                }
                Rakp::Message4(response) => match response.rmcp_plus_status_code {
                    StatusCode::NoErrors => {
                        let mut rakp4_input_buffer: Vec<u8> = Vec::new();
                        append_u128_to_vec(
                            &mut rakp4_input_buffer,
                            self.session.remote_console_random_number,
                        );
                        append_u32_to_vec(
                            &mut rakp4_input_buffer,
                            self.session.managed_id.unwrap(),
                        );
                        append_u128_to_vec(
                            &mut rakp4_input_buffer,
                            self.session.managed_system_guid.unwrap(),
                        );
                        let auth_code =
                            hash_hmac_sha_256(self.session.sik.unwrap().into(), rakp4_input_buffer);

                        match response.integrity_check_value.as_ref().unwrap()[..]
                            == auth_code[..16]
                        {
                            true => {
                                self.session.state = State::Established;
                            }
                            false => Err(IPMIClientError::MismatchedKeyExchangeAuthCode)?,
                        }
                    }
                    _ => Err(IPMIClientError::FailedToOpenSession(
                        response.rmcp_plus_status_code,
                    ))?,
                },
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_cipher_suites(&mut self, payload: RespPayload, cipher_list_index: u8) -> Result<()> {
        let response: GetChannelCipherSuitesResponse =
            payload.data.try_into().map_err(PacketError::IPMIPayload)?;
        // update total cipher bytes for the ipmi client object
        if let Some(mut old_bytes) = self.session.cipher_suite_bytes.clone() {
            response
                .cypher_suite_record_data_bytes
                .iter()
                .for_each(|byte| old_bytes.push(*byte));
            self.session.cipher_suite_bytes = Some(old_bytes);
        } else {
            self.session.cipher_suite_bytes = Some(response.cypher_suite_record_data_bytes.clone());
        }

        match response.is_last() {
            false => {
                let cipher_packet = GetChannelCipherSuitesRequest::new(
                    0xe,
                    PayloadType::Ipmi,
                    true,
                    cipher_list_index,
                )
                .create_packet();
                self.send_unauth_packet(cipher_packet)?;
                Ok(())
            }
            true => {
                // parse through cipher suite records
                self.choose_ciphers();

                // set new state - beginning authentication
                self.session.state = State::Authentication;
                Ok(())
            }
        }
    }

    fn choose_ciphers(&mut self) {
        let mut priority_map: HashMap<
            u8,
            (
                u8,
                (AuthAlgorithm, IntegrityAlgorithm, ConfidentialityAlgorithm),
            ),
        > = HashMap::new();
        if let Some(bytes) = self.session.cipher_suite_bytes.clone() {
            bytes.split(|x| *x == 0xc0).for_each(|bytes| {
                if bytes.len() != 4 {
                    return;
                }
                let auth_value: (u8, AuthAlgorithm) = match bytes[1] {
                    0x01 => (2, AuthAlgorithm::RakpHmacSha1),
                    0x02 => (1, AuthAlgorithm::RakpHmacMd5),
                    0x03 => (3, AuthAlgorithm::RakpHmacSha256),
                    _ => (0, AuthAlgorithm::RakpNone),
                };
                let integ_value: (u8, IntegrityAlgorithm) = match bytes[2] {
                    0x41 => (2, IntegrityAlgorithm::HmacSha196),
                    0x42 => (3, IntegrityAlgorithm::HmacMd5128),
                    0x43 => (1, IntegrityAlgorithm::Md5128),
                    0x44 => (4, IntegrityAlgorithm::HmacSha256128),
                    _ => (0, IntegrityAlgorithm::None),
                };
                let confid_value: (u8, ConfidentialityAlgorithm) = match bytes[3] {
                    0x81 => (3, ConfidentialityAlgorithm::AesCbc128),
                    0x82 => (2, ConfidentialityAlgorithm::XRc4128),
                    0x83 => (1, ConfidentialityAlgorithm::XRc440),
                    _ => (0, ConfidentialityAlgorithm::None),
                };
                priority_map.insert(
                    bytes[0],
                    (
                        auth_value.0 + integ_value.0 + confid_value.0,
                        (auth_value.1, integ_value.1, confid_value.1),
                    ),
                );
            });
            let id_to_use = priority_map.iter().max_by_key(|entry| entry.1 .0).unwrap();
            self.session.auth_algorithm = Some(id_to_use.1 .1 .0.clone());
            self.session.integrity_algorithm = Some(id_to_use.1 .1 .1.clone());
            self.session.confidentiality_algorithm = Some(id_to_use.1 .1 .2.clone());
        } else {
            self.session.auth_algorithm = Some(AuthAlgorithm::RakpNone);
            self.session.integrity_algorithm = Some(IntegrityAlgorithm::None);
            self.session.confidentiality_algorithm = Some(ConfidentialityAlgorithm::None);
        }
    }

    fn validate_rakp2(&self, response: &RAKPMessage2) -> Result<()> {
        if response.key_exchange_auth_code.is_none() {
            return Ok(());
        }
        let mut rakp2_input_buffer: Vec<u8> = Vec::new();
        append_u32_to_vec(
            &mut rakp2_input_buffer,
            self.session.remote_console_session_id.unwrap(),
        );
        append_u32_to_vec(&mut rakp2_input_buffer, self.session.managed_id.unwrap());
        append_u128_to_vec(
            &mut rakp2_input_buffer,
            self.session.remote_console_random_number,
        );
        append_u128_to_vec(
            &mut rakp2_input_buffer,
            self.session.managed_system_random_number.unwrap(),
        );
        append_u128_to_vec(
            &mut rakp2_input_buffer,
            self.session.managed_system_guid.unwrap(),
        );
        rakp2_input_buffer.push(0x14);
        rakp2_input_buffer.push(
            self.username
                .len()
                .try_into()
                .map_err(IPMIClientError::UsernameOver255InLength)?,
        );
        self.username
            .chars()
            .for_each(|char| rakp2_input_buffer.push(char as u8));

        let manual_auth_code = hash_hmac_sha_256(
            self.session.password_mac_key.clone().unwrap(),
            rakp2_input_buffer,
        );
        let mut vec_auth_code = Vec::new();
        vec_auth_code.extend_from_slice(manual_auth_code.as_slice());
        if &vec_auth_code != response.key_exchange_auth_code.as_ref().unwrap() {
            Err(IPMIClientError::FailedToValidateRAKP2)?
        }
        Ok(())
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

        self.session.sik = Some(hash_hmac_sha_256(
            self.session.password_mac_key.clone().unwrap(),
            sik_input,
        ));
        self.session.k1 = Some(hash_hmac_sha_256(
            self.session.sik.unwrap().into(),
            [1; 20].into(),
        ));
        self.session.k2 = Some(hash_hmac_sha_256(
            self.session.sik.unwrap().into(),
            [2; 20].into(),
        ));

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
    pub fn send_packet(&mut self, mut request_packet: Packet) -> Result<Packet> {
        request_packet.set_session_id(self.session.managed_id);
        request_packet.set_session_seq_num(self.session.seq_number);
        self.client_socket
            .send(
                &request_packet
                    .to_encrypted_bytes(&self.session.k1, &self.session.k2)
                    .unwrap(),
            )
            .map_err(IPMIClientError::FailedSend)?;
        self.session.seq_number += 1;

        let Ok((n_bytes, _)) = self.client_socket.recv_from(&mut self.buf) else {
            Err(IPMIClientError::NoResponse)?
        };

        let response_packet: Packet = Packet::try_from((&self.buf[..n_bytes], &self.session.k2))?;

        Ok(response_packet)
    }

    pub fn send_ipmi_cmd<CMD: IpmiCommand>(&mut self, ipmi_cmd: CMD) -> Result<CMD::Output> {
        let payload = ipmi_cmd.payload();
        let packet = Packet::new(
            RmcpHeader::default(),
            IpmiHeader::V2_0(IpmiV2Header::new_est(32)),
            payload,
        );
        let resp = self.send_packet(packet)?;
        Ok(<CMD>::parse(resp.payload.data()).unwrap())
    }

    pub fn send_raw_request(&mut self, data: &[u8]) -> Result<RespPayload> {
        let send_data = if data.len() > 2 {
            data[2..].to_vec()
        } else {
            Vec::new()
        };

        let raw_request: Packet = IpmiRawRequest::new(data[0], data[1], send_data).create_packet();

        let response: Packet = self.send_packet(raw_request)?;
        let Payload::IpmiResp(payload) = response.payload else {
            Err(IPMIClientError::NoResponse)?
        };
        Ok(payload)
    }

    fn set_privilege_level(&mut self) -> Result<()> {
        // Set session privilege level to ADMIN
        let pri = &self.privilege;
        if !matches!(pri, Privilege::Administrator) {
            let set_session_req = IpmiRawRequest::new(
                NetFn::App,
                Command::SetSessionPrivilegeLevel,
                vec![Privilege::Administrator as u8],
            )
            .create_packet();

            let set_session_response: Packet = self.send_packet(set_session_req)?;
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
