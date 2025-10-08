# Deployment Guide and Troubleshooting

## Overview

This document provides comprehensive deployment instructions and troubleshooting guidance for the Wasmbed platform.

## Prerequisites

### System Requirements

**Minimum Requirements**:
- CPU: 4 cores
- RAM: 8GB
- Storage: 50GB free space
- Network: Internet connection for package downloads

**Recommended Requirements**:
- CPU: 8 cores
- RAM: 16GB
- Storage: 100GB free space
- Network: Stable internet connection

### Software Dependencies

**Required Software**:
- Docker 20.10+
- Docker Compose 2.0+
- Kubernetes cluster (k3d recommended)
- QEMU system emulators
- Rust toolchain 1.70+

**QEMU Emulators**:
```bash
# Install QEMU emulators
sudo apt-get update
sudo apt-get install -y \
    qemu-system-riscv32 \
    qemu-system-arm \
    qemu-system-xtensa \
    qemu-utils
```

**Rust Toolchain**:
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

## Deployment Instructions

### 1. Platform Deployment

**Complete Platform Deployment**:
```bash
# Deploy the complete platform
./scripts/deploy.sh
```

**Deployment Process**:
1. Create k3d Kubernetes cluster
2. Build Docker images for all components
3. Generate TLS certificates
4. Deploy Kubernetes resources
5. Verify deployment status

**Verification**:
```bash
# Check deployment status
kubectl get pods -n wasmbed
kubectl get services -n wasmbed
kubectl get crds | grep wasmbed
```

### 2. Application Testing

**Run Complete Tests**:
```bash
# Run all application tests
./scripts/app.sh test
```

**Test Components**:
- Unit tests for all Rust crates
- Kubernetes deployment tests
- API endpoint tests
- CRD functionality tests
- QEMU emulation tests

### 3. System Monitoring

**Monitor System Status**:
```bash
# Check system status
./scripts/monitor.sh status
```

**Monitoring Components**:
- Kubernetes cluster health
- Gateway server status
- Device connections
- Application deployments
- Resource utilization

### 4. Cleanup

**Complete System Cleanup**:
```bash
# Clean up entire system
./scripts/cleanup.sh
```

**Cleanup Process**:
- Remove Kubernetes resources
- Delete Docker images
- Clean up TLS certificates
- Remove QEMU files
- Clean build artifacts

## Configuration

### Environment Variables

**Gateway Configuration**:
```bash
# Gateway environment variables
WASMBED_GATEWAY_HOST=0.0.0.0
WASMBED_GATEWAY_PORT=8080
WASMBED_GATEWAY_TLS_PORT=4423
WASMBED_GATEWAY_LOG_LEVEL=info
WASMBED_GATEWAY_MAX_DEVICES=100
```

**Kubernetes Configuration**:
```bash
# Kubernetes environment variables
KUBECONFIG=~/.kube/config
KUBERNETES_NAMESPACE=wasmbed
KUBERNETES_CONTEXT=k3d-wasmbed
```

### Configuration Files

**Gateway ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-gateway-config
  namespace: wasmbed
data:
  pairing_mode: "false"
  pairing_timeout_seconds: "300"
  heartbeat_timeout_seconds: "30"
  max_devices: "100"
  log_level: "info"
```

**TLS Configuration**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: wasmbed-tls-secret-rsa
  namespace: wasmbed
type: kubernetes.io/tls
data:
  tls.key: <base64-encoded-private-key>
  tls.crt: <base64-encoded-certificate>
```

## Troubleshooting

### Common Issues

#### 1. Kubernetes Connection Issues

**Symptoms**:
- Gateway unable to connect to Kubernetes API
- Controller reconciliation failing
- Device status updates not working

**Diagnosis**:
```bash
# Check Kubernetes cluster status
kubectl cluster-info
kubectl get nodes
kubectl get pods -n wasmbed
```

**Solutions**:
```bash
# Restart k3d cluster
k3d cluster delete wasmbed
k3d cluster create wasmbed --port "8080:80@loadbalancer" --port "4423:443@loadbalancer"

# Check service account permissions
kubectl get serviceaccount wasmbed-gateway -n wasmbed
kubectl describe clusterrolebinding wasmbed-gateway
```

#### 2. TLS Certificate Issues

**Symptoms**:
- TLS handshake failures
- Certificate validation errors
- Gateway failing to start

**Diagnosis**:
```bash
# Check certificate status
kubectl get secret wasmbed-tls-secret-rsa -n wasmbed
kubectl describe secret wasmbed-tls-secret-rsa -n wasmbed

# Test TLS connection
openssl s_client -connect localhost:4423 -servername wasmbed-gateway
```

**Solutions**:
```bash
# Regenerate certificates
./scripts/security/generate-certs.sh

# Recreate TLS secret
kubectl delete secret wasmbed-tls-secret-rsa -n wasmbed
kubectl create secret tls wasmbed-tls-secret-rsa \
  --cert=server-cert.pem \
  --key=server-key.pem \
  -n wasmbed
```

#### 3. QEMU Emulation Issues

**Symptoms**:
- QEMU devices not starting
- Firmware build failures
- Serial communication errors

**Diagnosis**:
```bash
# Check QEMU installation
qemu-system-riscv32 --version
qemu-system-arm --version
qemu-system-xtensa --version

# Check firmware build
cargo build --release --target riscv32imac-unknown-none-elf
```

**Solutions**:
```bash
# Install missing QEMU packages
sudo apt-get install -y qemu-system-riscv32 qemu-system-arm qemu-system-xtensa

# Fix firmware build issues
cargo clean
cargo build --release --target riscv32imac-unknown-none-elf
```

#### 4. Docker Build Issues

**Symptoms**:
- Docker images failing to build
- Build timeouts
- Dependency resolution errors

**Diagnosis**:
```bash
# Check Docker status
docker version
docker info

# Check build logs
docker build -t wasmbed-gateway . 2>&1 | tee build.log
```

**Solutions**:
```bash
# Clean Docker cache
docker system prune -a

# Rebuild with verbose output
docker build --no-cache -t wasmbed-gateway .
```

#### 5. API Endpoint Issues

**Symptoms**:
- 404 errors for API endpoints
- Authentication failures
- CORS errors

**Diagnosis**:
```bash
# Test API endpoints
curl -v http://localhost:8080/health
curl -v http://localhost:8080/api/v1/admin/pairing-mode
```

**Solutions**:
```bash
# Check gateway logs
kubectl logs -f deployment/wasmbed-gateway -n wasmbed

# Restart gateway
kubectl rollout restart deployment/wasmbed-gateway -n wasmbed
```

### Advanced Troubleshooting

#### 1. Performance Issues

**Symptoms**:
- Slow response times
- High resource utilization
- Memory leaks

**Diagnosis**:
```bash
# Check resource usage
kubectl top pods -n wasmbed
kubectl top nodes

# Check memory usage
kubectl describe pod <pod-name> -n wasmbed
```

**Solutions**:
```bash
# Scale resources
kubectl scale deployment wasmbed-gateway --replicas=3 -n wasmbed

# Check resource limits
kubectl get deployment wasmbed-gateway -o yaml -n wasmbed
```

#### 2. Security Issues

**Symptoms**:
- Authentication failures
- Certificate errors
- Permission denied errors

**Diagnosis**:
```bash
# Check RBAC permissions
kubectl auth can-i get pods --as=system:serviceaccount:wasmbed:wasmbed-gateway -n wasmbed

# Check certificate validity
openssl x509 -in server-cert.pem -text -noout
```

**Solutions**:
```bash
# Fix RBAC permissions
kubectl apply -f resources/k8s/rbac/gateway-rbac.yaml

# Regenerate certificates
./scripts/security/generate-certs.sh
```

#### 3. Network Issues

**Symptoms**:
- Connection timeouts
- Network unreachable errors
- Port forwarding issues

**Diagnosis**:
```bash
# Check network connectivity
ping <gateway-ip>
telnet <gateway-ip> 8080
telnet <gateway-ip> 4423

# Check port forwarding
kubectl port-forward service/wasmbed-gateway-service 8080:8080 -n wasmbed
```

**Solutions**:
```bash
# Restart network services
sudo systemctl restart docker
sudo systemctl restart k3d

# Check firewall rules
sudo ufw status
sudo iptables -L
```

## Monitoring and Logging

### Log Collection

**Gateway Logs**:
```bash
# View gateway logs
kubectl logs -f deployment/wasmbed-gateway -n wasmbed

# View controller logs
kubectl logs -f deployment/wasmbed-k8s-controller -n wasmbed
```

**System Logs**:
```bash
# View system logs
journalctl -u docker
journalctl -u k3d
```

### Metrics Collection

**Kubernetes Metrics**:
```bash
# Check pod metrics
kubectl top pods -n wasmbed

# Check node metrics
kubectl top nodes
```

**Application Metrics**:
```bash
# Check application status
kubectl get applications -n wasmbed
kubectl get devices -n wasmbed
```

### Health Checks

**System Health**:
```bash
# Check system health
./scripts/monitor.sh health

# Check component health
kubectl get pods -n wasmbed
kubectl get services -n wasmbed
```

**API Health**:
```bash
# Check API health
curl http://localhost:8080/health

# Check TLS health
curl -k https://localhost:4423/health
```

## Backup and Recovery

### Backup Procedures

**Kubernetes Resources**:
```bash
# Backup all resources
kubectl get all -n wasmbed -o yaml > wasmbed-backup.yaml

# Backup CRDs
kubectl get crds -o yaml > crds-backup.yaml
```

**Configuration Backup**:
```bash
# Backup configuration
kubectl get configmaps -n wasmbed -o yaml > configmaps-backup.yaml
kubectl get secrets -n wasmbed -o yaml > secrets-backup.yaml
```

### Recovery Procedures

**Resource Recovery**:
```bash
# Restore resources
kubectl apply -f wasmbed-backup.yaml

# Restore CRDs
kubectl apply -f crds-backup.yaml
```

**Configuration Recovery**:
```bash
# Restore configuration
kubectl apply -f configmaps-backup.yaml
kubectl apply -f secrets-backup.yaml
```

## Security Considerations

### Certificate Management

**Certificate Renewal**:
```bash
# Renew certificates
./scripts/security/generate-certs.sh

# Update secrets
kubectl delete secret wasmbed-tls-secret-rsa -n wasmbed
kubectl create secret tls wasmbed-tls-secret-rsa \
  --cert=server-cert.pem \
  --key=server-key.pem \
  -n wasmbed
```

**Certificate Validation**:
```bash
# Validate certificates
openssl x509 -in server-cert.pem -text -noout
openssl verify -CAfile ca-cert.pem server-cert.pem
```

### Access Control

**RBAC Management**:
```bash
# Check permissions
kubectl auth can-i get pods --as=system:serviceaccount:wasmbed:wasmbed-gateway -n wasmbed

# Update permissions
kubectl apply -f resources/k8s/rbac/gateway-rbac.yaml
```

**Network Security**:
```bash
# Check network policies
kubectl get networkpolicies -n wasmbed

# Apply network policies
kubectl apply -f resources/k8s/network-policies/
```

## Performance Optimization

### Resource Optimization

**CPU Optimization**:
```bash
# Check CPU usage
kubectl top pods -n wasmbed

# Adjust CPU limits
kubectl patch deployment wasmbed-gateway -p '{"spec":{"template":{"spec":{"containers":[{"name":"wasmbed-gateway","resources":{"limits":{"cpu":"1000m"}}}]}}}}' -n wasmbed
```

**Memory Optimization**:
```bash
# Check memory usage
kubectl top pods -n wasmbed

# Adjust memory limits
kubectl patch deployment wasmbed-gateway -p '{"spec":{"template":{"spec":{"containers":[{"name":"wasmbed-gateway","resources":{"limits":{"memory":"1Gi"}}}]}}}}' -n wasmbed
```

### Network Optimization

**Connection Pooling**:
```bash
# Check connection settings
kubectl describe deployment wasmbed-gateway -n wasmbed

# Update connection settings
kubectl patch deployment wasmbed-gateway -p '{"spec":{"template":{"spec":{"containers":[{"name":"wasmbed-gateway","env":[{"name":"MAX_CONNECTIONS","value":"1000"}]}]}}}}' -n wasmbed
```

**Load Balancing**:
```bash
# Check service configuration
kubectl get service wasmbed-gateway-service -n wasmbed

# Update service configuration
kubectl patch service wasmbed-gateway-service -p '{"spec":{"type":"LoadBalancer"}}' -n wasmbed
```
