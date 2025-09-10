# Documentation Index

## Overview

This document provides a comprehensive index of all documentation available in the Wasmbed platform, organized by category and purpose.

## Quick Start

- [README](../README.md) - Main project overview and quick start guide
- [Development Setup](development/setup.md) - Complete development environment setup
- [Deployment Guide](deployment/deployment-guide.md) - Production deployment instructions

## Architecture Documentation

### System Architecture
- [System Overview](architecture/system-overview.md) - Complete system architecture documentation
- [Architecture Diagrams](architecture/diagrams.md) - Comprehensive Mermaid diagrams
- [Communication Protocols](architecture/communication-protocols.md) - TLS, CBOR, and DDS protocols
- [Security Architecture](architecture/security-architecture.md) - Security design and implementation

### Integration Architecture
- [PX4 Integration](integration/px4-integration.md) - PX4 autopilot integration
- [microROS Integration](integration/microros-integration.md) - microROS bridge implementation
- [FastDDS Integration](integration/fastdds-integration.md) - FastDDS middleware integration

## Implementation Documentation

### Core Components
- [Technical Choices](implementation/technical-choices.md) - Implementation choices and library documentation
- [Problems Resolved](implementation/problems-resolved.md) - Problems encountered and solutions
- [Workflow Implementation](workflows/implementation.md) - Complete workflow documentation

### Development
- [Development Setup](development/setup.md) - Development environment setup
- [Contributing Guidelines](development/contributing.md) - Contribution guidelines
- [Testing Guidelines](development/testing.md) - Testing procedures and guidelines

## API Documentation

### API Reference
- [API Reference](api/api-reference.md) - Complete REST API documentation
- [Custom Resource Definitions](api/crds.md) - Kubernetes CRD documentation

## Security Documentation

### Security Overview
- [Security Overview](security/security-overview.md) - Comprehensive security documentation

## Deployment Documentation

### Deployment and Operations
- [Deployment Guide](deployment/deployment-guide.md) - Step-by-step deployment instructions
- [Configuration Guide](deployment/configuration.md) - Platform configuration options
- [Troubleshooting Guide](deployment/troubleshooting.md) - Common issues and solutions

## Problems and Solutions

### Current Issues
- [Known Issues](problems/known-issues.md) - Current known issues and workarounds
- [Missing Implementations](problems/missing-implementations.md) - Critical missing features

## Documentation Structure

```
docs/
├── README.md                          # This file
├── architecture/
│   ├── system-overview.md            # System architecture
│   ├── communication-protocols.md    # Communication protocols
│   └── security-architecture.md      # Security architecture
├── implementation/
│   ├── technical-choices.md          # Implementation choices
│   └── problems-resolved.md          # Problems and solutions
├── workflows/
│   └── implementation.md             # Workflow documentation
├── api/
│   ├── api-reference.md              # REST API documentation
│   └── crds.md                       # CRD documentation
├── security/
│   └── security-overview.md          # Security documentation
├── deployment/
│   ├── deployment-guide.md           # Deployment instructions
│   ├── configuration.md              # Configuration guide
│   └── troubleshooting.md            # Troubleshooting guide
├── integration/
│   ├── px4-integration.md            # PX4 integration
│   ├── microros-integration.md       # microROS integration
│   └── fastdds-integration.md        # FastDDS integration
├── development/
│   ├── setup.md                      # Development setup
│   ├── contributing.md               # Contributing guidelines
│   └── testing.md                    # Testing guidelines
└── problems/
    ├── known-issues.md               # Known issues
    └── missing-implementations.md    # Missing implementations
```

## Documentation Standards

### Writing Guidelines
- Use clear, concise language
- Provide comprehensive examples
- Include code snippets where appropriate
- Maintain consistent formatting
- Update documentation with code changes

### Content Guidelines
- Start with overview and purpose
- Provide step-by-step instructions
- Include troubleshooting information
- Document all public APIs
- Explain design decisions

### Maintenance Guidelines
- Keep documentation up to date
- Review documentation regularly
- Update examples and code snippets
- Remove obsolete information
- Add new features and changes

## Getting Help

### Documentation Issues
If you find issues with the documentation:
1. Check if the issue is already documented
2. Create an issue in the project repository
3. Provide specific details about the problem
4. Suggest improvements if possible

### Contributing Documentation
To contribute to the documentation:
1. Follow the writing guidelines
2. Use the established structure
3. Test all examples and code snippets
4. Submit a pull request with changes
5. Include a description of changes

### Support Channels
- GitHub Issues: For bug reports and feature requests
- Documentation Issues: For documentation problems
- Community Forum: For general questions and discussions

## Version Information

**Documentation Version**: 1.0.0
**Last Updated**: 2024-01-01
**Platform Version**: 1.0.0

## License

This documentation is licensed under the same license as the Wasmbed platform. See the main project LICENSE file for details.
