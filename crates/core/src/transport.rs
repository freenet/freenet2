#![allow(dead_code)] // TODO: Remove before integration
//! Freenet Transport protocol implementation.
//!
//! # Transport
//!
//! The transport layer is responsible for reliably sending and receiving messages
//! over the network.
//!
//! ## Message Streaming
//!
//! The transport layer supports two types of messages:
//!
//! - Short messages that can fit in a single UDP packet.
//! - Streamed messages that are split into multiple UDP packets.
//!
//! The purpose of streamed messages is to allow a node to start sending a message before
//! it has been received completely (although it must know the message size before starting
//! to send).
//!
//! ## Congestion Control
//!
//! The transport layer implements a simple congestion control algorithm which assumes
//! that congestion won't occur if the upstream rate is less than `max_upstream_rate`.
//! Choosing an appropriate and conservative value for `max_upstream_rate` is therefore
//! important to avoid congestion.
//!
//! ## Encryption
//!
//! Each peer chooses a symmetric key that is used to encrypt *inbound* messages for that peer,
//! the exception is inbound connections to the gateway peer which will use the key provided
//! by the peer initiating the connection in both directions.
//!
//! Each peer initiates a connection by encrypting its chosen key with the public key of the
//! peer it is connecting to. The encrypted key is then sent to the peer in the first message
//! of the connection, repeated until a correctly encrypted response is received. The peer
//! receiving the message will decrypt the key and use it to encrypt future messages.
//!
//! ## Opening a Connection
//!
//! ### Neither peer is a gateway
//!
//! 1. Peer A sends a `ConnectionStart` message to Peer B with its chosen symmetric key
//!    encrypted with Peer B's public key, resending every 200ms until...
//! 2. Peer B receives the message and decrypts the symmetric key, it then sends a `ConnectionAck`
//!    message to Peer A encrypted with the symmetric key.
//! 3. Peer B stores the `ConnectionStart` and `ConnectionAck` messages in [UdpConnection] and
//!    if its sees that message again it resends the `ConnectionAck` message.

mod bw;
mod connection_handler;
mod connection_info;
mod crypto;
mod symmetric_message;

use aes_gcm::aead::rand_core::SeedableRng;
use aes_gcm::{
    aead::{generic_array::GenericArray, AeadMut},
    aes::cipher::Unsigned,
    AeadCore, AeadInPlace, Aes128Gcm,
};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng};
use std::cell::RefCell;

use self::{
    connection_handler::MAX_PACKET_SIZE, connection_info::ConnectionError,
    crypto::TransportPublicKey,
};

struct ReceiverStream {}

impl ReceiverStream {
    /// Will await until a full message is received, does error handling, reassembling the message from parts, decryption, etc.
    async fn receive_message(&self) -> Result<Vec<u8>, ConnectionError> {
        todo!()
    }

    async fn read_part(&self) -> Result<StreamedMessagePart, ConnectionError> {
        todo!()
    }
}

struct StreamedMessagePart {
    data: PacketData,
    part_start_position: usize,
    message_size: usize,
}

// todo: split this into type for handling inbound (encrypted)/outbound (decrypted) packets for clarity
struct PacketData<const N: usize = MAX_PACKET_SIZE> {
    data: [u8; N],
    size: usize,
}

// This must be very fast, but doesn't need to be cryptographically secure.
thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(
        SmallRng::from_rng(thread_rng()).expect("failed to create RNG")
    );
}

// These are the same as the AES-GCM 128 constants, but extracting them from Aes128Gcm
// as consts was awkward.
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

impl<const N: usize> PacketData<N> {
    fn encrypted_with_cipher(data: &[u8], cipher: &mut Aes128Gcm) -> Self {
        debug_assert!(data.len() <= MAX_PACKET_SIZE - NONCE_SIZE - TAG_SIZE);

        let nonce: [u8; NONCE_SIZE] = RNG.with(|rng| rng.borrow_mut().gen());

        let mut buffer = [0u8; N];
        buffer[..NONCE_SIZE].copy_from_slice(&nonce);

        // Encrypt the data in place
        let payload_length = data.len();
        buffer[NONCE_SIZE..NONCE_SIZE + payload_length].copy_from_slice(data);
        let tag = cipher
            .encrypt_in_place_detached(
                &nonce.into(),
                &[],
                &mut buffer[NONCE_SIZE..NONCE_SIZE + payload_length],
            )
            .unwrap();

        // Append the tag to the buffer
        buffer[NONCE_SIZE + payload_length..NONCE_SIZE + payload_length + TAG_SIZE]
            .copy_from_slice(tag.as_slice());

        Self {
            data: buffer,
            size: NONCE_SIZE + payload_length + TAG_SIZE,
        }
    }

    fn encrypted_with_remote(data: &[u8], remote_key: &TransportPublicKey) -> Self {
        let encrypted_data: Vec<u8> = remote_key.encrypt(data);
        debug_assert!(encrypted_data.len() <= MAX_PACKET_SIZE);
        let mut data = [0; N];
        data.copy_from_slice(&encrypted_data[..]);
        Self {
            data,
            size: encrypted_data.len(),
        }
    }

    fn send_data(&self) -> &[u8] {
        &self.data[..self.size]
    }

    fn decrypt(&self, inbound_sym_key: &mut Aes128Gcm) -> Result<Self, aes_gcm::Error> {
        let tag_size = <Aes128Gcm as AeadCore>::TagSize::to_usize();

        debug_assert!(self.data.len() >= NONCE_SIZE + tag_size);
        debug_assert!(self.data.len() <= MAX_PACKET_SIZE + NONCE_SIZE + tag_size);

        let nonce = GenericArray::from_slice(&self.data[..NONCE_SIZE]);
        // Adjusted to extract the tag from the end of the encrypted data
        let tag = GenericArray::from_slice(&self.data[self.size - TAG_SIZE..self.size]);
        let encrypted_data = &self.data[NONCE_SIZE..self.size - TAG_SIZE];
        let mut buffer = [0u8; N];
        let buffer_len = encrypted_data.len();
        buffer[..buffer_len].copy_from_slice(encrypted_data);

        inbound_sym_key.decrypt_in_place_detached(nonce, &[], &mut buffer[..buffer_len], tag)?;

        Ok(Self {
            data: buffer,
            size: buffer_len,
        })
    }
}

struct SenderStream {}

impl SenderStream {
    /// Will await until the message is sent, handles breaking the message into parts, encryption, etc.
    async fn send_message(&self, _data: &[u8]) -> Result<(), SenderStreamError> {
        todo!()
    }
}

struct BytesPerSecond(f64);

impl BytesPerSecond {
    pub fn new(bytes_per_second: f64) -> Self {
        assert!(bytes_per_second >= 0.0);
        Self(bytes_per_second)
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum SenderStreamError {
    #[error("stream closed unexpectedly")]
    Closed,
    #[error("message too big, size: {size}, max size: {max_size}")]
    MessageExceedsLength { size: usize, max_size: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::{Aes128Gcm, KeyInit};
    use rand::rngs::OsRng;

    #[test]
    fn test_encryption_decryption() {
        // Generate a random 128-bit (16 bytes) key
        let mut key = [0u8; 16];
        OsRng.fill_bytes(&mut key);

        // Create a key object for AES-GCM
        let key = GenericArray::from_slice(&key);

        // Create a new AES-128-GCM instance
        let mut cipher = Aes128Gcm::new(key);
        let original_data = b"Hello, world!";
        let packet_data: PacketData<MAX_PACKET_SIZE> =
            PacketData::encrypted_with_cipher(original_data, &mut cipher);

        // Ensure data is not plainly visible
        assert_ne!(packet_data.data[..packet_data.size], *original_data);

        // Test decryption
        match packet_data.decrypt(&mut cipher) {
            Ok(decrypted_data) => {
                // Ensure decrypted data matches original
                assert_eq!(&decrypted_data.data[..decrypted_data.size], original_data);
            }
            Err(e) => panic!("Decryption failed with error: {:?}", e),
        }
    }

    // Additional tests can be added here, such as testing with different data sizes,
    // ensuring that encryption/decryption fails with incorrect keys, etc.
}
