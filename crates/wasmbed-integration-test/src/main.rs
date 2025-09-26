use wasmbed_microros_bridge::{MicroRosBridge, BridgeConfig, QosConfig, WasmRuntimeIntegration};
use wasmbed_fastdds_middleware::{FastDdsMiddleware, MiddlewareConfig, QosConfig as FastDdsQosConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Wasmbed Platform Integration (Hello World)");
    
    // Test microROS bridge
    println!("1. Testing microROS bridge...");
    let microros_config = BridgeConfig {
        dds_domain_id: 0,
        node_name: "test_node".to_string(),
        qos: QosConfig::default(),
    };
    
    let microros_bridge = MicroRosBridge::new(microros_config, WasmRuntimeIntegration::default()).await?;
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
    
    // Test message publishing
    println!("3. Testing message publishing...");
    let test_data = b"Hello, WASM World!".to_vec();
    microros_bridge.publish_message("/hello/world", test_data).await?;
    println!("   ✓ Hello world message published successfully");
    
    // Test topic subscription
    println!("4. Testing topic subscription...");
    microros_bridge.subscribe_to_topic("/hello/response").await?;
    println!("   ✓ Topic subscribed successfully");
    
    // Test status retrieval
    println!("5. Testing status retrieval...");
    let microros_status = microros_bridge.get_status().await;
    let fastdds_status = fastdds.get_status().await;
    
    println!("   microROS bridge status: initialized={}, connected={}", 
             microros_status.initialized, microros_status.connected);
    println!("   FastDDS middleware status: {:?}", fastdds_status);
    
    // Test shutdown
    println!("6. Testing shutdown...");
    microros_bridge.shutdown().await?;
    fastdds.shutdown().await?;
    println!("   ✓ All bridges shutdown successfully");
    
    println!("\n🎉 All tests passed! Wasmbed Platform (Hello World) is working correctly.");
    Ok(())
}