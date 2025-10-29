# Wasmbed Platform Troubleshooting Guide

This guide provides solutions for common issues encountered when deploying and operating the Wasmbed Platform.

## Common Issues and Solutions

### Port Conflicts

#### Problem
Services fail to start due to port conflicts.

#### Symptoms
- Error messages about ports already in use
- Services fail to bind to ports
- Connection refused errors

#### Solution
```bash
# Stop all services and free ports
./scripts/06-stop-services.sh

# Force kill processes using Wasmbed ports
sudo fuser -k 3000/tcp 3001/tcp 8080/tcp 8081/tcp 30460/tcp 2>/dev/null || true

# Restart deployment
./scripts/04-deploy-infrastructure.sh
```

### Certificate Issues

#### Problem
TLS handshake failures or certificate errors.

#### Symptoms
- `UnsupportedCertVersion` errors
- TLS handshake failures
- Certificate validation errors

#### Solution
```bash
# Remove existing certificates
rm -rf certs/*

# Regenerate certificates
./scripts/04-deploy-infrastructure.sh

# Verify certificate versions
openssl x509 -in certs/ca-cert.pem -text -noout | grep Version
openssl x509 -in certs/server-cert.pem -text -noout | grep Version
```

#### Manual Certificate Generation
```bash
# Generate CA certificate (X.509 v3)
openssl req -x509 -newkey rsa:4096 -keyout certs/ca-key.pem -out certs/ca-cert.pem -days 365 -nodes -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=Wasmbed CA" -extensions v3_ca -extfile <(echo -e "[v3_ca]\nbasicConstraints = critical,CA:TRUE\nkeyUsage = critical, keyCertSign, cRLSign\nsubjectKeyIdentifier = hash\nauthorityKeyIdentifier = keyid:always,issuer")

# Generate server certificate (X.509 v3)
openssl req -newkey rsa:4096 -keyout certs/server-key.pem -out certs/server.csr -nodes -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=127.0.0.1"

# Sign server certificate
openssl x509 -req -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/server-cert.pem -days 365 -CAcreateserial -extensions v3_req -extfile <(echo -e "[v3_req]\nbasicConstraints = CA:FALSE\nkeyUsage = nonRepudiation, digitalSignature, keyEncipherment\nextendedKeyUsage = serverAuth\nsubjectAltName = @alt_names\n[alt_names]\nDNS.1 = localhost\nIP.1 = 127.0.0.1")
```

### Renode Issues

#### Problem
Renode devices fail to start or stop unexpectedly.

#### Symptoms
- Renode processes exit immediately
- Device status shows "Stopped" after starting
- Renode console errors

#### Solution
```bash
# Check Renode installation
which renode

# Test Renode directly
renode --console --execute "mach create; mach LoadPlatformDescription @platforms/boards/arduino_nano_33_ble.repl"

# Check Renode binary path in manager
cargo run --release -p wasmbed-qemu-manager -- list
```

#### Renode Configuration Issues
```bash
# Verify Renode platform files exist
ls /path/to/renode/platforms/boards/

# Check Renode version
renode --version

# Test specific platform
renode --console --execute "mach create; mach LoadPlatformDescription @platforms/boards/stm32f4_discovery.repl"
```

### Kubernetes Issues

#### Problem
Kubernetes resources fail to create or update.

#### Symptoms
- CRD creation failures
- Controller errors
- Resource reconciliation issues

#### Solution
```bash
# Check Kubernetes cluster status
kubectl cluster-info

# Verify CRDs are installed
kubectl get crd

# Check controller logs
kubectl logs -n wasmbed deployment/wasmbed-device-controller
kubectl logs -n wasmbed deployment/wasmbed-application-controller
kubectl logs -n wasmbed deployment/wasmbed-gateway-controller

# Restart controllers
kubectl rollout restart deployment/wasmbed-device-controller -n wasmbed
kubectl rollout restart deployment/wasmbed-application-controller -n wasmbed
kubectl rollout restart deployment/wasmbed-gateway-controller -n wasmbed
```

### Gateway Issues

#### Problem
Gateway fails to start or accept connections.

#### Symptoms
- Gateway process exits
- TLS handshake failures
- Device connection failures

#### Solution
```bash
# Check gateway logs
tail -f logs/gateway.log

# Verify gateway configuration
cargo run --release -p wasmbed-gateway -- --help

# Restart gateway with correct parameters
pkill -f wasmbed-gateway
nohup cargo run --release -p wasmbed-gateway -- --bind-addr 127.0.0.1:8081 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-1 > logs/gateway.log 2>&1 &
```

### API Server Issues

#### Problem
API server fails to respond or returns errors.

#### Symptoms
- HTTP 500 errors
- API endpoints not responding
- Dashboard connection failures

#### Solution
```bash
# Check API server logs
tail -f logs/api-server.log

# Test API endpoints
curl http://localhost:3001/health
curl http://localhost:3001/api/v1/devices

# Restart API server
pkill -f wasmbed-api-server
nohup cargo run --release -p wasmbed-api-server -- --port 3001 --gateway-endpoint http://localhost:8080 --infrastructure-endpoint http://localhost:30460 > logs/api-server.log 2>&1 &
```

### Firmware Issues

#### Problem
Firmware fails to compile or run.

#### Symptoms
- Compilation errors
- Runtime panics
- TLS connection failures

#### Solution
```bash
# Check firmware compilation
cargo build --release --bin firmware_arduino_nano_33_ble

# Check crypto provider installation
grep -n "default_provider" firmware/firmware_arduino_nano_33_ble.rs

# Test firmware standalone
cargo run --release --bin firmware_arduino_nano_33_ble
```

#### Crypto Provider Issues
```rust
// Ensure crypto provider is installed in main()
fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");
    // ... rest of main
}
```

### Dashboard Issues

#### Problem
React dashboard fails to load or display data.

#### Symptoms
- Blank dashboard page
- JavaScript errors
- API connection failures

#### Solution
```bash
# Check dashboard logs
tail -f logs/dashboard.log

# Restart dashboard
cd dashboard-react
npm start

# Check API connectivity
curl http://localhost:3001/health
```

#### Build Issues
```bash
# Clean and rebuild dashboard
cd dashboard-react
rm -rf node_modules package-lock.json
npm install
npm run build
```

## Service Recovery Procedures

### Complete System Recovery

```bash
# Stop all services
./scripts/06-stop-services.sh

# Clean environment
./scripts/01-cleanup-environment.sh

# Rebuild components
./scripts/03-build-components.sh

# Redeploy infrastructure
./scripts/04-deploy-infrastructure.sh

# Verify deployment
./scripts/05-check-system-status.sh
```

### Individual Service Recovery

#### Gateway Recovery
```bash
# Stop gateway
pkill -f wasmbed-gateway

# Restart gateway
nohup cargo run --release -p wasmbed-gateway -- --bind-addr 127.0.0.1:8081 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-1 > logs/gateway.log 2>&1 &

# Verify gateway
curl http://localhost:8080/health
```

#### API Server Recovery
```bash
# Stop API server
pkill -f wasmbed-api-server

# Restart API server
nohup cargo run --release -p wasmbed-api-server -- --port 3001 --gateway-endpoint http://localhost:8080 --infrastructure-endpoint http://localhost:30460 > logs/api-server.log 2>&1 &

# Verify API server
curl http://localhost:3001/health
```

#### Controller Recovery
```bash
# Restart device controller
pkill -f wasmbed-device-controller
nohup cargo run --release -p wasmbed-device-controller > logs/device-controller.log 2>&1 &

# Restart application controller
pkill -f wasmbed-application-controller
nohup cargo run --release -p wasmbed-application-controller > logs/application-controller.log 2>&1 &

# Restart gateway controller
pkill -f wasmbed-gateway-controller
nohup cargo run --release -p wasmbed-gateway-controller > logs/gateway-controller.log 2>&1 &
```

## Diagnostic Commands

### System Health Check
```bash
# Overall system status
./scripts/05-check-system-status.sh

# Service health
curl http://localhost:3001/health
curl http://localhost:8080/health

# Port usage
netstat -tlnp | grep -E "3000|3001|8080|8081|30460"

# Process status
ps aux | grep wasmbed
```

### Log Analysis
```bash
# Recent errors
grep -i error logs/*.log

# Gateway issues
grep -i "tls\|certificate\|handshake" logs/gateway.log

# API issues
grep -i "error\|panic\|failed" logs/api-server.log

# Device issues
grep -i "renode\|device" logs/device-controller.log
```

### Network Diagnostics
```bash
# Test API connectivity
curl -v http://localhost:3001/api/v1/devices

# Test gateway connectivity
curl -v http://localhost:8080/health

# Test TLS connectivity
openssl s_client -connect 127.0.0.1:8081 -cert certs/device-cert.pem -key certs/device-key.pem -CAfile certs/ca-cert.pem
```

## Performance Issues

### High CPU Usage
```bash
# Check CPU usage
top -p $(pgrep wasmbed)

# Check for infinite loops
grep -i "loop\|while\|for" logs/*.log

# Monitor resource usage
kubectl top pods -n wasmbed
```

### Memory Issues
```bash
# Check memory usage
ps aux | grep wasmbed | awk '{print $4, $6, $11}'

# Check for memory leaks
valgrind --tool=memcheck cargo run --release -p wasmbed-gateway

# Monitor memory growth
watch -n 1 'ps aux | grep wasmbed'
```

### Network Performance
```bash
# Check network connections
netstat -an | grep -E "3000|3001|8080|8081"

# Monitor network traffic
tcpdump -i lo port 8081

# Test network latency
ping -c 10 127.0.0.1
```

## Prevention and Best Practices

### Regular Maintenance
```bash
# Daily health checks
./scripts/05-check-system-status.sh

# Weekly log rotation
logrotate /etc/logrotate.d/wasmbed

# Monthly certificate renewal
openssl x509 -in certs/server-cert.pem -noout -dates
```

### Monitoring Setup
```bash
# Set up monitoring alerts
curl -X POST http://localhost:3001/api/v1/monitoring/alerts \
  -H "Content-Type: application/json" \
  -d '{"service": "gateway", "threshold": 90, "metric": "cpu"}'
```

### Backup Procedures
```bash
# Backup configurations
cp qemu_devices.json qemu_devices.json.backup.$(date +%Y%m%d)

# Backup certificates
tar -czf certs-backup-$(date +%Y%m%d).tar.gz certs/

# Backup Kubernetes resources
kubectl get devices,applications,gateways -n wasmbed -o yaml > k8s-backup-$(date +%Y%m%d).yaml
```

## Getting Help

### Log Collection
```bash
# Collect all logs for analysis
tar -czf wasmbed-logs-$(date +%Y%m%d).tar.gz logs/

# Collect system information
uname -a > system-info.txt
rustc --version >> system-info.txt
kubectl version >> system-info.txt
```

### Support Resources
1. Check [Known Issues](../problems/known-issues.md)
2. Review [API Reference](../api/api-reference.md)
3. Consult [Security Overview](../security/security-overview.md)
4. Check [Complete Implementation](../implementation/complete-implementation.md)

---

**Last Updated**: 2025  
**Version**: 0.1.0  
**Status**: Production Ready