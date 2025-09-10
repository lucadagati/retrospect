use wasmbed_microros_bridge::{MicroRosBridge, BridgeConfig, QosConfig, Px4Config};
use wasmbed_fastdds_middleware::{FastDdsMiddleware, MiddlewareConfig, QosConfig as FastDdsQosConfig};
use wasmbed_px4_communication::{Px4CommunicationBridge, Px4BridgeConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Wasmbed Platform Integration");
    
    // Test microROS bridge
    println!("1. Testing microROS bridge...");
    let microros_config = BridgeConfig {
        dds_domain_id: 0,
        node_name: "test_node".to_string(),
        qos: QosConfig::default(),
        px4_config: Px4Config::default(),
    };
    
    let microros_bridge = MicroRosBridge::new(microros_config).await?;
    microros_bridge.initialize().await?;
    println!("   ✓ microROS bridge initialized successfully");
    
    // Test FastDDS middleware
    println!("2. Testing FastDDS middleware...");
    let fastdds_config = MiddlewareConfig {
        domain_id: 0,
        qos: FastDdsQosConfig::default(),
        transport: wasmbed_fastdds_middleware::TransportConfig::default(),
    };
    
    let fastdds = FastDdsMiddleware::new(fastdds_config).await?;
    fastdds.initialize().await?;
    println!("   ✓ FastDDS middleware initialized successfully");
    
    // Test PX4 communication bridge
    println!("3. Testing PX4 communication bridge...");
    let px4_config = Px4BridgeConfig {
        dds_domain_id: 0,
        node_name: "px4_bridge".to_string(),
        system_id: 1,
        component_id: 1,
        mavlink_version: 2,
        heartbeat_interval: Duration::from_secs(1),
        command_timeout: Duration::from_secs(5),
    };
    
    let px4_bridge = Px4CommunicationBridge::new(px4_config).await?;
    px4_bridge.initialize().await?;
    println!("   ✓ PX4 communication bridge initialized successfully");
    
    // Test message publishing
    println!("4. Testing message publishing...");
    let test_data = b"test message".to_vec();
    microros_bridge.publish_message("/fmu/in/vehicle_command", test_data).await?;
    println!("   ✓ Message published successfully");
    
    // Test topic subscription
    println!("5. Testing topic subscription...");
    microros_bridge.subscribe_to_topic("/fmu/out/vehicle_status").await?;
    println!("   ✓ Topic subscribed successfully");
    
    // Test status retrieval
    println!("6. Testing status retrieval...");
    let microros_status = microros_bridge.get_status().await;
    let fastdds_status = fastdds.get_status().await;
    let px4_status = px4_bridge.get_status().await;
    
    println!("   microROS bridge status: initialized={}, connected={}", 
             microros_status.initialized, microros_status.connected);
    println!("   FastDDS middleware status: {:?}", fastdds_status);
    println!("   PX4 bridge status: initialized={}, connected={}", 
             px4_status.initialized, px4_status.connected);
    
    // Test shutdown
    println!("7. Testing shutdown...");
    microros_bridge.shutdown().await?;
    fastdds.shutdown().await?;
    px4_bridge.shutdown().await?;
    println!("   ✓ All bridges shutdown successfully");
    
    println!("\n🎉 All tests passed! Wasmbed Platform is working correctly.");
    Ok(())
}
