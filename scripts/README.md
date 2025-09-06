# Wasmbed Scripts

This directory contains all scripts for managing the Wasmbed platform, organized by category.

## Structure

### Setup & Deployment (`setup/`)
Scripts for configuring and deploying the platform:
- `01-setup-platform.sh` - Initial platform setup
- `02-deploy-to-k8s.sh` - Deploy to Kubernetes
- `03-build-images.sh` - Build Docker images
- `04-cleanup-platform.sh` - Platform cleanup
- `20-dev-environment.sh` - Development environment setup

### Testing (`testing/`)
Scripts for running tests:
- `05-test-complete.sh` - Complete system tests
- `06-test-unit.sh` - Unit tests
- `07-test-integration.sh` - Integration tests
- `08-test-end-to-end.sh` - End-to-end tests
- `09-test-security.sh` - Security tests
- `run-all-tests.sh` - Run all tests

### Security (`security/`)
Scripts for security management:
- `10-security-scan.sh` - Security scanning
- `11-security-hardening.sh` - Security hardening
- `12-certificate-rotate.sh` - Certificate rotation
- `generate-certs.sh` - Certificate generation

### Monitoring & Maintenance (`monitoring/`)
Scripts for monitoring and maintenance:
- `13-monitor-platform.sh` - Platform monitoring
- `14-show-logs.sh` - Log viewing
- `15-show-status.sh` - System status
- `16-health-check.sh` - Health checks
- `17-backup-platform.sh` - Platform backup
- `18-restore-platform.sh` - Platform restore
- `19-disaster-recovery.sh` - Disaster recovery

### Development (`development/`)
Scripts for development:
- `21-build-all.sh` - Complete build
- `22-lint-code.sh` - Code linting
- `23-generate-docs.sh` - Documentation generation

## Quick Usage

### Initial Setup
```bash
./scripts/setup/01-setup-platform.sh
```

### Complete Testing
```bash
./scripts/testing/run-all-tests.sh
```

### Deploy to Kubernetes
```bash
./scripts/setup/02-deploy-to-k8s.sh
```

### Monitoring
```bash
./scripts/monitoring/13-monitor-platform.sh
```

## Prerequisites

- Rust 1.88+
- Docker
- kubectl
- k3d (for local development)
- Nix (optional, for development environment)

## Configuration

Scripts use environment variables for configuration. See `docs/development/setup.md` for details.