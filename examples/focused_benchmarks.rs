use std::convert::TryFrom;
use std::time::Instant;
use std::hint::black_box;

use mqtt_proto::{
    v3::{Connect as ConnectV3, LastWill as LastWillV3, Packet as PacketV3, Publish as PublishV3},
    v5::{Connect as ConnectV5, LastWill as LastWillV5, Packet as PacketV5, Publish as PublishV5},
    Pid, QoS, QosPid, TopicName,
};

fn main() {
    println!("=== MQTT Proto 性能瓶颈分析示例 ===");
    println!("模型信息: Claude 3.5 Sonnet (20241022)");
    println!("用于flamegraph分析的综合性能测试");
    
    // 检测编译模式
    let build_mode = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("构建模式: {}", build_mode);
    println!("防止编译器优化: 启用black_box\n");

    // 测试1: 基本编码性能
    test_basic_encoding();
    
    // 测试2: 大载荷编码性能
    test_large_payload_encoding();
    
    // 测试3: 解码性能
    test_decoding_performance();
    
    // 测试4: 往返性能
    test_roundtrip_performance();
    
    // 测试5: 内存分配分析
    test_memory_allocation();
    
    // 测试6: V5协议特性性能
    test_v5_features();
    
    // 测试7: 验证防优化措施
    test_optimization_prevention();
}

fn test_basic_encoding() {
    println!("🔍 测试1: 基本编码性能");
    
    // 创建简单的V3 Connect包
    let client_id = black_box("test_client".into());
    let connect = ConnectV3::new(client_id, 60);
    let packet = PacketV3::Connect(connect);
    
    // 测量编码时间
    let iterations = if cfg!(debug_assertions) { 50_000 } else { 100_000 };
    let start = Instant::now();
    let mut total_bytes = 0usize;
    
    for i in 0..iterations {
        let encoded = black_box(packet.encode().unwrap());
        total_bytes += black_box(encoded.as_ref().len());
        
        // 确保结果被使用，防止优化
        if i % (iterations / 10) == 0 {
            black_box(&encoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!("  V3 Connect编码: {:.0} 次/秒, 总字节: {}", ops_per_sec, total_bytes);
    
    // V5版本对比
    let client_id = black_box("test_client_v5".into());
    let connect = ConnectV5::new(client_id, 60);
    let packet = PacketV5::Connect(connect);
    
    let start = Instant::now();
    let mut total_bytes = 0usize;
    
    for i in 0..iterations {
        let encoded = black_box(packet.encode().unwrap());
        total_bytes += black_box(encoded.as_ref().len());
        
        if i % (iterations / 10) == 0 {
            black_box(&encoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!("  V5 Connect编码: {:.0} 次/秒, 总字节: {}", ops_per_sec, total_bytes);
    println!();
}

fn test_large_payload_encoding() {
    println!("🔍 测试2: 大载荷编码性能");
    
    let sizes = [1024, 4096, 16384, 65536];
    let iterations = if cfg!(debug_assertions) { 5_000 } else { 10_000 };
    
    for &size in &sizes {
        let payload = black_box("x".repeat(size));
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
            topic_name: TopicName::try_from("test/topic").unwrap(),
            payload: payload.into(),
        });
        
        let start = Instant::now();
        let mut total_bytes = 0usize;
        
        for i in 0..iterations {
            let encoded = black_box(packet.encode().unwrap());
            total_bytes += black_box(encoded.as_ref().len());
            
            // 定期访问结果防止优化
            if i % (iterations / 5) == 0 {
                black_box(&encoded);
            }
        }
        
        let duration = start.elapsed();
        let throughput = (total_bytes as f64) / duration.as_secs_f64() / 1024.0 / 1024.0;
        println!("  {}KB载荷: {:.2} MB/s, {} 次编码", size / 1024, throughput, iterations);
    }
    println!();
}

fn test_decoding_performance() {
    println!("🔍 测试3: 解码性能");
    
    // 预编码一些测试数据
    let client_id = black_box("decode_test_client".into());
    let mut connect = ConnectV3::new(client_id, 60);
    connect.username = Some(black_box("user".into()));
    connect.password = Some(black_box("pass".into()));
    connect.last_will = Some(LastWillV3 {
        qos: QoS::Level1,
        retain: false,
        topic_name: TopicName::try_from("will/topic").unwrap(),
        message: black_box("will message".into()),
    });
    
    let packet = PacketV3::Connect(connect);
    let encoded = black_box(packet.encode().unwrap());
    
    let iterations = if cfg!(debug_assertions) { 50_000 } else { 100_000 };
    let start = Instant::now();
    let mut decode_count = 0usize;
    
    for i in 0..iterations {
        let decoded = black_box(PacketV3::decode(encoded.as_ref()).unwrap());
        decode_count += 1;
        
        // 确保解码结果被使用
        if i % (iterations / 10) == 0 {
            black_box(&decoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = decode_count as f64 / duration.as_secs_f64();
    println!("  V3 Connect解码: {:.0} 次/秒, 解码次数: {}", ops_per_sec, decode_count);
    
    // V5解码性能测试
    let client_id = black_box("decode_test_client_v5".into());
    let mut connect = ConnectV5::new(client_id, 60);
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
    let encoded = black_box(packet.encode().unwrap());
    
    let start = Instant::now();
    let mut decode_count = 0usize;
    
    for i in 0..iterations {
        let decoded = black_box(PacketV5::decode(encoded.as_ref()).unwrap());
        decode_count += 1;
        
        if i % (iterations / 10) == 0 {
            black_box(&decoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = decode_count as f64 / duration.as_secs_f64();
    println!("  V5 Connect解码: {:.0} 次/秒, 解码次数: {}", ops_per_sec, decode_count);
    println!();
}

fn test_roundtrip_performance() {
    println!("🔍 测试4: 往返性能");
    
    let client_id = black_box("roundtrip_client".into());
    let connect = ConnectV3::new(client_id, 60);
    let packet = PacketV3::Connect(connect);
    
    let iterations = if cfg!(debug_assertions) { 25_000 } else { 50_000 };
    let start = Instant::now();
    let mut roundtrip_count = 0usize;
    
    for i in 0..iterations {
        let encoded = black_box(packet.encode().unwrap());
        let decoded = black_box(PacketV3::decode(encoded.as_ref()).unwrap());
        roundtrip_count += 1;
        
        // 确保往返结果被使用
        if i % (iterations / 10) == 0 {
            black_box(&encoded);
            black_box(&decoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = roundtrip_count as f64 / duration.as_secs_f64();
    println!("  V3 Connect往返: {:.0} 次/秒, 往返次数: {}", ops_per_sec, roundtrip_count);
    
    // V5往返性能
    let client_id = black_box("roundtrip_client_v5".into());
    let connect = ConnectV5::new(client_id, 60);
    let packet = PacketV5::Connect(connect);
    
    let start = Instant::now();
    let mut roundtrip_count = 0usize;
    
    for i in 0..iterations {
        let encoded = black_box(packet.encode().unwrap());
        let decoded = black_box(PacketV5::decode(encoded.as_ref()).unwrap());
        roundtrip_count += 1;
        
        if i % (iterations / 10) == 0 {
            black_box(&encoded);
            black_box(&decoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = roundtrip_count as f64 / duration.as_secs_f64();
    println!("  V5 Connect往返: {:.0} 次/秒, 往返次数: {}", ops_per_sec, roundtrip_count);
    println!();
}

fn test_memory_allocation() {
    println!("🔍 测试5: 内存分配模式");
    
    // 测试不同大小载荷的分配模式
    let sizes = [0, 100, 1000, 10000];
    let iterations = if cfg!(debug_assertions) { 5_000 } else { 10_000 };
    
    for &size in &sizes {
        let payload = black_box("x".repeat(size));
        let packet = PacketV3::Publish(PublishV3 {
            dup: false,
            retain: false,
            qos_pid: QosPid::Level0,
            topic_name: TopicName::try_from("test").unwrap(),
            payload: payload.into(),
        });
        
        // 测量编码时间和内存使用
        let start = Instant::now();
        let mut total_bytes = 0usize;
        
        for i in 0..iterations {
            let encoded = black_box(packet.encode().unwrap());
            total_bytes += black_box(encoded.as_ref().len());
            
            if i % (iterations / 5) == 0 {
                black_box(&encoded);
            }
        }
        
        let duration = start.elapsed();
        let avg_time = duration.as_micros() as f64 / iterations as f64;
        println!("  {}字节载荷: {:.2}μs/次, 总分配: {}字节", size, avg_time, total_bytes);
    }
    println!();
}

fn test_v5_features() {
    println!("🔍 测试6: V5协议特性性能");
    
    // V5 Publish包性能测试
    let payload = black_box("v5 test payload".repeat(100));
    let packet = PacketV5::Publish(PublishV5 {
        dup: false,
        retain: false,
        qos_pid: QosPid::Level1(Pid::try_from(1).unwrap()),
        topic_name: TopicName::try_from("v5/test/topic").unwrap(),
        payload: payload.into(),
        properties: Default::default(),
    });
    
    let iterations = if cfg!(debug_assertions) { 25_000 } else { 50_000 };
    let start = Instant::now();
    let mut encode_count = 0usize;
    
    for i in 0..iterations {
        let encoded = black_box(packet.encode().unwrap());
        encode_count += 1;
        
        if i % (iterations / 10) == 0 {
            black_box(&encoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = encode_count as f64 / duration.as_secs_f64();
    println!("  V5 Publish编码: {:.0} 次/秒, 编码次数: {}", ops_per_sec, encode_count);
    
    // 测试往返性能
    let start = Instant::now();
    let mut roundtrip_count = 0usize;
    let half_iterations = iterations / 2;
    
    for i in 0..half_iterations {
        let encoded = black_box(packet.encode().unwrap());
        let decoded = black_box(PacketV5::decode(encoded.as_ref()).unwrap());
        roundtrip_count += 1;
        
        if i % (half_iterations / 5) == 0 {
            black_box(&encoded);
            black_box(&decoded);
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = roundtrip_count as f64 / duration.as_secs_f64();
    println!("  V5 Publish往返: {:.0} 次/秒, 往返次数: {}", ops_per_sec, roundtrip_count);
    
    println!();
}

fn test_optimization_prevention() {
    println!("🔍 测试7: 验证防优化措施");
    
    let start = Instant::now();
    let mut dummy_sum = 0usize;
    
    // 创建一些计算工作，确保不被优化掉
    for i in 0..10_000 {
        let client_id = black_box(format!("client_{}", i));
        let connect = ConnectV3::new(client_id.into(), 60);
        let packet = PacketV3::Connect(connect);
        
        let encoded = black_box(packet.encode().unwrap());
        dummy_sum += black_box(encoded.as_ref().len());
        
        // 定期使用结果
        if i % 1000 == 0 {
            black_box(&encoded);
        }
    }
    
    let duration = start.elapsed();
    println!("  防优化验证: {}ms, 虚拟和: {}", duration.as_millis(), dummy_sum);
    
    println!();
    println!("✅ 性能分析完成！");
    println!("💡 运行flamegraph分析这些热点:");
    println!("   cargo flamegraph --example focused_benchmarks");
    println!("💡 或者在dev模式下运行:");
    println!("   cargo flamegraph --example focused_benchmarks --dev");
} 