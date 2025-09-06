# Deployment and Troubleshooting Guide

## 🎯 Overview

This guide provides comprehensive instructions for deploying Wasmbed in various environments and troubleshooting common issues.

## 🚀 Installation Guide

### Prerequisites

#### System Requirements

```bash
# Operating System
- Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+)
- macOS (10.15+)
- Windows (WSL2 recommended)

# Hardware Requirements
- CPU: 2+ cores
- Memory: 4GB+ RAM
- Storage: 10GB+ free space
- Network: Internet connectivity for downloads
```

#### Required Software

```bash
# Core Dependencies
- Docker 20.10+
- kubectl 1.28+
- k3d 5.4+
- QEMU 6.0+ (for MCU emulation)
- Nix 2.8+ (for development environment)

# Optional Dependencies
- Helm 3.8+ (for advanced deployments)
- Prometheus (for monitoring)
- Grafana (for dashboards)
```

### Development Environment Setup

#### 1. Install Nix

```bash
# Install Nix package manager
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Verify installation
nix --version
```

#### 2. Clone Repository

```bash
# Clone Wasmbed repository
git clone https://github.com/your-org/wasmbed.git
cd wasmbed

# Enter development environment
nix develop
```

#### 3. Complete Setup

```bash
# Run complete setup script
./scripts/setup.sh

# Verify installation
./scripts/test.sh
```

### Production Environment Setup

#### 1. Kubernetes Cluster

```bash
# Create production cluster (example with k3s)
curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION=v1.28.0 sh -

# Configure kubectl
sudo cat /etc/rancher/k3s/k3s.yaml > ~/.kube/config
chmod 600 ~/.kube/config

# Verify cluster
kubectl cluster-info
```

#### 2. Install Wasmbed

```bash
# Create namespace
kubectl create namespace wasmbed

# Apply CRDs
kubectl apply -f resources/k8s/crds/

# Apply RBAC
kubectl apply -f resources/k8s/rbac/

# Deploy Gateway
kubectl apply -f resources/k8s/gateway/

# Deploy Controller
kubectl apply -f resources/k8s/controller/
```

#### 3. Verify Deployment

```bash
# Check all components
kubectl get all -n wasmbed

# Check CRDs
kubectl get crd | grep wasmbed

# Check Gateway logs
kubectl logs -n wasmbed deployment/wasmbed-gateway

# Check Controller logs
kubectl logs -n wasmbed deployment/wasmbed-k8s-controller
```

## 🔧 Configuration

### Environment Variables

#### Gateway Configuration

```yaml
# Gateway environment variables
env:
- name: WASMBED_GATEWAY_HOST
  value: "0.0.0.0"
- name: WASMBED_GATEWAY_PORT
  value: "8080"
- name: WASMBED_TLS_CERT_FILE
  value: "/etc/wasmbed/gateway.crt"
- name: WASMBED_TLS_KEY_FILE
  value: "/etc/wasmbed/gateway.key"
- name: WASMBED_TLS_CA_FILE
  value: "/etc/wasmbed/ca.crt"
- name: WASMBED_HEARTBEAT_INTERVAL
  value: "30"
- name: WASMBED_HEARTBEAT_TIMEOUT
  value: "90"
- name: RUST_LOG
  value: "info"
```

#### Controller Configuration

```yaml
# Controller environment variables
env:
- name: WASMBED_GATEWAY_URL
  value: "http://wasmbed-gateway:8080"
- name: WASMBED_CONTROLLER_RECONCILIATION_INTERVAL
  value: "30"
- name: WASMBED_CONTROLLER_MAX_RETRIES
  value: "3"
- name: WASMBED_CONTROLLER_BACKOFF_MULTIPLIER
  value: "2.0"
- name: RUST_LOG
  value: "info"
```

### TLS Certificate Management

#### Generate Development Certificates

```bash
# Generate CA certificate
cargo run -p wasmbed-cert-tool -- generate-ca \
  --common-name "Wasmbed Development CA" \
  --out-key resources/dev-certs/ca.key \
  --out-cert resources/dev-certs/ca.der

# Generate Gateway certificate
cargo run -p wasmbed-cert-tool -- issue-cert \
  --ca-key resources/dev-certs/ca.key \
  --ca-cert resources/dev-certs/ca.der \
  --common-name "wasmbed-gateway" \
  --out-key resources/dev-certs/gateway.key \
  --out-cert resources/dev-certs/gateway.der \
  server

# Generate MCU certificates
for i in {0..9}; do
  cargo run -p wasmbed-cert-tool -- issue-cert \
    --ca-key resources/dev-certs/ca.key \
    --ca-cert resources/dev-certs/ca.der \
    --common-name "mcu-$i" \
    --out-key resources/dev-certs/client-$i.key \
    --out-cert resources/dev-certs/client-$i.der \
    client
done
```

#### Production Certificate Setup

```bash
# Use Let's Encrypt for production
certbot certonly --standalone -d gateway.yourdomain.com

# Convert to DER format
openssl x509 -in /etc/letsencrypt/live/gateway.yourdomain.com/cert.pem \
  -outform DER -out gateway.crt

openssl rsa -in /etc/letsencrypt/live/gateway.yourdomain.com/privkey.pem \
  -outform DER -out gateway.key
```

## 🔍 Monitoring

### Health Checks

#### Gateway Health

```bash
# Port forward to Gateway
kubectl port-forward -n wasmbed service/wasmbed-gateway 8080:8080

# Check health endpoint
curl http://localhost:8080/health

# Check readiness
curl http://localhost:8080/ready

# Get metrics
curl http://localhost:8080/metrics
```

#### Controller Health

```bash
# Port forward to Controller
kubectl port-forward -n wasmbed deployment/wasmbed-k8s-controller 8080:8080

# Check health endpoint
curl http://localhost:8080/health

# Check readiness
curl http://localhost:8080/ready

# Get metrics
curl http://localhost:8080/metrics
```

### Logging

#### Enable Debug Logging

```bash
# Set debug log level
kubectl patch deployment wasmbed-gateway -n wasmbed --type='merge' \
  -p='{"spec":{"template":{"spec":{"containers":[{"name":"gateway","env":[{"name":"RUST_LOG","value":"debug"}]}]}}}}'

kubectl patch deployment wasmbed-k8s-controller -n wasmbed --type='merge' \
  -p='{"spec":{"template":{"spec":{"containers":[{"name":"controller","env":[{"name":"RUST_LOG","value":"debug"}]}]}}}}'
```

#### View Logs

```bash
# Gateway logs
kubectl logs -n wasmbed deployment/wasmbed-gateway -f

# Controller logs
kubectl logs -n wasmbed deployment/wasmbed-k8s-controller -f

# All Wasmbed logs
kubectl logs -n wasmbed -l app.kubernetes.io/part-of=wasmbed -f
```

### Metrics Collection

#### Prometheus Configuration

```yaml
# prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'wasmbed-gateway'
      static_configs:
      - targets: ['wasmbed-gateway:8080']
      metrics_path: /metrics
    - job_name: 'wasmbed-controller'
      static_configs:
      - targets: ['wasmbed-k8s-controller:8080']
      metrics_path: /metrics
```

#### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Wasmbed Dashboard",
    "panels": [
      {
        "title": "Connected Devices",
        "type": "stat",
        "targets": [
          {
            "expr": "wasmbed_devices_connected_total"
          }
        ]
      },
      {
        "title": "Application Deployments",
        "type": "stat",
        "targets": [
          {
            "expr": "wasmbed_applications_running_total"
          }
        ]
      }
    ]
  }
}
```

## 🛠️ Troubleshooting

### Common Issues

#### 1. Gateway Won't Start

**Symptoms:**
- Gateway pod in `CrashLoopBackOff`
- TLS certificate errors
- Port binding issues

**Diagnosis:**
```bash
# Check Gateway logs
kubectl logs -n wasmbed deployment/wasmbed-gateway

# Check TLS certificates
kubectl exec -n wasmbed deployment/wasmbed-gateway -- ls -la /etc/wasmbed/

# Check port binding
kubectl exec -n wasmbed deployment/wasmbed-gateway -- netstat -tlnp
```

**Solutions:**
```bash
# Fix TLS certificate permissions
kubectl exec -n wasmbed deployment/wasmbed-gateway -- chmod 600 /etc/wasmbed/*.key

# Regenerate certificates
./scripts/generate-certs.sh

# Check port conflicts
kubectl get svc -n wasmbed
```

#### 2. Controller Not Reconciling

**Symptoms:**
- Applications stuck in `Creating` phase
- No reconciliation logs
- Gateway connection errors

**Diagnosis:**
```bash
# Check Controller logs
kubectl logs -n wasmbed deployment/wasmbed-k8s-controller

# Check Gateway connectivity
kubectl exec -n wasmbed deployment/wasmbed-k8s-controller -- curl -v http://wasmbed-gateway:8080/health

# Check RBAC permissions
kubectl auth can-i get applications --as=system:serviceaccount:wasmbed:wasmbed-controller-sa
```

**Solutions:**
```bash
# Fix RBAC permissions
kubectl apply -f resources/k8s/rbac/

# Restart Controller
kubectl rollout restart deployment/wasmbed-k8s-controller -n wasmbed

# Check Gateway URL
kubectl get svc wasmbed-gateway -n wasmbed
```

#### 3. MCU Connection Issues

**Symptoms:**
- Devices not enrolling
- TLS handshake failures
- Heartbeat timeouts

**Diagnosis:**
```bash
# Check Gateway logs for enrollment attempts
kubectl logs -n wasmbed deployment/wasmbed-gateway | grep enrollment

# Test TLS connection
openssl s_client -connect gateway.yourdomain.com:8080 -CAfile ca.crt

# Check device certificates
openssl x509 -in client-0.der -inform DER -text -noout
```

**Solutions:**
```bash
# Regenerate device certificates
cargo run -p wasmbed-cert-tool -- issue-cert \
  --ca-key resources/dev-certs/ca.key \
  --ca-cert resources/dev-certs/ca.der \
  --common-name "mcu-0" \
  --out-key resources/dev-certs/client-0.key \
  --out-cert resources/dev-certs/client-0.der \
  client

# Check network connectivity
ping gateway.yourdomain.com
telnet gateway.yourdomain.com 8080
```

#### 4. Application Deployment Failures

**Symptoms:**
- Applications stuck in `Deploying` phase
- WASM binary errors
- Device not found errors

**Diagnosis:**
```bash
# Check Application status
kubectl get application my-app -o yaml

# Check target devices
kubectl get devices -l device-type=hifive1

# Check WASM binary
echo "AGFzbQEB..." | base64 -d | file -
```

**Solutions:**
```bash
# Verify WASM binary
wasm-validate my-app.wasm

# Check device availability
kubectl get devices --show-labels

# Update Application target
kubectl patch application my-app --type='merge' \
  -p='{"spec":{"target_devices":{"device_names":["available-device"]}}}'
```

### Performance Issues

#### 1. High Memory Usage

**Symptoms:**
- Pods being OOM killed
- Slow response times
- High memory consumption

**Diagnosis:**
```bash
# Check memory usage
kubectl top pods -n wasmbed

# Check memory limits
kubectl get pods -n wasmbed -o yaml | grep -A 5 resources:

# Check memory metrics
kubectl exec -n wasmbed deployment/wasmbed-gateway -- cat /proc/meminfo
```

**Solutions:**
```bash
# Increase memory limits
kubectl patch deployment wasmbed-gateway -n wasmbed --type='merge' \
  -p='{"spec":{"template":{"spec":{"containers":[{"name":"gateway","resources":{"limits":{"memory":"256Mi"}}}]}}}}'

# Optimize application configuration
kubectl patch application my-app --type='merge' \
  -p='{"spec":{"config":{"memory_limit":524288}}}'
```

#### 2. Slow Reconciliation

**Symptoms:**
- Long deployment times
- Controller lag
- High CPU usage

**Diagnosis:**
```bash
# Check reconciliation frequency
kubectl logs -n wasmbed deployment/wasmbed-k8s-controller | grep "Reconciling"

# Check CPU usage
kubectl top pods -n wasmbed

# Check etcd performance
kubectl get events --sort-by='.lastTimestamp'
```

**Solutions:**
```bash
# Adjust reconciliation intervals
kubectl patch deployment wasmbed-k8s-controller -n wasmbed --type='merge' \
  -p='{"spec":{"template":{"spec":{"containers":[{"name":"controller","env":[{"name":"WASMBED_CONTROLLER_RECONCILIATION_INTERVAL","value":"60"}]}]}}}}'

# Scale Controller
kubectl scale deployment wasmbed-k8s-controller -n wasmbed --replicas=2
```

### Network Issues

#### 1. Service Discovery Problems

**Symptoms:**
- Controller can't reach Gateway
- DNS resolution failures
- Network policy blocking traffic

**Diagnosis:**
```bash
# Test service connectivity
kubectl exec -n wasmbed deployment/wasmbed-k8s-controller -- nslookup wasmbed-gateway

# Check network policies
kubectl get networkpolicies -n wasmbed

# Test port connectivity
kubectl exec -n wasmbed deployment/wasmbed-k8s-controller -- telnet wasmbed-gateway 8080
```

**Solutions:**
```bash
# Fix network policies
kubectl apply -f resources/k8s/network-policies/

# Check service endpoints
kubectl get endpoints -n wasmbed

# Restart services
kubectl rollout restart deployment/wasmbed-gateway -n wasmbed
```

#### 2. TLS Certificate Issues

**Symptoms:**
- TLS handshake failures
- Certificate validation errors
- Expired certificates

**Diagnosis:**
```bash
# Check certificate expiration
openssl x509 -in gateway.crt -inform DER -noout -dates

# Verify certificate chain
openssl verify -CAfile ca.crt gateway.crt

# Test TLS connection
openssl s_client -connect gateway.yourdomain.com:8080 -CAfile ca.crt
```

**Solutions:**
```bash
# Renew certificates
./scripts/generate-certs.sh

# Update secrets
kubectl create secret tls wasmbed-gateway-tls \
  --cert=gateway.crt --key=gateway.key -n wasmbed --dry-run=client -o yaml | kubectl apply -f -

# Restart Gateway
kubectl rollout restart deployment/wasmbed-gateway -n wasmbed
```

## 🔧 Maintenance

### Backup and Recovery

#### Backup CRDs

```bash
# Backup all Wasmbed resources
kubectl get devices -n wasmbed -o yaml > backup-devices.yaml
kubectl get applications -n wasmbed -o yaml > backup-applications.yaml
kubectl get secrets -n wasmbed -o yaml > backup-secrets.yaml
```

#### Restore from Backup

```bash
# Restore resources
kubectl apply -f backup-devices.yaml
kubectl apply -f backup-applications.yaml
kubectl apply -f backup-secrets.yaml
```

### Updates and Upgrades

#### Update Wasmbed

```bash
# Update images
kubectl set image deployment/wasmbed-gateway gateway=wasmbed-gateway:v1.1.0 -n wasmbed
kubectl set image deployment/wasmbed-k8s-controller controller=wasmbed-k8s-controller:v1.1.0 -n wasmbed

# Verify update
kubectl rollout status deployment/wasmbed-gateway -n wasmbed
kubectl rollout status deployment/wasmbed-k8s-controller -n wasmbed
```

#### Rollback

```bash
# Rollback to previous version
kubectl rollout undo deployment/wasmbed-gateway -n wasmbed
kubectl rollout undo deployment/wasmbed-k8s-controller -n wasmbed
```

### Cleanup

#### Remove Wasmbed

```bash
# Delete all resources
kubectl delete namespace wasmbed

# Remove CRDs
kubectl delete crd devices.wasmbed.github.io
kubectl delete crd applications.wasmbed.github.io

# Clean up local files
rm -rf resources/dev-certs/*
```

---

**Last Updated**: September 2024  
**Version**: Deployment v1.0.0  
**Maintainer**: Wasmbed Development Team
