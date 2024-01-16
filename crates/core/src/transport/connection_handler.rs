use super::*;
use crate::node::PeerId;
use aes_gcm::{aes::Aes128, KeyInit};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::vec::Vec;
use std::{borrow::Cow, time::Duration};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::task;

use super::{
    connection_info::ConnectionInfo,
    crypto::{TransportKeypair, TransportPublicKey},
};

/// The maximum size of a received UDP packet, MTU typically is 1500
pub(super) const MAX_PACKET_SIZE: usize = 1500;

pub(super) type ConnectionHandlerMessage = (SocketAddr, Vec<u8>);

const PROTOC_VERSION: [u8; 2] = 1u16.to_le_bytes();

pub struct PeerConnection(mpsc::Receiver<PacketData>);

impl PeerConnection {
    pub async fn recv(&self) -> Result<Vec<u8>, ConnectionError> {
        todo!()
    }

    pub async fn send(&self, _message: Vec<u8>) -> Result<(), ConnectionError> {
        todo!()
    }
}

pub(crate) struct ConnectionHandler {
    connection_info: HashMap<PeerId, ConnectionInfo>,
    listen_port: u16,
    is_gateway: bool,
    max_upstream_rate: BytesPerSecond,
    send_queue: mpsc::Sender<(SocketAddr, ConnectionEvent)>,
}

impl ConnectionHandler {
    pub async fn new(
        keypair: TransportKeypair,
        listen_port: u16,
        is_gateway: bool,
        max_upstream_rate: BytesPerSecond,
    ) -> Result<Self, TransportError> {
        // Bind the UDP socket to the specified port
        let socket = UdpSocket::bind(("0.0.0.0", listen_port)).await?;

        // Channel buffer is one so senders will await until the receiver is ready, important for bandwidth limiting
        let (send_queue, send_queue_receiver) = mpsc::channel(1);

        let transport = UdpPacketsListener {
            connection_raw_packet_senders: HashMap::new(),
            socket,
            send_queue: send_queue_receiver,
            this_peer_keypair: keypair,
        };
        let connection_handler = ConnectionHandler {
            connection_info: HashMap::new(),
            listen_port,
            is_gateway,
            max_upstream_rate,
            send_queue,
        };

        task::spawn(transport.listen());

        Ok(connection_handler)
    }

    pub async fn connect(
        &mut self,
        remote_public_key: TransportPublicKey,
        remote_addr: SocketAddr,
        remote_is_gateway: bool,
    ) -> Result<PeerConnection, TransportError> {
        if !remote_is_gateway {
            self.send_queue
                .send((
                    remote_addr,
                    ConnectionEvent::ConnectionStart { remote_public_key },
                ))
                .await?;
            todo!("wait for response from the udp listener and return a `PeerConnection` instance")
        } else {
            todo!("establish connection with a gateway")
        }
    }

    fn update_max_upstream_rate(&mut self, max_upstream_rate: BytesPerSecond) {
        self.max_upstream_rate = max_upstream_rate;
    }

    fn handle_unrecognized_message(&self, (_socket, packet): (SocketAddr, PacketData)) {
        if !self.is_gateway {
            tracing::warn!(
                packet = ?packet.send_data(),
                "Received unrecognized message, ignoring because not a gateway",
            );
            return;
        }
        // use self.keypair to decrypt the message, which should contain a symmetric key
        todo!()
    }
}

/// Handles UDP transport internally.
struct UdpPacketsListener {
    socket: UdpSocket,
    connection_raw_packet_senders: HashMap<SocketAddr, (ConnectionInfo, mpsc::Sender<PacketData>)>,
    send_queue: mpsc::Receiver<(SocketAddr, ConnectionEvent)>,
    this_peer_keypair: TransportKeypair,
}

impl UdpPacketsListener {
    async fn listen(mut self) {
        loop {
            let mut buf = [0u8; MAX_PACKET_SIZE];
            tokio::select! {
                // Handling of inbound packets
                recv_result = self.socket.recv_from(&mut buf) => {
                    match recv_result {
                        Ok((size, addr)) => {
                            match self.connection_raw_packet_senders.get(&addr) {
                                Some((conn_info, sender)) => {
                                    let packet_data = PacketData::from_encrypted(std::mem::replace(&mut buf, [0; MAX_PACKET_SIZE]), size, &conn_info.outbound_symmetric_key);
                                    if let Err(e) = sender.send(packet_data).await {
                                        tracing::warn!("Failed to send raw packet to connection sender: {:?}", e);
                                    }
                                }
                                None => {
                                    self
                                        .handle_unrecognized_remote(addr);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to receive UDP packet: {:?}", e);
                        }
                    }
                },
                // Handling of outbound packets
                send_message = self.send_queue.recv() => {
                    if let Some((remote_addr, event)) = send_message {
                        match event {
                            ConnectionEvent::SendRawPacket(data) => {
                                if let Err(e) = self.socket.send_to(data.send_data(), remote_addr).await {
                                    tracing::warn!("Failed to send UDP packet: {:?}", e);
                                }
                            }
                            ConnectionEvent::ConnectionStart { remote_public_key  }  => {
                                match self.traverse_nat(remote_addr, remote_public_key).await {
                                    Err(error) => {
                                        tracing::error!(%error, ?remote_addr, "Failed to establish connection");
                                    }
                                    Ok(connection_info) => {
                                        let (peer_message_sender, peer_message_receiver) = mpsc::channel(1);
                                        self.connection_raw_packet_senders.insert(remote_addr, (connection_info, peer_message_sender));
                                        todo!()
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }

    async fn traverse_nat(
        &mut self,
        remote_addr: SocketAddr,
        remote_public_key: TransportPublicKey,
    ) -> Result<ConnectionInfo, TransportError> {
        enum ConnectionState {
            Start,
            SendIntroPacket,
            AckProtoc,
        }
        // todo: probably should use exponential backoff with an upper limit: `timeout`
        let timeout = Duration::from_secs(5);

        // todo: probably instead of a fixed interval we should monotonically increase the interval
        // until we reach a maximum, and then just keep trying at that maximum interval
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(200));
        const MAX_FAILURES: usize = 20;
        let mut failures = 0;
        let mut packet = [0u8; MAX_PACKET_SIZE];
        let mut state = ConnectionState::Start;

        let outbound_sym_key_bytes = rand::random::<[u8; 16]>();
        let outbound_sym_key: Aes128 =
            Aes128::new_from_slice(&outbound_sym_key_bytes).expect("valid length");
        let mut inbound_sym_key: Option<Aes128> = None;

        let outbound_intro_packet =
            PacketData::encrypted_with_remote(&outbound_sym_key_bytes, &remote_public_key);

        const HELLO: &[u8; 5] = b"hello";
        let hello_packet = {
            let mut packet = [0; MAX_PACKET_SIZE];
            packet[..HELLO.len()].copy_from_slice(HELLO);
            PacketData::encrypted_with_cipher(packet, HELLO.len(), &outbound_sym_key)
        };

        while failures < MAX_FAILURES {
            match state {
                ConnectionState::Start => {
                    tracing::debug!("Sending protocol version to remote");
                    if let Err(error) = self.socket.send_to(&PROTOC_VERSION, remote_addr).await {
                        failures += 1;
                        if failures == MAX_FAILURES {
                            return Err(error.into());
                        }
                        tick.tick().await;
                        continue;
                    }
                }
                ConnectionState::SendIntroPacket => {
                    tracing::debug!("Sending intro packet to remote");
                    if let Err(error) = self
                        .socket
                        .send_to(outbound_intro_packet.send_data(), remote_addr)
                        .await
                    {
                        failures += 1;
                        if failures == MAX_FAILURES {
                            return Err(error.into());
                        }
                        tick.tick().await;
                        continue;
                    }
                }
                ConnectionState::AckProtoc => {
                    self.socket
                        .send_to(hello_packet.send_data(), remote_addr)
                        .await?;
                }
            }
            match tokio::time::timeout(timeout, self.socket.recv_from(&mut packet)).await {
                Ok(Ok((size, response_remote))) => {
                    if response_remote != remote_addr {
                        todo!("is a different remote, handle this message");
                    }
                    match state {
                        ConnectionState::Start if size == PROTOC_VERSION.len() => {
                            // is a response with the protocol version from the remote,
                            // connection established, next wait for the encrypted private key
                            match &packet[..size] {
                                version if version == PROTOC_VERSION.as_slice() => {}
                                other_version => {
                                    // todo: handle this case in the future when we have more than one protocol version
                                    return Err(TransportError::ConnectionEstablishmentFailure {
                                        cause: format!(
                                            "remote is using a different protocol version: {:?}",
                                            String::from_utf8_lossy(other_version)
                                        )
                                        .into(),
                                    });
                                }
                            }
                            state = ConnectionState::SendIntroPacket;
                            continue;
                        }
                        ConnectionState::Start => {
                            failures += 1;
                            tracing::debug!("Received unexpect response from remote");
                        }
                        ConnectionState::SendIntroPacket if inbound_sym_key.is_none() => {
                            // try to decrypt the message with the symmetric key
                            let key_bytes =
                                self.this_peer_keypair.secret.decrypt(&packet[..size])?;
                            let key: Aes128 = Aes128::new_from_slice(&key_bytes).map_err(|_| {
                                TransportError::ConnectionEstablishmentFailure {
                                    cause: "invalid symmetric key".into(),
                                }
                            })?;
                            inbound_sym_key = Some(key);
                            state = ConnectionState::AckProtoc;
                        }
                        ConnectionState::AckProtoc => {
                            PacketData::decrypt(
                                &mut packet[..size],
                                inbound_sym_key
                                    .as_ref()
                                    .expect("should be set at this stage"),
                            );
                            if &packet[..size] == HELLO.as_slice() {
                                return Ok(ConnectionInfo {
                                    outbound_symmetric_key: outbound_sym_key,
                                    inbound_symmetric_key: inbound_sym_key
                                        .expect("should be set at this stage"),
                                    remote_public_key,
                                    remote_is_gateway: false,
                                    remote_addr,
                                });
                            } else {
                                tracing::debug!("Received unrecognized message from remote");
                                return Err(TransportError::ConnectionEstablishmentFailure {
                                    cause: "received unrecognized message from remote".into(),
                                });
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                Ok(Err(io_error)) => {
                    failures += 1;
                    tracing::debug!(%io_error, "Failed to receive UDP response");
                }
                Err(_) => {
                    failures += 1;
                    tracing::debug!("Failed to receive UDP response, time out");
                }
            }
            tick.tick().await;
        }
        Err(TransportError::ConnectionEstablishmentFailure {
            cause: "max connection attemps reached".into(),
        })
    }

    fn handle_unrecognized_remote(&mut self, _remote: SocketAddr) {
        tracing::warn!("Received unrecognized remote, ignoring");
    }
}

pub(super) enum ConnectionEvent {
    SendRawPacket(PacketData),
    ConnectionStart {
        remote_public_key: TransportPublicKey,
    },
}

// Define a custom error type for the transport layer
#[derive(Debug, thiserror::Error)]
pub(super) enum TransportError {
    #[error("missing peer: {0}")]
    MissingPeer(PeerId),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("transport handler channel closed")]
    ChannelClosed(#[from] mpsc::error::SendError<(SocketAddr, ConnectionEvent)>),
    #[error("failed while establishing connection, reason: {cause}")]
    ConnectionEstablishmentFailure { cause: Cow<'static, str> },
    #[error(transparent)]
    DescryptionError(#[from] rsa::errors::Error),
}
