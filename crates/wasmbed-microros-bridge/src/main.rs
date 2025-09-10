// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use wasmbed_microros_bridge::{BridgeConfig, MicroRosBridge, WasmRuntimeIntegration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wasmbed_microros_bridge=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Wasmbed microROS Bridge");

    // Get configuration from environment variables
    let dds_domain_id = env::var("ROS_DOMAIN_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u32>()
        .unwrap_or(0);

    let node_name = env::var("NODE_NAME")
        .unwrap_or_else(|_| "wasmbed_microros_bridge".to_string());

    let gateway_url = env::var("WASMBED_GATEWAY_URL")
        .unwrap_or_else(|_| "http://wasmbed-gateway.wasmbed-system.svc.cluster.local:8080".to_string());

    let runtime_id = env::var("RUNTIME_ID")
        .unwrap_or_else(|_| "default-runtime".to_string());

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Create bridge configuration
    let config = BridgeConfig {
        dds_domain_id,
        node_name: node_name.clone(),
        qos: Default::default(),
        px4_config: Default::default(),
    };

    // Create WASM runtime integration
    let wasm_integration = WasmRuntimeIntegration {
        gateway_url,
        runtime_id,
    };

    // Create and initialize the bridge
    let bridge = Arc::new(
        MicroRosBridge::new(config, wasm_integration)
            .await
            .map_err(|e| {
                error!("Failed to create microROS bridge: {}", e);
                e
            })?
    );

    // Initialize the bridge
    bridge.initialize().await.map_err(|e| {
        error!("Failed to initialize microROS bridge: {}", e);
        e
    })?;

    info!("microROS bridge initialized successfully");

    // Create HTTP API router
    let app = Router::new()
        .merge(bridge.create_api_router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        );

    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
