# Wasmbed Technical Documentation

Welcome to the comprehensive technical documentation for the Wasmbed IoT Device Management Platform.

## 📚 Documentation Structure

### 🏗️ Architecture & Design
- **[System Architecture](architecture/system-overview.md)** - High-level system design and components
- **[Protocol Design](architecture/protocol-design.md)** - Detailed communication protocol specification
- **[Security Architecture](architecture/security.md)** - Security model and implementation
- **[Data Flow](architecture/data-flow.md)** - End-to-end data flow diagrams

### 🔧 Implementation Guides
- **[Gateway Implementation](implementation/gateway.md)** - MPU Gateway detailed implementation
- **[Controller Implementation](implementation/controller.md)** - Kubernetes controller implementation
- **[MCU Firmware](implementation/mcu-firmware.md)** - RISC-V firmware implementation
- **[Protocol Implementation](implementation/protocol.md)** - Communication protocol implementation

### 📋 API Reference
- **[Kubernetes CRDs](api/crds.md)** - Custom Resource Definitions reference
- **[Gateway API](api/gateway-api.md)** - Gateway REST API reference
- **[Protocol Messages](api/protocol-messages.md)** - Protocol message formats
- **[Controller API](api/controller-api.md)** - Controller health and metrics endpoints

### 🚀 Deployment & Operations
- **[Installation Guide](deployment/installation.md)** - Complete installation instructions
- **[Configuration](deployment/configuration.md)** - System configuration options
- **[Monitoring](deployment/monitoring.md)** - Monitoring and observability
- **[Troubleshooting](deployment/troubleshooting.md)** - Common issues and solutions

### 🧪 Development & Testing
- **[Development Setup](development/setup.md)** - Development environment setup
- **[Testing Guide](development/testing.md)** - Testing strategies and tools
- **[Debugging](development/debugging.md)** - Debugging techniques and tools
- **[Performance](development/performance.md)** - Performance optimization guide

### 📊 Specifications
- **[Protocol Specification](specifications/protocol.md)** - Detailed protocol specification
- **[Security Specification](specifications/security.md)** - Security requirements and implementation
- **[Performance Specification](specifications/performance.md)** - Performance requirements
- **[Compatibility](specifications/compatibility.md)** - Compatibility matrix

## 🎯 Quick Start

### For Developers
1. Read [System Architecture](architecture/system-overview.md) for high-level understanding
2. Follow [Development Setup](development/setup.md) to set up your environment
3. Review [Gateway Implementation](implementation/gateway.md) for core concepts
4. Check [API Reference](api/crds.md) for detailed interfaces

### For Operators
1. Follow [Installation Guide](deployment/installation.md) for deployment
2. Review [Configuration](deployment/configuration.md) for system setup
3. Check [Monitoring](deployment/monitoring.md) for operational insights
4. Keep [Troubleshooting](deployment/troubleshooting.md) handy for issues

### For Architects
1. Start with [System Architecture](architecture/system-overview.md)
2. Review [Protocol Design](architecture/protocol-design.md) for communication
3. Check [Security Architecture](architecture/security.md) for security model
4. Review [Performance Specification](specifications/performance.md) for requirements

## 📖 Documentation Standards

### Code Examples
All code examples are tested and verified to work with the current version.

### Diagrams
- System diagrams use Mermaid syntax
- Protocol diagrams use PlantUML
- Architecture diagrams use draw.io format

### Version Information
- All documentation is current as of Wasmbed v0.1.0
- Compatible with Rust 1.88+ and Kubernetes 1.33+
- Tested with k3d v5.6.0 and QEMU 8.0+

## 🤝 Contributing to Documentation

### Guidelines
1. **Accuracy**: All technical details must be verified
2. **Completeness**: Include all necessary steps and context
3. **Clarity**: Use clear, concise language
4. **Examples**: Provide working code examples
5. **Testing**: Verify all examples work

### Documentation Workflow
1. Create feature branch: `git checkout -b docs/feature-name`
2. Update relevant documentation files
3. Test all code examples
4. Submit pull request with detailed description
5. Review and merge

## 📞 Support

For questions about this documentation:
- **Issues**: [GitHub Issues](https://github.com/your-org/wasmbed/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/wasmbed/discussions)
- **Wiki**: [Documentation Wiki](https://github.com/your-org/wasmbed/wiki)

---

**Last Updated**: September 2024  
**Version**: Wasmbed v0.1.0  
**Maintainer**: Wasmbed Development Team
