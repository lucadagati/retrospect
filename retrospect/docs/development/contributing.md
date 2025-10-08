# Development Setup and Contributing Guidelines

## Overview

This document provides comprehensive guidance for setting up a development environment and contributing to the Wasmbed platform.

## Development Environment Setup

### Prerequisites

**System Requirements**:
- Linux (Ubuntu 20.04+ recommended)
- macOS (10.15+)
- Windows 10+ (with WSL2)

**Required Software**:
- Rust toolchain 1.70+
- Docker 20.10+
- Docker Compose 2.0+
- Git 2.30+
- Make 4.0+

### Installation Steps

#### 1. Install Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required components
rustup component add rustfmt clippy
rustup target add riscv32imac-unknown-none-elf
rustup target add thumbv7em-none-eabihf
rustup target add xtensa-esp32-espidf

# Verify installation
rustc --version
cargo --version
```

#### 2. Install Docker

**Ubuntu/Debian**:
```bash
# Install Docker
sudo apt-get update
sudo apt-get install -y docker.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker
```

**macOS**:
```bash
# Install Docker Desktop
brew install --cask docker
```

**Windows**:
```bash
# Install Docker Desktop
# Download from https://www.docker.com/products/docker-desktop
```

#### 3. Install QEMU Emulators

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install -y \
    qemu-system-riscv32 \
    qemu-system-arm \
    qemu-system-xtensa \
    qemu-utils
```

**macOS**:
```bash
brew install qemu
```

**Windows**:
```bash
# Install QEMU via Chocolatey
choco install qemu
```

#### 4. Install Kubernetes (k3d)

```bash
# Install k3d
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Verify installation
k3d version
```

#### 5. Install Additional Tools

```bash
# Install additional development tools
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    ninja-build \
    clang \
    llvm-dev \
    libclang-dev

# Install Rust development tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-audit
cargo install cargo-deny
```

### Project Setup

#### 1. Clone Repository

```bash
# Clone the repository
git clone <repository-url>
cd retrospect

# Verify project structure
ls -la
```

#### 2. Build Project

```bash
# Build all components
cargo build --release

# Build specific components
cargo build --release -p wasmbed-gateway
cargo build --release -p wasmbed-k8s-controller
cargo build --release -p wasmbed-qemu-serial-bridge
```

#### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test -p wasmbed-gateway
cargo test -p wasmbed-k8s-controller
cargo test -p wasmbed-qemu-serial-bridge
```

#### 4. Setup Development Environment

```bash
# Setup pre-commit hooks
cargo install pre-commit
pre-commit install

# Setup development scripts
chmod +x scripts/*.sh
```

## Development Workflow

### Code Organization

**Project Structure**:
```
retrospect/
├── crates/                 # Rust crates
│   ├── wasmbed-gateway/    # Gateway server
│   ├── wasmbed-k8s-controller/ # Kubernetes controller
│   ├── wasmbed-qemu-serial-bridge/ # QEMU bridge
│   ├── wasmbed-firmware-*/ # Device firmware
│   └── wasmbed-*/         # Other components
├── docs/                  # Documentation
├── resources/             # Kubernetes resources
├── scripts/              # Development scripts
└── apps/                 # Application examples
```

**Crate Organization**:
- Each major component is a separate crate
- Shared functionality in common crates
- Clear dependency relationships
- Minimal external dependencies

### Coding Standards

#### Rust Code Style

**Formatting**:
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

**Linting**:
```bash
# Run clippy
cargo clippy --all-targets --all-features

# Run clippy with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings
```

**Code Style Guidelines**:
- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions
- Use meaningful variable and function names
- Add comprehensive documentation
- Implement proper error handling

#### Documentation Standards

**Code Documentation**:
```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `param1` - Description of parameter 1
/// * `param2` - Description of parameter 2
///
/// # Returns
///
/// Returns a Result containing the success value or an error
///
/// # Examples
///
/// ```
/// let result = example_function("param1", "param2")?;
/// ```
pub fn example_function(param1: &str, param2: &str) -> Result<String, Error> {
    // Implementation
}
```

**API Documentation**:
- Document all public APIs
- Include examples for complex functions
- Document error conditions
- Keep documentation up to date

### Testing Strategy

#### Unit Testing

**Test Organization**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_success() {
        // Test successful case
    }
    
    #[test]
    fn test_function_error() {
        // Test error case
    }
    
    #[test]
    fn test_function_edge_case() {
        // Test edge cases
    }
}
```

**Testing Guidelines**:
- Write tests for all public functions
- Test both success and error cases
- Use descriptive test names
- Keep tests simple and focused
- Mock external dependencies

#### Integration Testing

**Test Structure**:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integration_workflow() {
        // Test complete workflow
    }
}
```

**Integration Test Guidelines**:
- Test complete workflows
- Use real dependencies when possible
- Test error handling and recovery
- Test performance characteristics

#### End-to-End Testing

**E2E Test Setup**:
```bash
# Run end-to-end tests
./scripts/app.sh test
```

**E2E Test Guidelines**:
- Test complete system functionality
- Use realistic test data
- Test error scenarios
- Validate performance requirements

### Debugging and Development Tools

#### Debugging Tools

**Rust Debugging**:
```bash
# Debug with gdb
cargo build --release
gdb target/release/wasmbed-gateway

# Debug with lldb (macOS)
cargo build --release
lldb target/release/wasmbed-gateway
```

**Logging and Tracing**:
```rust
use log::{debug, info, warn, error};

// Use appropriate log levels
debug!("Debug information: {}", value);
info!("Information: {}", value);
warn!("Warning: {}", value);
error!("Error: {}", value);
```

#### Development Tools

**Cargo Watch**:
```bash
# Watch for changes and rebuild
cargo watch -x build

# Watch for changes and run tests
cargo watch -x test
```

**Cargo Expand**:
```bash
# Expand macros
cargo expand --lib
```

**Cargo Audit**:
```bash
# Check for security vulnerabilities
cargo audit
```

## Contributing Guidelines

### Contribution Process

#### 1. Fork and Clone

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/your-username/retrospect.git
cd retrospect

# Add upstream remote
git remote add upstream https://github.com/original-owner/retrospect.git
```

#### 2. Create Feature Branch

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes
# ... edit files ...

# Commit changes
git add .
git commit -m "Add your feature description"
```

#### 3. Test Changes

```bash
# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Run formatting check
cargo fmt --check
```

#### 4. Submit Pull Request

```bash
# Push changes
git push origin feature/your-feature-name

# Create pull request on GitHub
```

### Pull Request Guidelines

#### PR Requirements

**Code Quality**:
- All tests must pass
- Code must be formatted with `cargo fmt`
- Code must pass `cargo clippy` checks
- Code must have appropriate documentation

**Documentation**:
- Update relevant documentation
- Add tests for new functionality
- Update changelog if applicable

**Testing**:
- Add unit tests for new functionality
- Add integration tests if applicable
- Test error cases and edge conditions

#### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

### Code Review Process

#### Review Guidelines

**Code Review Checklist**:
- [ ] Code follows project style guidelines
- [ ] Code is well-documented
- [ ] Tests are comprehensive
- [ ] Error handling is appropriate
- [ ] Performance considerations addressed
- [ ] Security implications considered

**Review Process**:
1. Automated checks must pass
2. At least one reviewer approval required
3. Address all review comments
4. Update documentation if needed
5. Merge after approval

### Issue Reporting

#### Bug Reports

**Bug Report Template**:
```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Ubuntu 20.04]
- Rust version: [e.g., 1.70.0]
- Docker version: [e.g., 20.10.0]

## Additional Context
Any additional information
```

#### Feature Requests

**Feature Request Template**:
```markdown
## Feature Description
Clear description of the feature

## Use Case
Why is this feature needed

## Proposed Solution
How should this be implemented

## Alternatives Considered
Other approaches considered

## Additional Context
Any additional information
```

### Development Best Practices

#### Code Quality

**Best Practices**:
- Write clean, readable code
- Use meaningful names
- Implement proper error handling
- Add comprehensive tests
- Document public APIs

**Performance**:
- Profile code for performance bottlenecks
- Use appropriate data structures
- Minimize allocations
- Consider memory usage

**Security**:
- Validate all inputs
- Use secure coding practices
- Handle sensitive data appropriately
- Follow security guidelines

#### Testing Best Practices

**Testing Guidelines**:
- Write tests early and often
- Test both success and failure cases
- Use realistic test data
- Mock external dependencies
- Test edge cases and error conditions

**Test Organization**:
- Group related tests together
- Use descriptive test names
- Keep tests simple and focused
- Avoid test interdependencies

#### Documentation Best Practices

**Documentation Guidelines**:
- Document all public APIs
- Include examples for complex functions
- Keep documentation up to date
- Use clear, concise language
- Include error conditions

**Documentation Types**:
- API documentation
- Architecture documentation
- User guides
- Developer guides
- Troubleshooting guides

## Release Process

### Version Management

**Semantic Versioning**:
- Major version: Breaking changes
- Minor version: New features
- Patch version: Bug fixes

**Version Bumping**:
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Create release tag
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

### Release Checklist

**Pre-Release**:
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Version numbers updated
- [ ] Security audit completed

**Release**:
- [ ] Create release tag
- [ ] Build release artifacts
- [ ] Publish to package registry
- [ ] Update documentation
- [ ] Announce release

**Post-Release**:
- [ ] Monitor for issues
- [ ] Update issue tracker
- [ ] Plan next release
- [ ] Gather feedback
