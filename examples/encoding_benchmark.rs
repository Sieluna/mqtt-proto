use std::convert::TryFrom;
use std::hint::black_box;

use mqtt_proto::{
    v3::{Connect as ConnectV3, LastWill as LastWillV3, Packet as PacketV3, Publish as PublishV3},
    v5::{Connect as ConnectV5, LastWill as LastWillV5, Packet as PacketV5, Publish as PublishV5},
    Pid, QoS, QosPid, TopicName,
};

/// 专门用于编码性能分析的示例
/// 这个示例会执行大量的编码操作，用于flamegraph分析
fn main() {
    println!("🔥 MQTT编码性能热点分析");
    println!("运行flamegraph: cargo flamegraph --example encoding_benchmark");
    println!("专注于编码操作的性能瓶颈识别");
    
    // 检测编译模式并调整迭代次数
    let build_mode = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("构建模式: {}", build_mode);
    println!("防止编译器优化: 启用black_box\n");

    // 高强度编码测试 - 适合flamegraph分析
    intensive_connect_encoding();
    intensive_publish_encoding();
    mixed_protocol_encoding();
    stress_test_encoding();
}

fn intensive_connect_encoding() {
    println!("🔥 密集Connect包编码测试...");
    
    // 创建各种复杂度的Connect包
    let packets = create_connect_variants();
    
    // 根据编译模式调整迭代次数
    let iterations = if cfg!(debug_assertions) { 25_000 } else { 50_000 };
    let mut total_bytes = 0usize;
    let mut encode_count = 0usize;
    
    // 大量重复编码操作，用于flamegraph分析
    for round in 0..iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(packet.encode()) {
                Ok(encoded) => {
                    total_bytes += black_box(encoded.as_ref().len());
                    encode_count += 1;
                    
                    // 确保结果被使用，防止优化
                    if round % (iterations / 10) == 0 && i == 0 {
                        println!("  编码轮次: {}, 累计字节: {}", round, total_bytes);
                        black_box(&encoded);
                    }
                }
                Err(e) => {
                    eprintln!("编码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ Connect编码测试完成，编码次数: {}, 总字节: {}", encode_count, total_bytes);
}

fn intensive_publish_encoding() {
    println!("🔥 密集Publish包编码测试...");
    
    // 创建不同大小的Publish包
    let packets = create_publish_variants();
    
    let iterations = if cfg!(debug_assertions) { 10_000 } else { 20_000 };
    let mut total_bytes = 0usize;
    let mut encode_count = 0usize;
    
    // 大量重复编码操作
    for round in 0..iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(packet.encode()) {
                Ok(encoded) => {
                    total_bytes += black_box(encoded.as_ref().len());
                    encode_count += 1;
                    
                    if round % (iterations / 5) == 0 && i == 0 {
                        println!("  Publish编码轮次: {}, 累计字节: {}", round, total_bytes);
                        black_box(&encoded);
                    }
                }
                Err(e) => {
                    eprintln!("Publish编码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ Publish编码测试完成，编码次数: {}, 总字节: {}", encode_count, total_bytes);
}

fn mixed_protocol_encoding() {
    println!("🔥 混合协议编码测试...");
    
    let v3_packets = create_v3_packets();
    let v5_packets = create_v5_packets();
    
    let iterations = if cfg!(debug_assertions) { 15_000 } else { 30_000 };
    let mut total_bytes = 0usize;
    let mut encode_count = 0usize;
    
    // 混合编码测试
    for round in 0..iterations {
        // V3编码
        for packet in &v3_packets {
            match black_box(packet.encode()) {
                Ok(encoded) => {
                    total_bytes += black_box(encoded.as_ref().len());
                    encode_count += 1;
                    
                    if round % (iterations / 10) == 0 {
                        black_box(&encoded);
                    }
                }
                Err(_) => {
                    eprintln!("V3编码错误");
                    return;
                }
            }
        }
        
        // V5编码
        for packet in &v5_packets {
            match black_box(packet.encode()) {
                Ok(encoded) => {
                    total_bytes += black_box(encoded.as_ref().len());
                    encode_count += 1;
                    
                    if round % (iterations / 10) == 0 {
                        black_box(&encoded);
                    }
                }
                Err(_) => {
                    eprintln!("V5编码错误");
                    return;
                }
            }
        }
        
        if round % (iterations / 5) == 0 {
            println!("  混合协议编码轮次: {}, 累计字节: {}", round, total_bytes);
        }
    }
    
    println!("✅ 混合协议编码测试完成，编码次数: {}, 总字节: {}", encode_count, total_bytes);
}

fn stress_test_encoding() {
    println!("🔥 压力测试编码...");
    
    // 创建大载荷包进行压力测试
    let large_payloads = create_large_payload_packets();
    
    let iterations = if cfg!(debug_assertions) { 2_500 } else { 5_000 };
    let mut total_bytes = 0usize;
    let mut encode_count = 0usize;
    
    for round in 0..iterations {
        for (i, packet) in large_payloads.iter().enumerate() {
            match black_box(packet.encode()) {
                Ok(encoded) => {
                    total_bytes += black_box(encoded.as_ref().len());
                    encode_count += 1;
                    
                    if round % (iterations / 5) == 0 && i == 0 {
                        println!("  大载荷编码轮次: {}, 累计字节: {}", round, total_bytes);
                        black_box(&encoded);
                    }
                }
                Err(e) => {
                    eprintln!("大载荷编码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ 压力测试编码完成，编码次数: {}, 总字节: {}", encode_count, total_bytes);
}

fn create_connect_variants() -> Vec<PacketV3> {
    let mut packets = Vec::new();
    
    // 最小Connect包
    packets.push(PacketV3::Connect(ConnectV3::new(
        black_box("min".into()), 
        60
    )));
    
    // 带用户名密码的Connect包
    let client_id = black_box("auth_client".into());
    let mut connect = ConnectV3::new(client_id, 30);
    connect.username = Some(black_box("username".into()));
    connect.password = Some(black_box("password".into()));
    packets.push(PacketV3::Connect(connect));
    
    // 完整功能的Connect包
    let client_id = black_box("full_client".into());
    let mut connect = ConnectV3::new(client_id, 30);
    connect.username = Some(black_box("user".into()));
    connect.password = Some(black_box("pass".into()));
    connect.last_will = Some(LastWillV3 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("will/topic").unwrap(),
        message: black_box("will message".into()),
    });
    packets.push(PacketV3::Connect(connect));
    
    // 长客户端ID的Connect包
    let long_client_id = black_box("very_long_client_identifier_for_testing_performance".into());
    let mut connect = ConnectV3::new(long_client_id, 120);
    connect.username = Some(black_box("long_username_for_testing".into()));
    connect.password = Some(black_box("long_password_for_performance_testing".into()));
    packets.push(PacketV3::Connect(connect));
    
    packets
}

fn create_publish_variants() -> Vec<PacketV3> {
    let sizes = [0, 100, 1000, 5000, 10000];
    let mut packets = Vec::new();
    
    for &size in &sizes {
        let payload = black_box("x".repeat(size));
        
        // QoS 0
        packets.push(PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level0,
            topic_name: TopicName::try_from("test/topic").unwrap(),
            payload: payload.clone().into(),
        }));
        
        // QoS 1
        packets.push(PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("test/topic").unwrap(),
            payload: payload.clone().into(),
        }));
        
        // QoS 2
        packets.push(PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level2(Pid::try_from(2).unwrap()),
            topic_name: TopicName::try_from("test/topic").unwrap(),
            payload: payload.into(),
        }));
    }
    
    packets
}

fn create_large_payload_packets() -> Vec<PacketV3> {
    let large_sizes = [20000, 50000, 100000];
    let mut packets = Vec::new();
    
    for &size in &large_sizes {
        let payload = black_box("L".repeat(size));
        packets.push(PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("large/payload/topic").unwrap(),
            payload: payload.into(),
        }));
    }
    
    packets
}

fn create_v3_packets() -> Vec<PacketV3> {
    vec![
        PacketV3::Connect(ConnectV3::new(black_box("v3_client".into()), 60)),
        PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("v3/topic").unwrap(),
            payload: black_box("v3 payload".into()),
        }),
    ]
}

fn create_v5_packets() -> Vec<PacketV5> {
    let mut packets = Vec::new();
    
    // V5 Connect包
    let client_id = black_box("v5_client".into());
    let mut connect = ConnectV5::new(client_id, 60);
    connect.last_will = Some(LastWillV5 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("v5/will/topic").unwrap(),
        payload: black_box("v5 will payload".into()),
        properties: Default::default(),
    });
    packets.push(PacketV5::Connect(connect));
    
    // V5 Publish包
    packets.push(PacketV5::Publish(PublishV5 {
        dup: false,
        retain: false,
        qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
        topic_name: TopicName::try_from("v5/topic").unwrap(),
        payload: black_box("v5 payload".into()),
        properties: Default::default(),
    }));
    
    packets
} 