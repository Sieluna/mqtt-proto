use std::pin::Pin;

use futures_lite::future::{block_on, poll_fn};
use futures_lite::Future;
use tokio_test::io::Builder;

use crate::common::poll::{GenericPollPacket, GenericPollPacketState, PollHeader};
use crate::common::utils::read_u16;
use crate::{Error, FromTokio};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockHeader {
    remaining_len: u32,
    packet_type: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum MockPacket {
    Connect {
        protocol_name: String,
        protocol_version: u8,
    },
    Publish {
        topic: String,
        mock_pid: u16,
        payload: Vec<u8>,
    },
    Other,
}

fn read_string(reader: &mut &[u8]) -> String {
    if reader.len() < 2 {
        return String::new();
    }
    let len_bytes: [u8; 2] = reader[0..2].try_into().unwrap();
    let len = u16::from_be_bytes(len_bytes);
    *reader = &reader[2..];
    if reader.len() < len as usize {
        return String::new();
    }
    let s = String::from_utf8_lossy(&reader[..len as usize]).to_string();
    *reader = &reader[len as usize..];
    s
}

impl PollHeader for MockHeader {
    type Error = Error;
    type Packet = MockPacket;

    fn new_with(hd: u8, remaining_len: u32) -> Result<Self, Self::Error> {
        Ok(MockHeader {
            remaining_len,
            packet_type: hd,
        })
    }

    fn build_empty_packet(&self) -> Option<Self::Packet> {
        if self.remaining_len == 0 {
            Some(MockPacket::Other)
        } else {
            None
        }
    }

    fn block_decode(self, reader: &mut &[u8]) -> Result<Self::Packet, Self::Error> {
        let packet = match self.packet_type & 0xF0 {
            0x10 => {
                let protocol_name = read_string(reader);
                if reader.is_empty() {
                    return Err(Error::InvalidVarByteInt);
                }
                let protocol_version = reader[0];
                *reader = &reader[1..];
                MockPacket::Connect {
                    protocol_name,
                    protocol_version,
                }
            }
            0x30 => {
                let topic = read_string(reader);
                let mock_pid = block_on(read_u16(reader))?;
                let payload = reader.to_vec();
                *reader = &[];
                MockPacket::Publish {
                    topic,
                    mock_pid,
                    payload,
                }
            }
            _ => MockPacket::Other,
        };
        Ok(packet)
    }

    fn remaining_len(&self) -> usize {
        self.remaining_len as usize
    }

    fn is_eof_error(err: &Self::Error) -> bool {
        err.is_eof()
    }
}

#[tokio::test]
#[cfg(feature = "dhat-heap")]
async fn poll_is_efficient_in_allocation() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // --- 1. Prepare Data & Mock IO ---
    const PAYLOAD_SIZE: usize = 1024;
    const MOCK_PID: u16 = 42;
    let control_byte = 0x30; // Publish packet type

    let topic = "a/b/c";
    let mut topic_buf: Vec<u8> = Vec::new();
    topic_buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    topic_buf.extend_from_slice(topic.as_bytes());

    // Create a 1KB payload with varied data
    let payload: Vec<u8> = (0..PAYLOAD_SIZE as u32).map(|i| (i % 256) as u8).collect();
    let mut body = topic_buf;
    body.extend_from_slice(&MOCK_PID.to_be_bytes());
    body.extend_from_slice(&payload);
    let body_len = body.len();

    // MQTT's remaining_len is a variable-length integer. We need to encode it correctly.
    let mut remaining_len_buf = Vec::new();
    crate::common::write_var_int(&mut remaining_len_buf, body_len).unwrap();

    // Simulate fragmented network reads with a larger payload
    let reader = Builder::new()
        .read(&[control_byte])
        .read(&remaining_len_buf)
        .read(&body[0..256])
        .read(&body[256..512])
        .read(&body[512..768])
        .read(&body[768..1024])
        .read(&body[1024..body_len])
        .build();
    let mut reader = FromTokio::new(reader);

    let mut state = GenericPollPacketState::<MockHeader>::default();
    let mut poll_packet = GenericPollPacket::new(&mut state, &mut reader);

    println!("\n--- `common::poll` Memory Report (1KB Payload) ---");
    let stats_start = dhat::HeapStats::get();
    println!(
        "Start:                  {:>5} bytes in {:>2} blocks",
        stats_start.curr_bytes, stats_start.curr_blocks
    );

    // --- 2. Poll the future until completion ---
    let result = poll_fn(|cx| {
        Pin::new(&mut poll_packet).poll(cx)
    }).await;
    assert!(result.is_ok());

    let stats_decoded = dhat::HeapStats::get();
    println!(
        "Poll & Decode (net):    {:>5} bytes in {:>2} blocks. Change: {:>+5} bytes, {:>+3} blocks",
        stats_decoded.curr_bytes,
        stats_decoded.curr_blocks,
        stats_decoded.curr_bytes as i64 - stats_start.curr_bytes as i64,
        stats_decoded.curr_blocks as i64 - stats_start.curr_blocks as i64
    );

    // --- 3. Teardown ---
    let (_total_len, buf, packet) = result.unwrap();
    let expected_packet = MockPacket::Publish {
        topic: topic.to_string(),
        mock_pid: MOCK_PID,
        payload: payload.clone(),
    };
    assert_eq!(packet, expected_packet);

    drop(buf);
    let stats_dropped_buf = dhat::HeapStats::get();
    println!(
        "Drop buffer:            {:>5} bytes in {:>2} blocks. Change: {:>+5} bytes, {:>+3} blocks",
        stats_dropped_buf.curr_bytes,
        stats_dropped_buf.curr_blocks,
        stats_dropped_buf.curr_bytes as i64 - stats_decoded.curr_bytes as i64,
        stats_dropped_buf.curr_blocks as i64 - stats_decoded.curr_blocks as i64
    );

    drop(packet);
    let stats_dropped_packet = dhat::HeapStats::get();
    println!(
        "Drop packet:            {:>5} bytes in {:>2} blocks. Change: {:>+5} bytes, {:>+3} blocks",
        stats_dropped_packet.curr_bytes,
        stats_dropped_packet.curr_blocks,
        stats_dropped_packet.curr_bytes as i64 - stats_dropped_buf.curr_bytes as i64,
        stats_dropped_packet.curr_blocks as i64 - stats_dropped_buf.curr_blocks as i64
    );

    println!("--- End Report ---\n");
}
