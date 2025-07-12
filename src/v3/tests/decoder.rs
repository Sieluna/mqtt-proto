use std::ops::Deref;
use std::sync::Arc;

use bytes::Bytes;
use futures_lite::future::block_on;

use crate::v3::*;
use crate::*;
use QoS::*;

#[test]
fn test_header_firstbyte() {
    use PacketType::*;
    let valid = vec![
        (0b0001_0000, Header::new(Connect, false, Level0, false, 0)),
        (0b0010_0000, Header::new(Connack, false, Level0, false, 0)),
        (0b0011_0000, Header::new(Publish, false, Level0, false, 0)),
        (0b0011_0001, Header::new(Publish, false, Level0, true, 0)),
        (0b0011_0010, Header::new(Publish, false, Level1, false, 0)),
        (0b0011_0011, Header::new(Publish, false, Level1, true, 0)),
        (0b0011_0100, Header::new(Publish, false, Level2, false, 0)),
        (0b0011_0101, Header::new(Publish, false, Level2, true, 0)),
        (0b0011_1000, Header::new(Publish, true, Level0, false, 0)),
        (0b0011_1001, Header::new(Publish, true, Level0, true, 0)),
        (0b0011_1010, Header::new(Publish, true, Level1, false, 0)),
        (0b0011_1011, Header::new(Publish, true, Level1, true, 0)),
        (0b0011_1100, Header::new(Publish, true, Level2, false, 0)),
        (0b0011_1101, Header::new(Publish, true, Level2, true, 0)),
        (0b0100_0000, Header::new(Puback, false, Level0, false, 0)),
        (0b0101_0000, Header::new(Pubrec, false, Level0, false, 0)),
        (0b0110_0010, Header::new(Pubrel, false, Level0, false, 0)),
        (0b0111_0000, Header::new(Pubcomp, false, Level0, false, 0)),
        (0b1000_0010, Header::new(Subscribe, false, Level0, false, 0)),
        (0b1001_0000, Header::new(Suback, false, Level0, false, 0)),
        (
            0b1010_0010,
            Header::new(Unsubscribe, false, Level0, false, 0),
        ),
        (0b1011_0000, Header::new(Unsuback, false, Level0, false, 0)),
        (0b1100_0000, Header::new(Pingreq, false, Level0, false, 0)),
        (0b1101_0000, Header::new(Pingresp, false, Level0, false, 0)),
        (
            0b1110_0000,
            Header::new(Disconnect, false, Level0, false, 0),
        ),
    ];
    for n in 0..=255 {
        let res = match valid.iter().find(|(byte, _)| *byte == n) {
            Some((_, header)) => Ok(*header),
            None if ((n & 0b110) == 0b110) && (n >> 4 == 3) => Err(Error::InvalidQos(3)),
            None => Err(Error::InvalidHeader),
        };
        let buf: &[u8] = &[n, 0];
        assert_eq!(res, Header::decode(buf), "{:08b}", n);
    }
}

#[test]
fn test_header_len() {
    use PacketType::*;
    for (bytes, res) in vec![
        (
            vec![1 << 4, 0],
            Ok(Header::new(Connect, false, Level0, false, 0)),
        ),
        (
            vec![1 << 4, 127],
            Ok(Header::new(Connect, false, Level0, false, 127)),
        ),
        (
            vec![1 << 4, 0x80, 0],
            Ok(Header::new(Connect, false, Level0, false, 0)),
        ), //Weird encoding for "0" buf matches spec
        (
            vec![1 << 4, 0x80, 1],
            Ok(Header::new(Connect, false, Level0, false, 128)),
        ),
        (
            vec![1 << 4, 0x80 + 16, 78],
            Ok(Header::new(Connect, false, Level0, false, 10000)),
        ),
        (
            vec![1 << 4, 0x80, 0x80, 0x80, 0x80],
            Err(Error::InvalidVarByteInt),
        ),
    ] {
        let slice_buf = bytes.as_slice();
        assert_eq!(res, Header::decode(slice_buf));
    }
}

#[test]
fn test_non_utf8_string() {
    let mut data: &[u8] = &[
        0b00110000, 10, // type=Publish, remaining_len=10
        0x00, 0x03, b'a', b'/', 0xc0_u8, // Topic with Invalid utf8
        b'h', b'e', b'l', b'l', b'o', // payload
    ];
    assert!(matches!(
        Packet::decode(data).unwrap_err(),
        Error::InvalidString
    ));
    assert_eq!(
        Packet::decode(data).unwrap_err(),
        block_on(PollPacket::new(&mut Default::default(), &mut data)).unwrap_err()
    );
}

#[test]
fn test_inner_length_too_long() {
    let mut data: &[u8] = &[
        0b00010000, 20, // Connect packet, remaining_len=20
        0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0b01000000, // +password
        0x00, 0x0a, // keepalive 10 sec
        0x00, 0x04, b't', b'e', b's', b't', // client_id
        0x00, 0x03, b'm', b'q', // password with invalid length
    ];
    assert_eq!(Ok(None), Packet::decode(data));
    assert_eq!(
        block_on(PollPacket::new(&mut Default::default(), &mut data)).unwrap_err(),
        Error::InvalidRemainingLength
    );
}

#[test]
fn test_decode_half_connect() {
    let mut data: &[u8] = &[
        0b00010000, 39, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00,
        0x0a, // 10 sec
              // 0x00, 0x04, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, // client_id
              // 0x00, 0x02, '/' as u8, 'a' as u8, // will topic = '/a'
              // 0x00, 0x07, 'o' as u8, 'f' as u8, 'f' as u8, 'l' as u8, 'i' as u8, 'n' as u8,
              // 'e' as u8, // will msg = 'offline'
              // 0x00, 0x04, 'r' as u8, 'u' as u8, 's' as u8, 't' as u8, // username = 'rust'
              // 0x00, 0x02, 'm' as u8, 'q' as u8, // password = 'mq'
    ];
    assert_eq!(Ok(None), Packet::decode(data));
    assert_eq!(12, data.len());
    assert!(
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap_err()
            .is_eof()
    );
}

#[test]
fn test_decode_connect_wrong_version() {
    let mut data: &[u8] = &[
        0b00010000, 39, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x01,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00, 0x0a, // 10 sec
        0x00, 0x04, b't', b'e', b's', b't', // client_id
        0x00, 0x02, b'/', b'a', // will topic = '/a'
        0x00, 0x07, b'o', b'f', b'f', b'l', b'i', b'n', b'e', // will msg = 'offline'
        0x00, 0x04, b'r', b'u', b's', b't', // username = 'rust'
        0x00, 0x02, b'm', b'q', // password = 'mq'
    ];
    assert_eq!(
        Packet::decode(data),
        Err(Error::InvalidProtocol("MQTT".to_owned(), 1)),
    );
    assert_eq!(
        Packet::decode(data).unwrap_err(),
        block_on(PollPacket::new(&mut Default::default(), &mut data)).unwrap_err()
    );
}

#[test]
fn test_decode_reserved_connect_flags() {
    let mut data: &[u8] = &[
        0b00010000, 16, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04,
        0b11001111, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00, 0x0a, // 10 sec
        0x00, 0x04, b't', b'e', b's', b't', // client_id
    ];
    assert_eq!(
        Packet::decode(data),
        Err(Error::InvalidConnectFlags(0b11001111)),
    );
    assert_eq!(
        Packet::decode(data).unwrap_err(),
        block_on(PollPacket::new(&mut Default::default(), &mut data)).unwrap_err()
    );
}

#[test]
fn test_decode_packet_n() {
    let data: &[u8] = &[
        // connect packet
        0b00010000, 39, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04,
        0b11001110, // +username, +password, -will retain, will qos=1, +last_will, +clean_session
        0x00, 0x0a, // 10 sec
        0x00, 0x04, b't', b'e', b's', b't', // client_id
        0x00, 0x02, b'/', b'a', // will topic = '/a'
        0x00, 0x07, b'o', b'f', b'f', b'l', b'i', b'n', b'e', // will msg = 'offline'
        0x00, 0x04, b'r', b'u', b's', b't', // username = 'rust'
        0x00, 0x02, b'm', b'q', // password = 'mq'
        // pingreq packet
        0b11000000, 0b00000000, // pingresp packet
        0b11010000, 0b00000000,
    ];

    let pkt1 = v3::Connect {
        protocol: Protocol::V311,
        keep_alive: 10,
        client_id: Arc::new("test".to_owned()),
        clean_session: true,
        last_will: Some(LastWill {
            topic_name: TopicName::try_from("/a".to_owned()).unwrap(),
            message: Bytes::from(b"offline".to_vec()),
            qos: QoS::Level1,
            retain: false,
        }),
        username: Some(Arc::new("rust".to_owned())),
        password: Some(Bytes::from(b"mq".to_vec())),
    };

    let pkt2 = Packet::Pingreq;
    let pkt3 = Packet::Pingresp;

    // decode 3 packets in a sequence stored in the same buffer
    let mut offset = 0;
    let mut data1 = &data[offset..];
    let decode_pkt1 = Packet::decode(data1).unwrap().unwrap();
    assert_eq!(
        Packet::decode(data1).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data1))
            .unwrap()
            .2
    );

    offset += total_len(pkt1.encode_len()).unwrap();
    let mut data2 = &data[offset..];
    let decode_pkt2 = Packet::decode(data2).unwrap().unwrap();
    assert_eq!(
        Packet::decode(data2).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data2))
            .unwrap()
            .2
    );

    offset += total_len(0).unwrap();
    let mut data3 = &data[offset..];
    let decode_pkt3 = Packet::decode(data3).unwrap().unwrap();
    assert_eq!(
        Packet::decode(data3).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data3))
            .unwrap()
            .2
    );

    assert_eq!(Packet::Connect(pkt1), decode_pkt1);
    assert_eq!(pkt2, decode_pkt2);
    assert_eq!(pkt3, decode_pkt3);
}

#[test]
fn test_decode_connack() {
    let mut data: &[u8] = &[0b00100000, 2, 0b00000000, 0b00000001];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Connack(v3::Connack {
            session_present: false,
            code: ConnectReturnCode::UnacceptableProtocolVersion,
        })
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_ping_req() {
    let mut data: &[u8] = &[0b11000000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Pingreq)), Packet::decode(data));
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_ping_resp() {
    let mut data: &[u8] = &[0b11010000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Pingresp)), Packet::decode(data));
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_disconnect() {
    let mut data: &[u8] = &[0b11100000, 0b00000000];
    assert_eq!(Ok(Some(Packet::Disconnect)), Packet::decode(data));
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_publish() {
    let data: &[u8] = &[
        0b00110000, 10, 0x00, 0x03, b'a', b'/', b'b', b'h', b'e', b'l', b'l', b'o', //
        0b00111000, 10, 0x00, 0x03, b'a', b'/', b'b', b'h', b'e', b'l', b'l', b'o', //
        0b00111101, 12, 0x00, 0x03, b'a', b'/', b'b', 0, 10, b'h', b'e', b'l', b'l', b'o',
    ];

    let mut data1 = data;
    assert_eq!(
        Header::decode(data1).unwrap(),
        Header::new_with(0b00110000, 10).unwrap(),
    );
    assert_eq!(data.len(), 38);

    match Packet::decode(data1).unwrap().unwrap() {
        Packet::Publish(p) => {
            assert!(!p.dup);
            assert!(!p.retain);
            assert_eq!(p.qos_pid, QosPid::Level0);
            assert_eq!(p.topic_name.deref(), "a/b");
            assert_eq!(core::str::from_utf8(p.payload.as_ref()).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
    assert_eq!(
        Packet::decode(data1).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data1))
            .unwrap()
            .2
    );

    let mut data2 = &data[12..];
    match Packet::decode(data2).unwrap().unwrap() {
        Packet::Publish(p) => {
            assert!(p.dup);
            assert!(!p.retain);
            assert_eq!(p.qos_pid, QosPid::Level0);
            assert_eq!(p.topic_name.deref(), "a/b");
            assert_eq!(core::str::from_utf8(p.payload.as_ref()).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
    assert_eq!(
        Packet::decode(data2).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data2))
            .unwrap()
            .2
    );

    let mut data3 = &data[24..];
    match Packet::decode(data3).unwrap().unwrap() {
        Packet::Publish(p) => {
            assert!(p.dup);
            assert!(p.retain);
            assert_eq!(p.qos_pid, QosPid::Level2(Pid::try_from(10).unwrap()));
            assert_eq!(p.topic_name.deref(), "a/b");
            assert_eq!(core::str::from_utf8(p.payload.as_ref()).unwrap(), "hello");
        }
        other => panic!("Failed decode: {:?}", other),
    }
    assert_eq!(
        Packet::decode(data3).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data3))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_pub_ack() {
    let mut data: &[u8] = &[0b01000000, 0b00000010, 0, 10];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Puback(Pid::try_from(10).unwrap())
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_pub_rec() {
    let mut data: &[u8] = &[0b01010000, 0b00000010, 0, 10];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Pubrec(Pid::try_from(10).unwrap())
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_pub_rel() {
    let mut data: &[u8] = &[0b01100010, 0b00000010, 0, 10];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Pubrel(Pid::try_from(10).unwrap())
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_pub_comp() {
    let mut data: &[u8] = &[0b01110000, 0b00000010, 0, 10];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Pubcomp(Pid::try_from(10).unwrap())
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_subscribe() {
    let mut data: &[u8] = &[0b10000010, 8, 0, 10, 0, 3, b'a', b'/', b'b', 0];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Subscribe(v3::Subscribe {
            pid: Pid::try_from(10).unwrap(),
            topics: vec![(
                TopicFilter::try_from("a/b".to_owned()).unwrap(),
                QoS::Level0
            )],
        })
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_suback() {
    let mut data: &[u8] = &[0b10010000, 3, 0, 10, 0b00000010];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Suback(v3::Suback {
            pid: Pid::try_from(10).unwrap(),
            topics: vec![SubscribeReturnCode::MaxLevel2],
        })
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_unsubscribe() {
    let mut data: &[u8] = &[0b10100010, 5, 0, 10, 0, 1, b'a'];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Unsubscribe(v3::Unsubscribe {
            pid: Pid::try_from(10).unwrap(),
            topics: vec![TopicFilter::try_from("a".to_owned()).unwrap(),],
        })
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[test]
fn test_decode_unsub_ack() {
    let mut data: &[u8] = &[0b10110000, 2, 0, 10];
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        Packet::Unsuback(Pid::try_from(10).unwrap())
    );
    assert_eq!(
        Packet::decode(data).unwrap().unwrap(),
        block_on(PollPacket::new(&mut Default::default(), &mut data))
            .unwrap()
            .2
    );
}

#[tokio::test(flavor = "current_thread")]
#[cfg(feature = "dhat-heap")]
async fn poll_actor_model_simulation_v3() {
    let _profiler = dhat::Profiler::builder().testing().build();

    const NUM_TASKS: usize = 100_000;

    let data: Arc<Vec<u8>> = Arc::new(vec![
        0b00010000, 39, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0b11001110, 0x00, 0x0a, 0x00,
        0x04, b't', b'e', b's', b't', 0x00, 0x02, b'/', b'a', 0x00, 0x07, b'o', b'f', b'f', b'l',
        b'i', b'n', b'e', 0x00, 0x04, b'r', b'u', b's', b't', 0x00, 0x02, b'm', b'q',
    ]);

    println!(
        "\n--- `v3::decoder` Actor Model Simulation ({} jobs) ---",
        NUM_TASKS
    );

    let stats_start = dhat::HeapStats::get();
    println!(
        "Start:               {:>5} bytes in {:>2} blocks",
        stats_start.curr_bytes, stats_start.curr_blocks
    );

    let simulation_start = std::time::Instant::now();
    let mut handles = Vec::with_capacity(NUM_TASKS);

    for _ in 0..NUM_TASKS {
        let data = data.clone();

        handles.push(tokio::spawn(async move {
            let mut buf: &[u8] = &data;
            let _ =
                futures_lite::future::block_on(PollPacket::new(&mut Default::default(), &mut buf));
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let total_simulation_time = simulation_start.elapsed();
    let total_data_size = data.len() * NUM_TASKS;
    let throughput_mbps =
        (total_data_size as f64 * 8.0) / (total_simulation_time.as_secs_f64() * 1_000_000.0);
    let jobs_per_sec = NUM_TASKS as f64 / total_simulation_time.as_secs_f64();

    drop(data);

    let stats_end = dhat::HeapStats::get();
    println!(
        "End:                 {:>5} bytes in {:>2} blocks. Change: {:>+5} bytes, {:>+3} blocks",
        stats_end.curr_bytes,
        stats_end.curr_blocks,
        stats_end.curr_bytes as i64 - stats_start.curr_bytes as i64,
        stats_end.curr_blocks as i64 - stats_start.curr_blocks as i64
    );
    println!(
        "Peak memory usage:   {:>5} bytes in {:>2} blocks",
        stats_end.max_bytes, stats_end.max_blocks
    );

    let summary = common::MemorySummary {
        test: "v3::decoder",
        bytes: (stats_start.curr_bytes as u64, stats_end.curr_bytes as u64),
        blocks: (stats_start.curr_blocks as u64, stats_end.curr_blocks as u64),
        peak_bytes: stats_end.max_bytes as u64,
        peak_blocks: stats_end.max_blocks as u64,
        throughput_mbps,
        jobs_per_sec,
        avg_time_per_job_us: total_simulation_time.as_micros() as f64 / NUM_TASKS as f64,
    };
    println!("{}", serde_json::to_string(&summary).unwrap());

    println!("--- End Report ---");
}
