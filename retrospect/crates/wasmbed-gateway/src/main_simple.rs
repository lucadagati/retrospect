// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{Level, info, error};
use tracing_subscriber::FmtSubscriber;

use wasmbed_tls_utils::{TlsUtils, TlsServer};

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
struct Args {
    #[arg(long, env = "WASMBED_GATEWAY_BIND_ADDR")]
    bind_addr: SocketAddr,
    #[arg(long, env = "WASMBED_GATEWAY_PRIVATE_KEY")]
    private_key: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CERTIFICATE")]
    certificate: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CLIENT_CA")]
    client_ca: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_NAMESPACE")]
    namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAMESPACE")]
    pod_namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAME")]
    pod_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let private_key_bytes =
        std::fs::read(&args.private_key).with_context(|| {
            format!(
                "Failed to read private key from {}",
                args.private_key.display()
            )
        })?;
    let certificate_bytes =
        std::fs::read(&args.certificate).with_context(|| {
            format!(
                "Failed to read certificate from {}",
                args.certificate.display()
            )
        })?;
    let client_ca_bytes =
        std::fs::read(&args.client_ca).with_context(|| {
            format!(
                "Failed to read client CA certificate from {}",
                args.client_ca.display()
            )
        })?;

    // Parse PEM certificates using our custom TLS utils
    let private_key = TlsUtils::parse_private_key(&private_key_bytes)
        .with_context(|| "Failed to parse private key")?;
    
    let certificate = TlsUtils::parse_certificate(&certificate_bytes)
        .with_context(|| "Failed to parse certificate")?;
    
    let client_ca_certs = TlsUtils::parse_certificates(&client_ca_bytes)
        .with_context(|| "Failed to parse client CA certificates")?;

    let server_key = match private_key {
        rustls_pki_types::PrivateKeyDer::Pkcs8(pkcs8) => pkcs8,
        _ => return Err(anyhow::anyhow!("Only PKCS8 private keys are supported")),
    };
    
    let client_ca = client_ca_certs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No CA certificate found in PEM file"))?;

    // Create custom TLS server
    let tls_server = TlsServer::new(args.bind_addr, certificate, server_key, client_ca);
    
    info!("Starting Wasmbed Gateway with custom TLS implementation");
    info!("Namespace: {}", args.namespace);
    info!("Pod: {}/{}", args.pod_namespace, args.pod_name);
    
    // Start the TLS server
    tls_server.start().await?;

    Ok(())
}
