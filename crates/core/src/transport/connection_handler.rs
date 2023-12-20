use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::vec::Vec;
use std::{borrow::Cow, time::Duration};

use aes_gcm::{Aes128Gcm, KeyInit};
use futures::channel::oneshot;
use futures::SinkExt;
use serde::Serialize;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::task;

use crate::transport::{
    packet_data::{MAX_DATA_SIZE, MAX_PACKET_SIZE},
    symmetric_message::{SymmetricMessage, SymmetricMessagePayload},
    ReceiverStream, SenderStream,
};

use super::SenderStreamError;
use super::{
    connection_info::ConnectionInfo,
    crypto::{TransportKeypair, TransportPublicKey},
    BytesPerSecond, PacketData,
};

pub(super) type ConnectionHandlerMessage = (SocketAddr, Vec<u8>);

const PROTOC_VERSION: [u8; 2] = 1u16.to_le_bytes();

// Constants for exponential backoff
const INITIAL_TIMEOUT: Duration = Duration::from_secs(5);
const TIMEOUT_MULTIPLIER: u64 = 2;
const MAX_TIMEOUT: Duration = Duration::from_secs(60); // Maximum timeout limit

// Constants for interval increase
const INITIAL_INTERVAL: Duration = Duration::from_millis(200);
const INTERVAL_INCREASE_FACTOR: u64 = 2;
const MAX_INTERVAL: Duration = Duration::from_millis(5000); // Maximum interval limit

pub(super) type SerializedMessage = Vec<u8>;

pub struct PeerConnection {
    inbound_recv: mpsc::Receiver<PacketData>,
    outbound_sender: mpsc::Sender<SerializedMessage>,
}

impl PeerConnection {
    pub async fn recv(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let _packet_data = self
            .inbound_recv
            .recv()
            .await
            .ok_or(ConnectionError::ChannelClosed);
        todo!()
    }

    pub async fn send<T: Serialize>(&mut self, data: &T) -> Result<(), ConnectionError> {
        let serialized_data = bincode::serialize(data).unwrap();
        if serialized_data.len() > MAX_DATA_SIZE {
            let mut sender = SenderStream::new(&mut self.outbound_sender);
            sender.start_send_unpin(serialized_data)?;
            Ok(())
        } else {
            self.outbound_sender.send(serialized_data).await.unwrap();
            todo!()
        }
    }
}

pub(crate) struct ConnectionHandler {
    max_upstream_rate: Arc<arc_swap::ArcSwap<BytesPerSecond>>,
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

        let max_upstream_rate = Arc::new(arc_swap::ArcSwap::from_pointee(max_upstream_rate));
        let transport = UdpPacketsListener {
            connection_raw_packet_senders: HashMap::new(),
            socket,
            send_queue: send_queue_receiver,
            this_peer_keypair: keypair,
            max_upstream_rate: max_upstream_rate.clone(),
            is_gateway,
        };
        let connection_handler = ConnectionHandler {
            send_queue,
            max_upstream_rate,
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
            let (open_connection, recv_connection) = oneshot::channel();
            self.send_queue
                .send((
                    remote_addr,
                    ConnectionEvent::ConnectionStart {
                        remote_public_key,
                        open_connection,
                    },
                ))
                .await?;
            let (outbound_sender, inbound_recv) =
                recv_connection.await.map_err(|e| anyhow::anyhow!(e))??;
            Ok(PeerConnection {
                inbound_recv,
                outbound_sender,
            })
        } else {
            todo!("establish connection with a gateway")
        }
    }

    fn update_max_upstream_rate(&mut self, max_upstream_rate: BytesPerSecond) {
        self.max_upstream_rate.store(Arc::new(max_upstream_rate));
    }
}

/// Handles UDP transport internally.
struct UdpPacketsListener {
    socket: UdpSocket,
    connection_raw_packet_senders: HashMap<SocketAddr, (ConnectionInfo, mpsc::Sender<PacketData>)>,
    send_queue: mpsc::Receiver<(SocketAddr, ConnectionEvent)>,
    this_peer_keypair: TransportKeypair,
    max_upstream_rate: Arc<arc_swap::ArcSwap<BytesPerSecond>>,
    is_gateway: bool,
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
                            match self.connection_raw_packet_senders.get_mut(&addr) {
                                Some((conn_info, sender)) => {
                                    // todo: check if is streamed or a single packet
                                    // then handle both cases

                                    // let packet_data = PacketData::decrypt(std::mem::replace(&mut buf, [0; MAX_PACKET_SIZE]),  &mut conn_info.outbound_symmetric_key).unwrap();
                                    // if let Err(e) = sender.send(packet_data).await {
                                    //     tracing::warn!("Failed to send raw packet to connection sender: {:?}", e);
                                    // }
                                }
                                None => {
                                    self
                                        .handle_unrecogized_remote(addr, &buf[..size]);
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
                            ConnectionEvent::ConnectionStart { remote_public_key, open_connection  }  => {
                                match self.traverse_nat(remote_addr, remote_public_key).await {
                                    Err(error) => {
                                        tracing::error!(%error, ?remote_addr, "Failed to establish connection");
                                    }
                                    Ok(connection_info) => {
                                        let (outbound_sender, outbound_receiver) = mpsc::channel(1);
                                        let (inbound_sender, inbound_recv) = mpsc::channel(1);
                                        self.connection_raw_packet_senders.insert(remote_addr, (connection_info, inbound_sender));
                                        let _ = open_connection.send(Ok((outbound_sender, inbound_recv)));
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
            AckConnection,
        }
        // Initialize timeout and interval
        let mut timeout = INITIAL_TIMEOUT;
        let mut interval_duration = INITIAL_INTERVAL;
        let mut tick = tokio::time::interval(interval_duration);

        const MAX_FAILURES: usize = 20;
        let mut failures = 0;
        let mut packet = [0u8; MAX_PACKET_SIZE];
        let mut state = ConnectionState::Start;

        let inbound_sym_key_bytes = rand::random::<[u8; 16]>();
        let inbound_sym_key = Aes128Gcm::new(&inbound_sym_key_bytes.into());
        let mut outbound_sym_key: Option<Aes128Gcm> = None;

        let outbound_intro_packet = {
            let mut data = [0u8; { 16 + PROTOC_VERSION.len() }];
            data[..PROTOC_VERSION.len()].copy_from_slice(&PROTOC_VERSION);
            data[PROTOC_VERSION.len()..].copy_from_slice(&inbound_sym_key_bytes);
            PacketData::<MAX_PACKET_SIZE>::encrypted_with_remote(&data, &remote_public_key)
        };

        while failures < MAX_FAILURES {
            match state {
                ConnectionState::Start => {
                    tracing::debug!("Sending protocol version to remote");
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
                ConnectionState::AckConnection => {
                    self.socket
                        .send_to(
                            SymmetricMessage::ack_ok(outbound_sym_key.as_mut().unwrap())?
                                .send_data(),
                            remote_addr,
                        )
                        .await?;
                }
            }
            match tokio::time::timeout(timeout, self.socket.recv_from(&mut packet)).await {
                Ok(Ok((size, response_remote))) => {
                    if response_remote != remote_addr {
                        todo!("is a different remote, handle this message");
                    }
                    match state {
                        ConnectionState::Start => {
                            // try to decrypt the message with the symmetric key
                            let Ok(data) = self.this_peer_keypair.secret.decrypt(&packet[..size])
                            else {
                                failures += 1;
                                tracing::debug!("Received unexpect packet from remote");
                                continue;
                            };
                            let key = Aes128Gcm::new_from_slice(&data[PROTOC_VERSION.len()..])
                                .map_err(|_| TransportError::ConnectionEstablishmentFailure {
                                    cause: "invalid symmetric key".into(),
                                })?;
                            let protocol_version = &data[..PROTOC_VERSION.len()];
                            if protocol_version != PROTOC_VERSION {
                                let packet = {
                                    let msg = SymmetricMessage {
                                        message_id: 0,
                                        confirm_receipt: None,
                                        payload: SymmetricMessagePayload::AckConnection {
                                            result: Err(
                                                "remote is using a different protocol version"
                                                    .into(),
                                            ),
                                        },
                                    };
                                    let mut packet = [0u8; MAX_PACKET_SIZE];
                                    bincode::serialize_into(packet.as_mut_slice(), &msg)?;
                                    PacketData::<MAX_PACKET_SIZE>::encrypted_with_cipher(
                                        &packet[..],
                                        &key,
                                    )
                                };
                                let _ = self.socket.send_to(packet.send_data(), remote_addr).await;
                                return Err(TransportError::ConnectionEstablishmentFailure {
                                    cause: format!(
                                        "remote is using a different protocol version: {:?}",
                                        String::from_utf8_lossy(protocol_version)
                                    )
                                    .into(),
                                });
                            }
                            outbound_sym_key = Some(key);
                            state = ConnectionState::AckConnection;
                            continue;
                        }

                        ConnectionState::AckConnection => {
                            let packet: PacketData<MAX_PACKET_SIZE> =
                                (std::mem::replace(&mut packet, [0; MAX_PACKET_SIZE]), size).into();
                            let decrypted = packet.decrypt(&inbound_sym_key).unwrap();
                            let packet =
                                bincode::deserialize::<SymmetricMessage>(decrypted.send_data())?;
                            if let SymmetricMessagePayload::AckConnection { result: Ok(_) } =
                                packet.payload
                            {
                                return Ok(ConnectionInfo {
                                    outbound_symmetric_key: outbound_sym_key
                                        .expect("should be set at this stage"),
                                    inbound_symmetric_key: inbound_sym_key,
                                    remote_public_key,
                                    remote_is_gateway: false,
                                    remote_addr,
                                });
                            }
                            tracing::debug!("Received unrecognized message from remote");
                            return Err(TransportError::ConnectionEstablishmentFailure {
                                cause: "received unrecognized message from remote".into(),
                            });
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
            // Update timeout using exponential backoff, capped at MAX_TIMEOUT
            timeout = std::cmp::min(
                Duration::from_secs(timeout.as_secs() * TIMEOUT_MULTIPLIER),
                MAX_TIMEOUT,
            );

            // Update interval, capped at MAX_INTERVAL
            if interval_duration < MAX_INTERVAL {
                interval_duration = std::cmp::min(
                    Duration::from_millis(
                        interval_duration.as_millis() as u64 * INTERVAL_INCREASE_FACTOR,
                    ),
                    MAX_INTERVAL,
                );
                tick = tokio::time::interval(interval_duration);
            }

            tick.tick().await;
        }
        Err(TransportError::ConnectionEstablishmentFailure {
            cause: "max connection attempts reached".into(),
        })
    }

    fn handle_unrecogized_remote(&self, _socket: SocketAddr, _packet: &[u8]) {
        // todo: try to decrypt with the rsa keypair to see if is an other peer initiating a connection
        // fail otherwise
        // then jump to nat_traversal, passing the apporopiuate state and parameters so we skip
        // unnecessary steps
        // logic for gateway should be slightly different cause we don't need to do nat traversal
        todo!()
    }
}

type PeerChannel = (mpsc::Sender<SerializedMessage>, mpsc::Receiver<PacketData>);

pub(super) enum ConnectionEvent {
    ConnectionStart {
        remote_public_key: TransportPublicKey,
        open_connection: oneshot::Sender<Result<PeerChannel, TransportError>>,
    },
    SendRawPacket(PacketData),
}

// Define a custom error type for the transport layer
#[derive(Debug, thiserror::Error)]
pub(super) enum TransportError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("transport handler channel closed")]
    ChannelClosed(#[from] mpsc::error::SendError<(SocketAddr, ConnectionEvent)>),
    #[error("failed while establishing connection, reason: {cause}")]
    ConnectionEstablishmentFailure { cause: Cow<'static, str> },
    #[error(transparent)]
    DescryptionError(#[from] rsa::errors::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Serialization(#[from] bincode::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConnectionError {
    #[error(transparent)]
    StreamingError(#[from] SenderStreamError),
    #[error("Connection closed")]
    ChannelClosed,
}
