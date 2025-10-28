# Configuration Guide

## Overview

This document provides comprehensive configuration guidance for the Wasmbed platform, including environment variables, configuration files, and deployment-specific settings.

## Configuration Architecture

### Configuration Layers

**Environment Variables**:
- Runtime configuration
- Security-sensitive values
- Environment-specific settings

**ConfigMaps**:
- Non-sensitive configuration
- Application settings
- Feature flags

**Secrets**:
- Sensitive data
- Certificates and keys
- Database credentials

**Command Line Arguments**:
- Runtime overrides
- Debug settings
- Development options

## Environment Variables

### Gateway Configuration

**Core Settings**:
```bash
# Server configuration
WASMBED_GATEWAY_HOST=0.0.0.0
WASMBED_GATEWAY_PORT=8080
WASMBED_GATEWAY_TLS_PORT=4423

# Logging configuration
WASMBED_GATEWAY_LOG_LEVEL=info
RUST_LOG=info
RUST_BACKTRACE=1

# Performance settings
WASMBED_GATEWAY_MAX_CONNECTIONS=10000
WASMBED_GATEWAY_CONNECTION_TIMEOUT=30s
WASMBED_GATEWAY_READ_TIMEOUT=60s
WASMBED_GATEWAY_WRITE_TIMEOUT=60s
```

**Security Settings**:
```bash
# TLS configuration
WASMBED_GATEWAY_TLS_ENABLED=true
WASMBED_GATEWAY_TLS_CERT_FILE=/etc/wasmbed-gateway/tls.crt
WASMBED_GATEWAY_TLS_KEY_FILE=/etc/wasmbed-gateway/tls.key
WASMBED_GATEWAY_TLS_CA_FILE=/etc/wasmbed-gateway/ca.crt

# Authentication settings
WASMBED_GATEWAY_AUTH_ENABLED=true
WASMBED_GATEWAY_AUTH_TIMEOUT=300s
WASMBED_GATEWAY_AUTH_MAX_ATTEMPTS=3
```

**Device Management**:
```bash
# Device settings
WASMBED_GATEWAY_MAX_DEVICES=1000
WASMBED_GATEWAY_DEVICE_TIMEOUT=300s
WASMBED_GATEWAY_HEARTBEAT_INTERVAL=30s
WASMBED_GATEWAY_HEARTBEAT_TIMEOUT=90s

# Pairing settings
WASMBED_GATEWAY_PAIRING_MODE=false
WASMBED_GATEWAY_PAIRING_TIMEOUT=300s
WASMBED_GATEWAY_PAIRING_MAX_ATTEMPTS=3
```

### Controller Configuration

**Kubernetes Settings**:
```bash
# Kubernetes configuration
KUBECONFIG=/etc/kubernetes/kubeconfig
KUBERNETES_NAMESPACE=wasmbed
KUBERNETES_CONTEXT=wasmbed-prod

# Controller settings
WASMBED_CONTROLLER_LOG_LEVEL=info
WASMBED_CONTROLLER_RECONCILIATION_INTERVAL=30s
WASMBED_CONTROLLER_MAX_RETRIES=3
WASMBED_CONTROLLER_RETRY_DELAY=5s
```

**Performance Settings**:
```bash
# Performance configuration
WASMBED_CONTROLLER_WORKER_THREADS=4
WASMBED_CONTROLLER_QUEUE_SIZE=1000
WASMBED_CONTROLLER_BATCH_SIZE=100
WASMBED_CONTROLLER_PROCESSING_TIMEOUT=60s
```

### QEMU Bridge Configuration

**QEMU Settings**:
```bash
# QEMU configuration
WASMBED_QEMU_BRIDGE_LOG_LEVEL=info
WASMBED_QEMU_BRIDGE_MAX_DEVICES=100
WASMBED_QEMU_BRIDGE_DEVICE_TIMEOUT=300s

# Serial communication
WASMBED_QEMU_BRIDGE_SERIAL_TIMEOUT=30s
WASMBED_QEMU_BRIDGE_SERIAL_BUFFER_SIZE=4096
WASMBED_QEMU_BRIDGE_SERIAL_RETRY_ATTEMPTS=3
```

**Device Settings**:
```bash
# RISC-V device
WASMBED_QEMU_RISCV_ENABLED=true
WASMBED_QEMU_RISCV_PORT=4444
WASMBED_QEMU_RISCV_MEMORY=128M
WASMBED_QEMU_RISCV_CPUS=2

# ARM device
WASMBED_QEMU_ARM_ENABLED=true
WASMBED_QEMU_ARM_PORT=4447
WASMBED_QEMU_ARM_MEMORY=64M
WASMBED_QEMU_ARM_CPUS=1

# ESP32 device
WASMBED_QEMU_ESP32_ENABLED=true
WASMBED_QEMU_ESP32_PORT=4449
WASMBED_QEMU_ESP32_MEMORY=4M
WASMBED_QEMU_ESP32_CPUS=1
```

## Configuration Files

### Gateway ConfigMap

**Basic Configuration**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-gateway-config
  namespace: wasmbed
data:
  # Server configuration
  host: "0.0.0.0"
  port: "8080"
  tls_port: "4423"
  
  # Logging configuration
  log_level: "info"
  
  # Performance settings
  max_connections: "10000"
  connection_timeout: "30s"
  read_timeout: "60s"
  write_timeout: "60s"
  
  # Device management
  max_devices: "1000"
  device_timeout: "300s"
  heartbeat_interval: "30s"
  heartbeat_timeout: "90s"
  
  # Pairing settings
  pairing_mode: "false"
  pairing_timeout: "300s"
  pairing_max_attempts: "3"
  
  # Security settings
  auth_enabled: "true"
  auth_timeout: "300s"
  auth_max_attempts: "3"
```

**Advanced Configuration**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-gateway-advanced-config
  namespace: wasmbed
data:
  # Performance tuning
  worker_threads: "8"
  queue_size: "10000"
  batch_size: "100"
  processing_timeout: "60s"
  
  # Memory management
  max_memory_usage: "80%"
  gc_interval: "30s"
  memory_pool_size: "1000"
  
  # Network optimization
  tcp_keepalive: "true"
  tcp_keepalive_time: "600s"
  tcp_keepalive_interval: "60s"
  tcp_keepalive_probes: "3"
  
  # Compression settings
  compression_enabled: "true"
  compression_level: "6"
  compression_min_size: "1024"
  
  # Rate limiting
  rate_limit_enabled: "true"
  rate_limit_requests: "1000"
  rate_limit_window: "60s"
  rate_limit_burst: "100"
```

### Controller ConfigMap

**Controller Configuration**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-controller-config
  namespace: wasmbed
data:
  # Controller settings
  log_level: "info"
  reconciliation_interval: "30s"
  max_retries: "3"
  retry_delay: "5s"
  
  # Performance settings
  worker_threads: "4"
  queue_size: "1000"
  batch_size: "100"
  processing_timeout: "60s"
  
  # Health check settings
  health_check_interval: "10s"
  health_check_timeout: "5s"
  health_check_retries: "3"
  
  # Metrics settings
  metrics_enabled: "true"
  metrics_port: "8080"
  metrics_path: "/metrics"
  
  # Application settings
  application_timeout: "300s"
  application_retry_attempts: "3"
  application_retry_delay: "10s"
  
  # Device settings
  device_timeout: "300s"
  device_retry_attempts: "3"
  device_retry_delay: "10s"
```

### QEMU Bridge ConfigMap

**QEMU Bridge Configuration**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-qemu-bridge-config
  namespace: wasmbed
data:
  # Bridge settings
  log_level: "info"
  max_devices: "100"
  device_timeout: "300s"
  
  # Serial communication
  serial_timeout: "30s"
  serial_buffer_size: "4096"
  serial_retry_attempts: "3"
  
  # RISC-V device
  riscv_enabled: "true"
  riscv_port: "4444"
  riscv_memory: "128M"
  riscv_cpus: "2"
  riscv_machine: "sifive_u"
  
  # ARM device
  arm_enabled: "true"
  arm_port: "4447"
  arm_memory: "64M"
  arm_cpus: "1"
  arm_machine: "stm32-p103"
  
  # ESP32 device
  esp32_enabled: "true"
  esp32_port: "4449"
  esp32_memory: "4M"
  esp32_cpus: "1"
  esp32_machine: "esp32"
  
  # Network settings
  network_enabled: "true"
  network_bridge: "br0"
  network_dhcp: "true"
  network_dns: "8.8.8.8"
```

## Secrets Configuration

### TLS Secrets

**TLS Certificate Secret**:
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

**CA Certificate Secret**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: wasmbed-ca-secret-rsa
  namespace: wasmbed
type: Opaque
data:
  ca-cert.pem: <base64-encoded-ca-certificate>
```

### Database Secrets

**Database Credentials**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: wasmbed-database-secret
  namespace: wasmbed
type: Opaque
data:
  username: <base64-encoded-username>
  password: <base64-encoded-password>
  host: <base64-encoded-host>
  port: <base64-encoded-port>
  database: <base64-encoded-database>
```

### API Keys

**API Key Secret**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: wasmbed-api-keys
  namespace: wasmbed
type: Opaque
data:
  admin-api-key: <base64-encoded-admin-key>
  user-api-key: <base64-encoded-user-key>
  device-api-key: <base64-encoded-device-key>
```

## Deployment-Specific Configuration

### Development Environment

**Development ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-dev-config
  namespace: wasmbed
data:
  # Development settings
  debug_mode: "true"
  log_level: "debug"
  hot_reload: "true"
  
  # Performance settings (relaxed for development)
  max_connections: "1000"
  connection_timeout: "60s"
  read_timeout: "120s"
  write_timeout: "120s"
  
  # Device settings (relaxed for development)
  max_devices: "100"
  device_timeout: "600s"
  heartbeat_interval: "60s"
  heartbeat_timeout: "180s"
  
  # Pairing settings (enabled for development)
  pairing_mode: "true"
  pairing_timeout: "600s"
  pairing_max_attempts: "10"
```

### Staging Environment

**Staging ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-staging-config
  namespace: wasmbed
data:
  # Staging settings
  debug_mode: "false"
  log_level: "info"
  hot_reload: "false"
  
  # Performance settings (moderate for staging)
  max_connections: "5000"
  connection_timeout: "45s"
  read_timeout: "90s"
  write_timeout: "90s"
  
  # Device settings (moderate for staging)
  max_devices: "500"
  device_timeout: "450s"
  heartbeat_interval: "45s"
  heartbeat_timeout: "135s"
  
  # Pairing settings (enabled for staging)
  pairing_mode: "true"
  pairing_timeout: "450s"
  pairing_max_attempts: "5"
```

### Production Environment

**Production ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-prod-config
  namespace: wasmbed
data:
  # Production settings
  debug_mode: "false"
  log_level: "warn"
  hot_reload: "false"
  
  # Performance settings (optimized for production)
  max_connections: "10000"
  connection_timeout: "30s"
  read_timeout: "60s"
  write_timeout: "60s"
  
  # Device settings (optimized for production)
  max_devices: "1000"
  device_timeout: "300s"
  heartbeat_interval: "30s"
  heartbeat_timeout: "90s"
  
  # Pairing settings (disabled for production)
  pairing_mode: "false"
  pairing_timeout: "300s"
  pairing_max_attempts: "3"
  
  # Security settings (strict for production)
  auth_enabled: "true"
  auth_timeout: "300s"
  auth_max_attempts: "3"
  rate_limit_enabled: "true"
  rate_limit_requests: "1000"
  rate_limit_window: "60s"
```

## Configuration Validation

### Validation Rules

**Gateway Configuration Validation**:
```rust
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub tls_port: u16,
    pub log_level: LogLevel,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub max_devices: u32,
    pub device_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub pairing_mode: bool,
    pub pairing_timeout: Duration,
    pub pairing_max_attempts: u32,
}

impl GatewayConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port ranges
        if self.port == 0 || self.port > 65535 {
            return Err(ConfigError::InvalidPort(self.port));
        }
        
        if self.tls_port == 0 || self.tls_port > 65535 {
            return Err(ConfigError::InvalidPort(self.tls_port));
        }
        
        // Validate timeouts
        if self.connection_timeout < Duration::from_secs(1) {
            return Err(ConfigError::InvalidTimeout("connection_timeout too short"));
        }
        
        if self.read_timeout < Duration::from_secs(1) {
            return Err(ConfigError::InvalidTimeout("read_timeout too short"));
        }
        
        if self.write_timeout < Duration::from_secs(1) {
            return Err(ConfigError::InvalidTimeout("write_timeout too short"));
        }
        
        // Validate device settings
        if self.max_devices == 0 {
            return Err(ConfigError::InvalidValue("max_devices must be greater than 0"));
        }
        
        if self.device_timeout < Duration::from_secs(30) {
            return Err(ConfigError::InvalidTimeout("device_timeout too short"));
        }
        
        // Validate heartbeat settings
        if self.heartbeat_interval < Duration::from_secs(10) {
            return Err(ConfigError::InvalidTimeout("heartbeat_interval too short"));
        }
        
        if self.heartbeat_timeout <= self.heartbeat_interval {
            return Err(ConfigError::InvalidTimeout("heartbeat_timeout must be greater than heartbeat_interval"));
        }
        
        // Validate pairing settings
        if self.pairing_timeout < Duration::from_secs(30) {
            return Err(ConfigError::InvalidTimeout("pairing_timeout too short"));
        }
        
        if self.pairing_max_attempts == 0 {
            return Err(ConfigError::InvalidValue("pairing_max_attempts must be greater than 0"));
        }
        
        Ok(())
    }
}
```

### Configuration Loading

**Configuration Loader**:
```rust
pub struct ConfigLoader {
    config: GatewayConfig,
}

impl ConfigLoader {
    pub fn new() -> Result<Self, ConfigError> {
        let config = GatewayConfig {
            host: env::var("WASMBED_GATEWAY_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("WASMBED_GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            tls_port: env::var("WASMBED_GATEWAY_TLS_PORT")
                .unwrap_or_else(|_| "4423".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            log_level: env::var("WASMBED_GATEWAY_LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidLogLevel)?,
            max_connections: env::var("WASMBED_GATEWAY_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue)?,
            connection_timeout: env::var("WASMBED_GATEWAY_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            read_timeout: env::var("WASMBED_GATEWAY_READ_TIMEOUT")
                .unwrap_or_else(|_| "60s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            write_timeout: env::var("WASMBED_GATEWAY_WRITE_TIMEOUT")
                .unwrap_or_else(|_| "60s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            max_devices: env::var("WASMBED_GATEWAY_MAX_DEVICES")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue)?,
            device_timeout: env::var("WASMBED_GATEWAY_DEVICE_TIMEOUT")
                .unwrap_or_else(|_| "300s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            heartbeat_interval: env::var("WASMBED_GATEWAY_HEARTBEAT_INTERVAL")
                .unwrap_or_else(|_| "30s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            heartbeat_timeout: env::var("WASMBED_GATEWAY_HEARTBEAT_TIMEOUT")
                .unwrap_or_else(|_| "90s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            pairing_mode: env::var("WASMBED_GATEWAY_PAIRING_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue)?,
            pairing_timeout: env::var("WASMBED_GATEWAY_PAIRING_TIMEOUT")
                .unwrap_or_else(|_| "300s".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
            pairing_max_attempts: env::var("WASMBED_GATEWAY_PAIRING_MAX_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue)?,
        };
        
        config.validate()?;
        
        Ok(Self { config })
    }
    
    pub fn get_config(&self) -> &GatewayConfig {
        &self.config
    }
}
```

## Configuration Management

### Dynamic Configuration Updates

**Configuration Watcher**:
```rust
pub struct ConfigWatcher {
    config: Arc<RwLock<GatewayConfig>>,
    watcher: notify::RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn new(config_path: &Path) -> Result<Self, ConfigError> {
        let config = Arc::new(RwLock::new(ConfigLoader::new()?.get_config().clone()));
        let config_clone = config.clone();
        
        let mut watcher = notify::RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            if let Ok(new_config) = ConfigLoader::new() {
                                if let Ok(mut config) = config_clone.write() {
                                    *config = new_config.get_config().clone();
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("watch error: {:?}", e),
                }
            },
            notify::WatcherConfig::default(),
        )?;
        
        watcher.watch(config_path, notify::RecursiveMode::NonRecursive)?;
        
        Ok(Self { config, watcher })
    }
    
    pub fn get_config(&self) -> Arc<RwLock<GatewayConfig>> {
        self.config.clone()
    }
}
```

### Configuration Backup and Restore

**Configuration Backup**:
```bash
# Backup configuration
kubectl get configmaps -n wasmbed -o yaml > configmaps-backup.yaml
kubectl get secrets -n wasmbed -o yaml > secrets-backup.yaml

# Backup specific configuration
kubectl get configmap wasmbed-gateway-config -n wasmbed -o yaml > gateway-config-backup.yaml
```

**Configuration Restore**:
```bash
# Restore configuration
kubectl apply -f configmaps-backup.yaml
kubectl apply -f secrets-backup.yaml

# Restore specific configuration
kubectl apply -f gateway-config-backup.yaml
```

## Best Practices

### Configuration Security

**Security Guidelines**:
- Never store sensitive data in ConfigMaps
- Use Secrets for all sensitive information
- Encrypt secrets at rest
- Rotate secrets regularly
- Use least privilege access

**Secret Management**:
- Use external secret management systems
- Implement secret rotation
- Monitor secret access
- Audit secret usage

### Configuration Validation

**Validation Guidelines**:
- Validate all configuration values
- Provide clear error messages
- Use appropriate data types
- Implement range checks
- Validate dependencies

**Error Handling**:
- Provide descriptive error messages
- Log configuration errors
- Implement fallback values
- Graceful degradation

### Configuration Documentation

**Documentation Guidelines**:
- Document all configuration options
- Provide examples
- Include default values
- Explain impact of changes
- Document dependencies
