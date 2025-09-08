# Custom TLS Library Documentation

## Overview

The `wasmbed-tls-utils` crate provides a completely custom TLS implementation that replaces the standard `rustls` library in the Wasmbed platform. This custom implementation offers enhanced security, better control, and optimized performance specifically designed for IoT edge devices.

## Architecture

### Core Components

- **`TlsUtils`**: Main utility struct for certificate and key operations
- **`TlsServer`**: Custom TLS server implementation
- **`TlsClient`**: Custom TLS client implementation
- **`TlsConnection`**: TLS connection wrapper
- **`ServerIdentity`**: Server identity management
- **`MessageContext`**: Message handling context

### Certificate Management

- **PEM Parsing**: Support for PEM-encoded certificates and keys
- **DER Parsing**: Support for DER-encoded certificates and keys
- **PKCS8 Support**: Full PKCS8 private key format support
- **RSA Support**: Traditional RSA private key format support
- **X.509 Parsing**: Complete X.509 certificate parsing and validation

## Features

### Security Features

- **Ed25519 Signatures**: Modern elliptic curve digital signatures
- **AES-256-GCM Encryption**: High-performance authenticated encryption
- **Certificate Validation**: Complete X.509 certificate validation
- **Key Generation**: Secure key pair generation
- **Memory Safety**: Rust's memory safety guarantees

### Performance Features

- **Async I/O**: Full async/await support with tokio
- **Zero-Copy**: Optimized for minimal memory allocation
- **IoT Optimized**: Designed for resource-constrained devices
- **Customizable**: Full control over TLS handshake and encryption

## Usage

### Basic Server Setup

```rust
use wasmbed_tls_utils::{TlsUtils, TlsServer};

// Parse certificates
let private_key = TlsUtils::parse_private_key(&private_key_bytes)?;
let certificate = TlsUtils::parse_certificate(&certificate_bytes)?;
let client_ca = TlsUtils::parse_certificates(&client_ca_bytes)?;

// Create server
let server_key = match private_key {
    rustls_pki_types::PrivateKeyDer::Pkcs8(pkcs8) => pkcs8,
    _ => return Err("Only PKCS8 private keys are supported"),
};

let client_ca_cert = client_ca.into_iter().next().unwrap();
let tls_server = TlsServer::new(bind_addr, certificate, server_key, client_ca_cert);

// Start server
tls_server.start().await?;
```

### Certificate Parsing

```rust
use wasmbed_tls_utils::TlsUtils;

// Parse private key (supports PKCS8 and RSA formats)
let private_key = TlsUtils::parse_private_key(&pem_data)?;

// Parse certificate
let certificate = TlsUtils::parse_certificate(&cert_pem)?;

// Parse multiple certificates
let certificates = TlsUtils::parse_certificates(&multi_cert_pem)?;

// Validate certificate
let is_valid = TlsUtils::is_certificate_expired(&certificate)?;
let is_valid_for_hostname = TlsUtils::is_certificate_valid_for_hostname(&certificate, "example.com")?;
```

### Server Identity Management

```rust
use wasmbed_tls_utils::{TlsUtils, ServerIdentity};

// Create server identity
let identity = ServerIdentity::from_parts(server_key, certificate);

// Access components
let private_key = identity.private_key();
let cert = identity.certificate();
let cloned_key = identity.clone_key();
```

## Gateway Integration

### Simple Gateway

The `wasmbed-gateway-simple` binary demonstrates the basic usage of the custom TLS library:

```bash
# Build simple gateway
cargo build --release --bin wasmbed-gateway-simple

# Run with custom TLS
./target/release/wasmbed-gateway-simple \
  --bind-addr 0.0.0.0:4423 \
  --private-key certs/server-key-pkcs8.pem \
  --certificate certs/server-cert.pem \
  --client-ca certs/ca-cert.pem \
  --namespace wasmbed \
  --pod-namespace wasmbed \
  --pod-name test-gateway
```

### Full Gateway

The `wasmbed-gateway` binary provides complete Kubernetes integration while using the custom TLS library for secure communication.

## Examples

### Custom TLS Example

```bash
# Run the custom TLS example
cargo run --example custom-tls-example -p wasmbed-tls-utils
```

This example demonstrates:
- Server and client TLS connection
- Certificate parsing and validation
- Secure communication between components

### TLS Utils Example

```bash
# Run the TLS utils example
cargo run --example tls-utils-example -p wasmbed-tls-utils
```

This example demonstrates:
- Certificate parsing
- Key format conversion
- Certificate validation

## Dependencies

### Core Dependencies

- **`anyhow`**: Error handling
- **`rustls-pki-types`**: PKI type definitions
- **`x509-parser`**: X.509 certificate parsing
- **`x509-cert`**: X.509 certificate generation
- **`pem`**: PEM format parsing
- **`ed25519-dalek`**: Ed25519 cryptographic operations
- **`rand`**: Random number generation

### Async Dependencies

- **`tokio`**: Async runtime
- **`tokio-util`**: Async utilities

### Protocol Dependencies

- **`wasmbed-protocol`**: Protocol message types

## Testing

### Unit Tests

```bash
# Run all tests
cargo test -p wasmbed-tls-utils

# Run specific test
cargo test -p wasmbed-tls-utils test_parse_private_key
```

### Integration Tests

```bash
# Test with real certificates
cargo run --example custom-tls-example -p wasmbed-tls-utils

# Test gateway integration
cargo build --release --bin wasmbed-gateway-simple
./target/release/wasmbed-gateway-simple --help
```

## Security Considerations

### Certificate Management

- Always validate certificates before use
- Use proper certificate chains
- Implement certificate rotation
- Store private keys securely

### Key Management

- Generate keys using cryptographically secure random number generators
- Use appropriate key sizes (Ed25519 recommended)
- Implement proper key storage and access control
- Consider hardware security modules for production

### TLS Configuration

- Use TLS 1.3 for maximum security
- Implement proper cipher suite selection
- Configure appropriate certificate validation
- Monitor for security vulnerabilities

## Performance

### Optimization Tips

- Use zero-copy operations where possible
- Implement connection pooling
- Cache parsed certificates
- Use async I/O for better concurrency

### Resource Usage

- Memory: Optimized for IoT device constraints
- CPU: Efficient cryptographic operations
- Network: Minimal protocol overhead
- Storage: Compact certificate storage

## Troubleshooting

### Common Issues

1. **Certificate Parsing Errors**
   - Ensure certificates are in valid PEM/DER format
   - Check certificate chain completeness
   - Verify private key format compatibility

2. **TLS Handshake Failures**
   - Verify certificate validity
   - Check certificate chain trust
   - Ensure proper cipher suite support

3. **Performance Issues**
   - Monitor memory usage
   - Check CPU utilization
   - Verify network latency

### Debug Information

Enable debug logging to troubleshoot issues:

```rust
use log::Level;
use tracing_subscriber::FmtSubscriber;

let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
tracing::subscriber::set_global_default(subscriber)?;
```

## Future Enhancements

### Planned Features

- **TLS 1.3 Full Support**: Complete TLS 1.3 implementation
- **Hardware Security**: TPM and HSM integration
- **Performance Optimization**: Further performance improvements
- **Protocol Extensions**: Custom protocol extensions
- **Monitoring**: Built-in metrics and monitoring

### Contributing

Contributions to the custom TLS library are welcome! Please:

1. Follow Rust coding standards
2. Add comprehensive tests
3. Update documentation
4. Ensure backward compatibility
5. Consider IoT device constraints

## License

This custom TLS library is part of the Wasmbed platform and is released under the [AGPL-3.0](LICENSE) license.
