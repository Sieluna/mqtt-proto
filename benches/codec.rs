use std::convert::TryFrom;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mqtt_proto::{
    v3::{Connect as ConnectV3, LastWill as LastWillV3, Packet as PacketV3, Publish as PublishV3},
    v5::{Connect as ConnectV5, LastWill as LastWillV5, Packet as PacketV5, Publish as PublishV5},
    Pid, QoS, QosPid, TopicName,
};

fn payload(len: usize) -> Vec<u8> {
    // Use 64-bit XorShift* as PRNG
    let mut s: usize = len.wrapping_add(0x9e37_79b9_7f4a_7c15); // MAGIC
    let mut out = Vec::with_capacity(len);

    for _ in 0..len {
        // 32-bit -> 64-bit XorShift*
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        let val = s.wrapping_mul(0x2545_f491_4f6c_dd1d) as u32;
        out.push((val >> 24) as u8);
    }
    out
}

fn create_v3_connect(payload_size: usize) -> PacketV3 {
    let mut conn = ConnectV3::new("bench-client".to_owned().into(), 60);
    conn.username = Some("user".to_owned().into());
    conn.password = Some("pass".to_owned().into());
    conn.last_will = Some(LastWillV3 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("will/topic".to_owned()).unwrap(),
        message: payload(payload_size).into(),
    });
    PacketV3::Connect(conn)
}

fn create_v5_connect(payload_size: usize) -> PacketV5 {
    let mut conn = ConnectV5::new("bench-client-v5".to_owned().into(), 60);
    conn.username = Some("user-v5".to_owned().into());
    conn.password = Some("pass-v5".to_owned().into());
    conn.last_will = Some(LastWillV5 {
        qos: QoS::Level2,
        retain: true,
        topic_name: TopicName::try_from("will/topic/v5".to_owned()).unwrap(),
        payload: payload(payload_size).into(),
        properties: Default::default(),
    });
    PacketV5::Connect(conn)
}

fn create_v3_publish(payload_size: usize) -> PacketV3 {
    PacketV3::Publish(PublishV3 {
        dup: false,
        retain: false,
        qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
        topic_name: TopicName::try_from("benchmark/topic".to_owned()).unwrap(),
        payload: payload(payload_size).into(),
    })
}

fn create_v5_publish(payload_size: usize) -> PacketV5 {
    PacketV5::Publish(PublishV5 {
        dup: false,
        retain: false,
        qos_pid: QosPid::Level2(Pid::try_from(2).unwrap()),
        topic_name: TopicName::try_from("benchmark/topic/v5".to_owned()).unwrap(),
        payload: payload(payload_size).into(),
        properties: Default::default(),
    })
}

fn bench_all(c: &mut Criterion) {
    let sizes = [64, 4 * 1024, 8 * 1024, 32 * 1024];

    for &size in &sizes {
        let v3_conn = create_v3_connect(size);
        let v5_conn = create_v5_connect(size);
        let v3_pub = create_v3_publish(size);
        let v5_pub = create_v5_publish(size);

        let v3_conn_bytes = v3_conn.encode().unwrap();
        let v5_conn_bytes = v5_conn.encode().unwrap();
        let v3_pub_bytes = v3_pub.encode().unwrap();
        let v5_pub_bytes = v5_pub.encode().unwrap();

        let mut group = c.benchmark_group(format!("size_{}_bytes", size));

        group.bench_with_input(
            BenchmarkId::new("v3_connect_encode", size),
            &v3_conn,
            |b, p| b.iter(|| black_box(p.encode())),
        );
        group.bench_with_input(
            BenchmarkId::new("v5_connect_encode", size),
            &v5_conn,
            |b, p| b.iter(|| black_box(p.encode())),
        );
        group.bench_with_input(
            BenchmarkId::new("v3_publish_encode", size),
            &v3_pub,
            |b, p| b.iter(|| black_box(p.encode())),
        );
        group.bench_with_input(
            BenchmarkId::new("v5_publish_encode", size),
            &v5_pub,
            |b, p| b.iter(|| black_box(p.encode())),
        );

        group.bench_with_input(
            BenchmarkId::new("v3_connect_decode", size),
            &v3_conn_bytes,
            |b, bytes| b.iter(|| black_box(PacketV3::decode(bytes.as_ref()))),
        );
        group.bench_with_input(
            BenchmarkId::new("v5_connect_decode", size),
            &v5_conn_bytes,
            |b, bytes| b.iter(|| black_box(PacketV5::decode(bytes.as_ref()))),
        );
        group.bench_with_input(
            BenchmarkId::new("v3_publish_decode", size),
            &v3_pub_bytes,
            |b, bytes| b.iter(|| black_box(PacketV3::decode(bytes.as_ref()))),
        );
        group.bench_with_input(
            BenchmarkId::new("v5_publish_decode", size),
            &v5_pub_bytes,
            |b, bytes| b.iter(|| black_box(PacketV5::decode(bytes.as_ref()))),
        );

        group.finish();
    }
}

criterion_group!(codec_benches, bench_all);
criterion_main!(codec_benches);
