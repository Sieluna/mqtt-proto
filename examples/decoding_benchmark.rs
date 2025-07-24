use std::convert::TryFrom;
use std::hint::black_box;

use mqtt_proto::{
    v3::{Connect as ConnectV3, LastWill as LastWillV3, Packet as PacketV3, Publish as PublishV3},
    v5::{Connect as ConnectV5, LastWill as LastWillV5, Packet as PacketV5, Publish as PublishV5},
    Pid, QoS, QosPid, TopicName,
};

/// 专门用于解码性能分析的示例
/// 这个示例会执行大量的解码操作，用于flamegraph分析
fn main() {
    println!("🔥 MQTT解码性能热点分析");
    println!("运行flamegraph: cargo flamegraph --example decoding_benchmark");
    println!("专注于解码操作的性能瓶颈识别");
    
    // 检测编译模式
    let build_mode = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("构建模式: {}", build_mode);
    println!("防止编译器优化: 启用black_box\n");

    // 预编码测试数据
    let test_data = prepare_test_data();
    
    // V3协议解码测试
    println!("🚀 === MQTT v3.1.1 解码测试 ===");
    intensive_v3_connect_decoding(&test_data.v3_connect_packets);
    intensive_v3_publish_decoding(&test_data.v3_publish_packets);
    mixed_size_v3_decoding(&test_data.v3_mixed_size_packets);
    stress_test_v3_decoding(&test_data.v3_large_packets);
    
    println!();
    
    // V5协议解码测试
    println!("🚀 === MQTT v5.0 解码测试 ===");
    intensive_v5_connect_decoding(&test_data.v5_connect_packets);
    intensive_v5_publish_decoding(&test_data.v5_publish_packets);
    mixed_size_v5_decoding(&test_data.v5_mixed_size_packets);
    stress_test_v5_decoding(&test_data.v5_large_packets);
    
    println!();
    
    // 协议版本对比测试
    println!("🚀 === 协议版本对比测试 ===");
    compare_protocol_decoding(&test_data);
}

struct TestData {
    // V3协议测试数据
    v3_connect_packets: Vec<mqtt_proto::VarBytes>,
    v3_publish_packets: Vec<mqtt_proto::VarBytes>,
    v3_mixed_size_packets: Vec<mqtt_proto::VarBytes>,
    v3_large_packets: Vec<mqtt_proto::VarBytes>,
    
    // V5协议测试数据
    v5_connect_packets: Vec<mqtt_proto::VarBytes>,
    v5_publish_packets: Vec<mqtt_proto::VarBytes>,
    v5_mixed_size_packets: Vec<mqtt_proto::VarBytes>,
    v5_large_packets: Vec<mqtt_proto::VarBytes>,
}

fn prepare_test_data() -> TestData {
    println!("📦 准备测试数据...");
    
    // V3协议测试数据
    let v3_connect_packets = create_encoded_v3_connects();
    let v3_publish_packets = create_encoded_v3_publishes();
    let v3_mixed_size_packets = create_mixed_size_v3_packets();
    let v3_large_packets = create_large_encoded_v3_packets();
    
    // V5协议测试数据
    let v5_connect_packets = create_encoded_v5_connects();
    let v5_publish_packets = create_encoded_v5_publishes();
    let v5_mixed_size_packets = create_mixed_size_v5_packets();
    let v5_large_packets = create_large_encoded_v5_packets();
    
    println!("✅ 测试数据准备完成");
    println!("  V3 Connect包数量: {}", v3_connect_packets.len());
    println!("  V3 Publish包数量: {}", v3_publish_packets.len());
    println!("  V3 混合大小包数量: {}", v3_mixed_size_packets.len());
    println!("  V3 大载荷包数量: {}", v3_large_packets.len());
    println!("  V5 Connect包数量: {}", v5_connect_packets.len());
    println!("  V5 Publish包数量: {}", v5_publish_packets.len());
    println!("  V5 混合大小包数量: {}", v5_mixed_size_packets.len());
    println!("  V5 大载荷包数量: {}", v5_large_packets.len());
    
    // 计算总的测试数据大小
    let v3_total_size: usize = v3_connect_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v3_publish_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v3_mixed_size_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v3_large_packets.iter().map(|p| p.as_ref().len()).sum::<usize>();
    
    let v5_total_size: usize = v5_connect_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v5_publish_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v5_mixed_size_packets.iter().map(|p| p.as_ref().len()).sum::<usize>()
        + v5_large_packets.iter().map(|p| p.as_ref().len()).sum::<usize>();
    
    println!("  V3总测试数据大小: {} 字节", v3_total_size);
    println!("  V5总测试数据大小: {} 字节", v5_total_size);
    println!();
    
    TestData {
        v3_connect_packets,
        v3_publish_packets,
        v3_mixed_size_packets,
        v3_large_packets,
        v5_connect_packets,
        v5_publish_packets,
        v5_mixed_size_packets,
        v5_large_packets,
    }
}

// === V3协议解码测试 ===

fn intensive_v3_connect_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 密集V3 Connect包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 50_000 } else { 100_000 };
    let mut total_decoded_objects = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV3::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_decoded_objects += 1;
                    
                    if round % (total_iterations / 10) == 0 && i == 0 {
                        println!("  V3 Connect解码轮次: {}, 累计解码: {}", round, successful_decodes);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V3 Connect解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V3 Connect解码测试完成，成功解码: {} 次, 对象数: {}", successful_decodes, total_decoded_objects);
}

fn intensive_v3_publish_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 密集V3 Publish包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 25_000 } else { 50_000 };
    let mut total_bytes_decoded = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV3::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_bytes_decoded += black_box(packet.as_ref().len());
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V3 Publish解码轮次: {}, 累计字节: {}", round, total_bytes_decoded);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V3 Publish解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V3 Publish解码测试完成，成功解码: {} 次, 总字节: {}", successful_decodes, total_bytes_decoded);
}

fn mixed_size_v3_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 混合大小V3包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 15_000 } else { 30_000 };
    let mut total_packet_count = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV3::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_packet_count += 1;
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V3混合大小解码轮次: {}, 包数: {}", round, total_packet_count);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V3混合大小解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V3混合大小解码测试完成，成功解码: {} 次, 包数: {}", successful_decodes, total_packet_count);
}

fn stress_test_v3_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 V3大载荷解码压力测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 2_500 } else { 5_000 };
    let mut total_large_bytes = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV3::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_large_bytes += black_box(packet.as_ref().len());
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V3大载荷解码轮次: {}, 累计大字节: {}", round, total_large_bytes);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V3大载荷解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V3大载荷解码压力测试完成，成功解码: {} 次, 大字节数: {}", successful_decodes, total_large_bytes);
}

// === V5协议解码测试 ===

fn intensive_v5_connect_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 密集V5 Connect包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 50_000 } else { 100_000 };
    let mut total_decoded_objects = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV5::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_decoded_objects += 1;
                    
                    if round % (total_iterations / 10) == 0 && i == 0 {
                        println!("  V5 Connect解码轮次: {}, 累计解码: {}", round, successful_decodes);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V5 Connect解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V5 Connect解码测试完成，成功解码: {} 次, 对象数: {}", successful_decodes, total_decoded_objects);
}

fn intensive_v5_publish_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 密集V5 Publish包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 25_000 } else { 50_000 };
    let mut total_bytes_decoded = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV5::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_bytes_decoded += black_box(packet.as_ref().len());
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V5 Publish解码轮次: {}, 累计字节: {}", round, total_bytes_decoded);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V5 Publish解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V5 Publish解码测试完成，成功解码: {} 次, 总字节: {}", successful_decodes, total_bytes_decoded);
}

fn mixed_size_v5_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 混合大小V5包解码测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 15_000 } else { 30_000 };
    let mut total_packet_count = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV5::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_packet_count += 1;
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V5混合大小解码轮次: {}, 包数: {}", round, total_packet_count);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V5混合大小解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V5混合大小解码测试完成，成功解码: {} 次, 包数: {}", successful_decodes, total_packet_count);
}

fn stress_test_v5_decoding(packets: &[mqtt_proto::VarBytes]) {
    println!("🔥 V5大载荷解码压力测试...");
    
    let mut successful_decodes = 0usize;
    let total_iterations = if cfg!(debug_assertions) { 2_500 } else { 5_000 };
    let mut total_large_bytes = 0usize;
    
    for round in 0..total_iterations {
        for (i, packet) in packets.iter().enumerate() {
            match black_box(PacketV5::decode(packet.as_ref())) {
                Ok(decoded) => {
                    successful_decodes += 1;
                    total_large_bytes += black_box(packet.as_ref().len());
                    
                    if round % (total_iterations / 5) == 0 && i == 0 {
                        println!("  V5大载荷解码轮次: {}, 累计大字节: {}", round, total_large_bytes);
                        black_box(&decoded);
                    }
                }
                Err(e) => {
                    eprintln!("V5大载荷解码错误: {:?}", e);
                    return;
                }
            }
        }
    }
    
    println!("✅ V5大载荷解码压力测试完成，成功解码: {} 次, 大字节数: {}", successful_decodes, total_large_bytes);
}

// === 协议版本对比测试 ===

fn compare_protocol_decoding(test_data: &TestData) {
    println!("🔥 V3 vs V5 协议解码性能对比...");
    
    let iterations = if cfg!(debug_assertions) { 10_000 } else { 20_000 };
    
    // V3 Connect性能
    let start = std::time::Instant::now();
    let mut v3_count = 0usize;
    
    for _ in 0..iterations {
        for packet in &test_data.v3_connect_packets {
            let decoded = black_box(PacketV3::decode(packet.as_ref()).unwrap());
            v3_count += 1;
            black_box(&decoded);
        }
    }
    let v3_duration = start.elapsed();
    let v3_ops_per_sec = v3_count as f64 / v3_duration.as_secs_f64();
    
    // V5 Connect性能
    let start = std::time::Instant::now();
    let mut v5_count = 0usize;
    
    for _ in 0..iterations {
        for packet in &test_data.v5_connect_packets {
            let decoded = black_box(PacketV5::decode(packet.as_ref()).unwrap());
            v5_count += 1;
            black_box(&decoded);
        }
    }
    let v5_duration = start.elapsed();
    let v5_ops_per_sec = v5_count as f64 / v5_duration.as_secs_f64();
    
    println!("📊 协议解码性能对比结果:");
    println!("  V3 Connect解码: {:.0} 次/秒", v3_ops_per_sec);
    println!("  V5 Connect解码: {:.0} 次/秒", v5_ops_per_sec);
    
    let performance_ratio = v5_ops_per_sec / v3_ops_per_sec;
    if performance_ratio > 1.0 {
        println!("  🚀 V5比V3快 {:.1}x", performance_ratio);
    } else {
        println!("  📉 V5比V3慢 {:.1}x", 1.0 / performance_ratio);
    }
    
    println!("✅ 协议版本对比测试完成");
}

// === V3协议测试数据创建函数 ===

fn create_encoded_v3_connects() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    
    // 最小Connect包
    let packet = PacketV3::Connect(ConnectV3::new(black_box("min_v3".into()), 60));
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 带认证的Connect包
    let client_id = black_box("auth_client_v3".into());
    let mut connect = ConnectV3::new(client_id, 30);
    connect.username = Some(black_box("username_v3".into()));
    connect.password = Some(black_box("password_v3".into()));
    let packet = PacketV3::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 完整功能Connect包
    let client_id = black_box("full_client_v3".into());
    let mut connect = ConnectV3::new(client_id, 30);
    connect.username = Some(black_box("user_v3".into()));
    connect.password = Some(black_box("pass_v3".into()));
    connect.last_will = Some(LastWillV3 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("will/topic/v3").unwrap(),
        message: black_box("will message v3".into()),
    });
    let packet = PacketV3::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 长标识符Connect包
    let long_client_id = black_box("very_long_client_identifier_for_v3_decode_testing_performance".into());
    let mut connect = ConnectV3::new(long_client_id, 120);
    connect.username = Some(black_box("long_username_for_v3_decoding_test".into()));
    connect.password = Some(black_box("long_password_for_v3_decoding_performance_test".into()));
    connect.last_will = Some(LastWillV3 {
        qos: QoS::Level2,
        retain: true,
        topic_name: TopicName::try_from("very/long/will/topic/path/for/v3/testing").unwrap(),
        message: black_box("This is a very long will message for testing v3 decoding performance".into()),
    });
    let packet = PacketV3::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    encoded_packets
}

fn create_encoded_v3_publishes() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let sizes = [0, 100, 1000, 5000, 10000];
    
    for &size in &sizes {
        let payload = black_box("x".repeat(size));
        
        // QoS 0 Publish
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level0,
            topic_name: TopicName::try_from("test/topic/v3").unwrap(),
            payload: payload.clone().into(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
        
        // QoS 1 Publish
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("test/topic/v3").unwrap(),
            payload: payload.clone().into(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
        
        // QoS 2 Publish  
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level2(Pid::try_from(2).unwrap()),
            topic_name: TopicName::try_from("test/topic/v3").unwrap(),
            payload: payload.into(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
}

fn create_mixed_size_v3_packets() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let sizes = [10, 50, 200, 800, 3200, 12800];
    
    for &size in &sizes {
        let payload = black_box("y".repeat(size));
        
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(42).unwrap()),
            topic_name: TopicName::try_from("mixed/size/topic/v3").unwrap(),
            payload: payload.into(),
        });
        
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
}

fn create_large_encoded_v3_packets() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let large_sizes = [20000, 50000, 100000];
    
    for &size in &large_sizes {
        let payload = black_box("L".repeat(size));
        
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(100).unwrap()),
            topic_name: TopicName::try_from("large/decode/test/topic/v3").unwrap(),
            payload: payload.into(),
        });
        
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
}

// === V5协议测试数据创建函数 ===

fn create_encoded_v5_connects() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    
    // 最小Connect包
    let packet = PacketV5::Connect(ConnectV5::new(black_box("min_v5".into()), 60));
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 带认证的Connect包
    let client_id = black_box("auth_client_v5".into());
    let mut connect = ConnectV5::new(client_id, 30);
    connect.username = Some(black_box("username_v5".into()));
    connect.password = Some(black_box("password_v5".into()));
    let packet = PacketV5::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 完整功能Connect包
    let client_id = black_box("full_client_v5".into());
    let mut connect = ConnectV5::new(client_id, 30);
    connect.username = Some(black_box("user_v5".into()));
    connect.password = Some(black_box("pass_v5".into()));
    connect.last_will = Some(LastWillV5 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("will/topic/v5").unwrap(),
        payload: black_box("will message v5".into()),
        properties: Default::default(),
    });
    let packet = PacketV5::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    // 长标识符Connect包
    let long_client_id = black_box("very_long_client_identifier_for_v5_decode_testing_performance".into());
    let mut connect = ConnectV5::new(long_client_id, 120);
    connect.username = Some(black_box("long_username_for_v5_decoding_test".into()));
    connect.password = Some(black_box("long_password_for_v5_decoding_performance_test".into()));
    connect.last_will = Some(LastWillV5 {
        qos: QoS::Level2,
        retain: true,
        topic_name: TopicName::try_from("very/long/will/topic/path/for/v5/testing").unwrap(),
        payload: black_box("This is a very long will message for testing v5 decoding performance with various features".into()),
        properties: Default::default(),
    });
    let packet = PacketV5::Connect(connect);
    encoded_packets.push(black_box(packet.encode().unwrap()));
    
    encoded_packets
}

fn create_encoded_v5_publishes() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let sizes = [0, 100, 1000, 5000, 10000];
    
    for &size in &sizes {
        let payload = black_box("x".repeat(size));
        
        // QoS 0 Publish
        let packet = PacketV5::Publish(PublishV5 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level0,
            topic_name: TopicName::try_from("test/topic/v5").unwrap(),
            payload: payload.clone().into(),
            properties: Default::default(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
        
        // QoS 1 Publish
        let packet = PacketV5::Publish(PublishV5 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("test/topic/v5").unwrap(),
            payload: payload.clone().into(),
            properties: Default::default(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
        
        // QoS 2 Publish  
        let packet = PacketV5::Publish(PublishV5 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level2(Pid::try_from(2).unwrap()),
            topic_name: TopicName::try_from("test/topic/v5").unwrap(),
            payload: payload.into(),
            properties: Default::default(),
        });
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
}

fn create_mixed_size_v5_packets() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let sizes = [10, 50, 200, 800, 3200, 12800];
    
    for &size in &sizes {
        let payload = black_box("y".repeat(size));
        
        let packet = PacketV5::Publish(PublishV5 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(42).unwrap()),
            topic_name: TopicName::try_from("mixed/size/topic/v5").unwrap(),
            payload: payload.into(),
            properties: Default::default(),
        });
        
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
}

fn create_large_encoded_v5_packets() -> Vec<mqtt_proto::VarBytes> {
    let mut encoded_packets = Vec::new();
    let large_sizes = [20000, 50000, 100000];
    
    for &size in &large_sizes {
        let payload = black_box("L".repeat(size));
        
        let packet = PacketV5::Publish(PublishV5 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(100).unwrap()),
            topic_name: TopicName::try_from("large/decode/test/topic/v5").unwrap(),
            payload: payload.into(),
            properties: Default::default(),
        });
        
        encoded_packets.push(black_box(packet.encode().unwrap()));
    }
    
    encoded_packets
} 