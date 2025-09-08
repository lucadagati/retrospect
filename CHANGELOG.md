# Changelog

All notable changes to the Wasmbed platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Custom TLS Library**: Complete replacement of rustls with custom TLS implementation
- **wasmbed-tls-utils crate**: New crate providing custom TLS functionality
- **TlsUtils**: Main utility struct for certificate and key operations
- **TlsServer**: Custom TLS server implementation
- **TlsClient**: Custom TLS client implementation
- **TlsConnection**: TLS connection wrapper with async I/O support
- **ServerIdentity**: Server identity management for compatibility
- **MessageContext**: Message handling context with protocol support
- **Certificate Parsing**: Support for PEM and DER formats
- **Key Format Support**: PKCS8 and RSA private key formats
- **X.509 Validation**: Complete X.509 certificate parsing and validation
- **Ed25519 Support**: Modern elliptic curve digital signatures
- **AES-256-GCM**: High-performance authenticated encryption
- **wasmbed-gateway-simple**: Simplified gateway using only custom TLS library
- **Custom TLS Examples**: Examples demonstrating TLS library usage
- **Comprehensive Documentation**: Complete documentation for custom TLS library
- **Enhanced Deployment Script**: Improved deploy-complete.sh with better error handling
- **Kubeconfig Management**: Automatic kubeconfig certificate extraction and management
- **Troubleshooting Script**: Comprehensive diagnostic and troubleshooting tool
- **Python3 Integration**: Python3 scripts for certificate management
- **Controller Integration**: Complete Kubernetes controller deployment
- **RBAC Configuration**: Proper RBAC setup for all components
- **TLS Secret Management**: Correct TLS secret naming and management
- **Deployment Verification**: Comprehensive deployment verification and testing
- **Diagnostic Reporting**: Automated diagnostic report generation
- **Network Connectivity Testing**: Built-in network connectivity verification
- **Docker Image Management**: Enhanced Docker image building and import
- **Error Recovery**: Automatic error recovery and pod restart capabilities

### Changed
- **Gateway TLS Implementation**: Replaced rustls with custom TLS library
- **Certificate Handling**: Enhanced certificate parsing and validation
- **Security Architecture**: Improved security with custom TLS implementation
- **Performance**: Optimized TLS operations for IoT devices
- **Memory Usage**: Reduced memory footprint with custom implementation
- **Dependencies**: Removed rustls dependency from gateway
- **Deployment Process**: Streamlined deployment with better error handling
- **Kubeconfig Handling**: External certificate file management
- **Secret Naming**: Standardized secret naming convention
- **Pod Management**: Enhanced pod health checking and restart logic
- **Service Configuration**: Improved service configuration and connectivity
- **Documentation Structure**: Enhanced documentation with troubleshooting guides
- **Script Organization**: Better organized and more robust deployment scripts

### Fixed
- **Certificate Parsing**: Fixed private key parsing issues
- **TLS Handshake**: Resolved TLS handshake failures
- **Gateway Startup**: Fixed gateway startup errors
- **Certificate Format Compatibility**: Support for multiple certificate formats
- **Key Format Conversion**: Proper handling of different key formats
- **Kubeconfig Errors**: Fixed "file name too long" errors in kubeconfig
- **Secret Mounting**: Resolved certificate secret mounting issues
- **Pod Crash Issues**: Fixed pod crash loop issues
- **Controller Deployment**: Resolved controller deployment problems
- **RBAC Permissions**: Fixed RBAC permission issues
- **Service Connectivity**: Resolved service connectivity problems
- **Certificate Validation**: Fixed certificate validation errors
- **Docker Image Import**: Resolved Docker image import issues
- **Namespace Management**: Fixed namespace creation and management
- **Resource Dependencies**: Resolved resource dependency issues

### Security
- **Enhanced Security**: Custom TLS implementation provides better security control
- **Memory Safety**: Rust's memory safety guarantees in TLS operations
- **Certificate Validation**: Improved certificate validation and trust chain handling
- **Key Management**: Better key generation and management
- **Encryption**: Stronger encryption algorithms and key exchange
- **Secret Management**: Enhanced Kubernetes secret management
- **RBAC Security**: Improved RBAC configuration and security
- **Network Security**: Enhanced network security and connectivity
- **Certificate Rotation**: Better certificate rotation and management
- **Access Control**: Improved access control and permissions

## [0.1.0] - 2025-01-08

### Added
- **Initial Release**: Complete Wasmbed platform implementation
- **Kubernetes Integration**: Full Kubernetes orchestration support
- **CRDs**: Device and Application Custom Resource Definitions
- **Gateway**: Wasmbed gateway with HTTP API and TLS support
- **Controller**: Kubernetes controller for resource management
- **MCU Support**: ESP32 and RISC-V device support
- **WASM Runtime**: WebAssembly runtime for edge devices
- **microROS Integration**: microROS bridge for PX4 communication
- **Security**: TLS 1.3, Ed25519 signatures, AES-256-GCM encryption
- **Monitoring**: Comprehensive monitoring and alerting
- **Testing**: Complete test suite and validation
- **Documentation**: Extensive documentation and examples
- **Scripts**: Automated deployment and management scripts

### Features
- **Multi-Platform**: Support for multiple MCU architectures
- **High Performance**: Optimized for IoT edge devices
- **Easy Deployment**: Automated setup and configuration
- **Comprehensive Testing**: End-to-end testing framework
- **Security First**: Security-first design approach
- **Extensible**: Modular and extensible architecture

## [0.0.1] - 2025-01-01

### Added
- **Project Initialization**: Initial project setup
- **Basic Structure**: Core project structure and organization
- **Dependencies**: Initial dependency management
- **Documentation**: Basic documentation framework
- **License**: AGPL-3.0 license implementation
