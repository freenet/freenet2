use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::vec;

use aes_gcm::Aes128Gcm;
use tokio::sync::mpsc;

use crate::{
    transport::{
        packet_data,
        sent_packet_tracker::SentPacketTracker,
        symmetric_message::{self},
        TransportError,
    },
    util::time_source::InstantTimeSrc,
};

use super::StreamId;

pub(crate) type SerializedStream = Vec<u8>;

// TODO: measure the space overhead of SymmetricMessage::LongMessage since is likely less than 100
/// The max payload we can send in a single fragment, this MUST be less than packet_data::MAX_DATA_SIZE
/// since we need to account for the space overhead of SymmetricMessage::LongMessage metadata
const MAX_DATA_SIZE: usize = packet_data::MAX_DATA_SIZE - 100;

// TODO: unit test
/// Handles sending a stream that is *not piped*. In the future this will be replaced by
/// piped streams which start forwarding before the stream has been received.
#[allow(clippy::too_many_arguments)]
pub(super) async fn send_stream(
    stream_id: StreamId,
    last_packet_id: Arc<AtomicU32>,
    sender: mpsc::Sender<(SocketAddr, Arc<[u8]>)>,
    destination_addr: SocketAddr,
    mut stream_to_send: SerializedStream,
    outbound_symmetric_key: Aes128Gcm,
    sent_packet_tracker: Arc<parking_lot::Mutex<SentPacketTracker<InstantTimeSrc>>>,
) -> Result<(), TransportError> {
    let total_length_bytes = stream_to_send.len() as u32;
    let mut total_packets = stream_to_send.len() / MAX_DATA_SIZE;
    total_packets += if stream_to_send.len() % MAX_DATA_SIZE == 0 {
        0
    } else {
        1
    };
    let mut sent_so_far = 0;
    let mut next_fragment_number = 1; // Fragment numbers are 1-indexed

    loop {
        if sent_so_far == total_packets {
            break;
        }
        let mut rest = {
            if stream_to_send.len() > MAX_DATA_SIZE {
                stream_to_send.split_off(MAX_DATA_SIZE)
            } else {
                std::mem::take(&mut stream_to_send)
            }
        };
        std::mem::swap(&mut stream_to_send, &mut rest);
        next_fragment_number += 1;
        let packet_id = last_packet_id.fetch_add(1, std::sync::atomic::Ordering::Release);
        super::packet_sending(
            destination_addr,
            &sender,
            packet_id,
            &outbound_symmetric_key,
            vec![],
            symmetric_message::StreamFragment {
                stream_id,
                total_length_bytes: total_length_bytes as u64,
                fragment_number: next_fragment_number,
                payload: rest,
            },
            &sent_packet_tracker,
        )
        .await?;
        sent_so_far += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::packet_data::PacketData;
    use aes_gcm::KeyInit;
    use std::net::{Ipv4Addr, SocketAddr};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_send_stream_success() {
        let (outbound_sender, mut outbound_receiver) = mpsc::channel(100);
        let remote_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
        let message = vec![1, 2, 3, 4, 5];
        let cipher = {
            let key = rand::random::<[u8; 16]>();
            Aes128Gcm::new(&key.into())
        };
        let sent_tracker = Arc::new(parking_lot::Mutex::new(SentPacketTracker::new()));

        let background_task = tokio::spawn(send_stream(
            StreamId::next(),
            Arc::new(AtomicU32::new(0)),
            outbound_sender,
            remote_addr,
            message.clone(),
            cipher.clone(),
            sent_tracker,
        ));

        let mut inbound_bytes = Vec::new();
        while let Some((_, packet)) = outbound_receiver.recv().await {
            let packet_data: PacketData = packet.as_ref().into();
            let decrypted_packet = packet_data.decrypt(&cipher).unwrap();
            inbound_bytes.extend_from_slice(decrypted_packet.data());
        }

        let result = background_task.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(message, inbound_bytes);
    }

    // Add more tests here for other scenarios
}
