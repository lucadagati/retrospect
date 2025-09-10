# Custom Resource Definitions (CRDs) Documentation

## Overview

This document provides comprehensive documentation for the Custom Resource Definitions (CRDs) used in the Wasmbed platform, including schema definitions, examples, and usage guidelines.

## Application CRD

### Definition

**API Version**: `wasmbed.github.io/v1alpha1`
**Kind**: `Application`
**Plural**: `applications`

### Schema

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: applications.wasmbed.github.io
spec:
  group: wasmbed.github.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              name:
                type: string
                description: "Application name"
                minLength: 1
                maxLength: 63
                pattern: "^[a-z0-9]([-a-z0-9]*[a-z0-9])?$"
              version:
                type: string
                description: "Application version"
                pattern: "^[0-9]+\\.[0-9]+\\.[0-9]+$"
              targetDevices:
                type: array
                items:
                  type: string
                  enum: ["mpu", "mcu", "riscv", "arm", "esp32"]
                description: "Target device types"
                minItems: 1
              wasmBinary:
                type: string
                format: byte
                description: "Base64-encoded WASM binary"
                minLength: 1
              config:
                type: object
                description: "Application configuration"
                properties:
                  microros:
                    type: object
                    properties:
                      node_name:
                        type: string
                        description: "microROS node name"
                      domain_id:
                        type: integer
                        description: "DDS domain ID"
                        minimum: 0
                        maximum: 232
                      qos_profile:
                        type: string
                        enum: ["reliable", "best_effort"]
                        description: "QoS profile"
                      transport:
                        type: string
                        enum: ["udp", "tcp", "serial"]
                        description: "Transport protocol"
                  fastdds:
                    type: object
                    properties:
                      domain_id:
                        type: integer
                        description: "DDS domain ID"
                        minimum: 0
                        maximum: 232
                      transport:
                        type: string
                        enum: ["udp", "tcp", "shared_memory"]
                        description: "Transport protocol"
                      port:
                        type: integer
                        description: "Transport port"
                        minimum: 1024
                        maximum: 65535
                  px4:
                    type: object
                    properties:
                      topics:
                        type: object
                        properties:
                          input_topics:
                            type: array
                            items:
                              type: string
                            description: "PX4 input topics"
                          output_topics:
                            type: array
                            items:
                              type: string
                            description: "PX4 output topics"
                      safety:
                        type: object
                        properties:
                          emergency_stop_enabled:
                            type: boolean
                            description: "Enable emergency stop"
                          failsafe_enabled:
                            type: boolean
                            description: "Enable failsafe"
                          battery_monitoring:
                            type: boolean
                            description: "Enable battery monitoring"
                  resources:
                    type: object
                    properties:
                      cpu_limit:
                        type: string
                        description: "CPU limit"
                        pattern: "^[0-9]+m$"
                      memory_limit:
                        type: string
                        description: "Memory limit"
                        pattern: "^[0-9]+(Mi|Gi)$"
                      storage_limit:
                        type: string
                        description: "Storage limit"
                        pattern: "^[0-9]+(Mi|Gi)$"
                additionalProperties: true
              dependencies:
                type: array
                items:
                  type: object
                  properties:
                    name:
                      type: string
                      description: "Dependency name"
                    version:
                      type: string
                      description: "Dependency version"
                    type:
                      type: string
                      enum: ["library", "service", "device"]
                      description: "Dependency type"
                description: "Application dependencies"
              environment:
                type: object
                additionalProperties:
                  type: string
                description: "Environment variables"
            required:
            - name
            - version
            - targetDevices
            - wasmBinary
          status:
            type: object
            properties:
              phase:
                type: string
                enum: ["Creating", "Deploying", "Running", "Stopping", "Stopped", "Failed"]
                description: "Application phase"
              message:
                type: string
                description: "Status message"
              deployedAt:
                type: string
                format: date-time
                description: "Deployment timestamp"
              lastUpdate:
                type: string
                format: date-time
                description: "Last update timestamp"
              instances:
                type: array
                items:
                  type: object
                  properties:
                    deviceId:
                      type: string
                      description: "Device ID"
                    status:
                      type: string
                      enum: ["Deploying", "Running", "Stopped", "Failed"]
                      description: "Instance status"
                    message:
                      type: string
                      description: "Instance message"
                    deployedAt:
                      type: string
                      format: date-time
                      description: "Instance deployment timestamp"
                    lastUpdate:
                      type: string
                      format: date-time
                      description: "Instance last update timestamp"
                description: "Application instances"
              metrics:
                type: object
                properties:
                  totalInstances:
                    type: integer
                    description: "Total number of instances"
                  runningInstances:
                    type: integer
                    description: "Number of running instances"
                  failedInstances:
                    type: integer
                    description: "Number of failed instances"
                  averageCpuUsage:
                    type: number
                    description: "Average CPU usage"
                  averageMemoryUsage:
                    type: number
                    description: "Average memory usage"
                description: "Application metrics"
```

### Examples

**Basic Application**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: hello-world
  namespace: wasmbed
spec:
  name: "Hello World"
  version: "1.0.0"
  targetDevices:
  - "riscv"
  wasmBinary: <base64-encoded-wasm-binary>
  config:
    resources:
      cpu_limit: "100m"
      memory_limit: "64Mi"
status:
  phase: "Running"
  message: "Application deployed successfully"
  deployedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
```

**PX4 Drone Control Application**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: px4-drone-control
  namespace: wasmbed
spec:
  name: "PX4 Drone Control"
  version: "1.0.0"
  targetDevices:
  - "riscv"
  wasmBinary: <base64-encoded-wasm-binary>
  config:
    microros:
      node_name: "px4_drone_control_node"
      domain_id: 0
      qos_profile: "reliable"
      transport: "udp"
    fastdds:
      domain_id: 0
      transport: "udp"
      port: 7400
    px4:
      topics:
        input_topics:
        - "/fmu/in/vehicle_command"
        - "/fmu/in/position_setpoint"
        - "/fmu/in/attitude_setpoint"
        output_topics:
        - "/fmu/out/vehicle_status"
        - "/fmu/out/vehicle_local_position"
        - "/fmu/out/battery_status"
        - "/fmu/out/vehicle_attitude"
        - "/fmu/out/actuator_outputs"
      safety:
        emergency_stop_enabled: true
        failsafe_enabled: true
        battery_monitoring: true
    resources:
      cpu_limit: "200m"
      memory_limit: "128Mi"
      storage_limit: "64Mi"
  dependencies:
  - name: "microros"
    version: "1.0.0"
    type: "library"
  - name: "fastdds"
    version: "2.0.0"
    type: "library"
  environment:
    LOG_LEVEL: "info"
    DEBUG_MODE: "false"
status:
  phase: "Running"
  message: "PX4 application deployed successfully"
  deployedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
  instances:
  - deviceId: "riscv-device-001"
    status: "Running"
    message: "Instance running successfully"
    deployedAt: "2024-01-01T00:00:00Z"
    lastUpdate: "2024-01-01T00:00:00Z"
  metrics:
    totalInstances: 1
    runningInstances: 1
    failedInstances: 0
    averageCpuUsage: 45.2
    averageMemoryUsage: 67.8
```

**Multi-Device Application**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: multi-device-app
  namespace: wasmbed
spec:
  name: "Multi-Device Application"
  version: "1.0.0"
  targetDevices:
  - "riscv"
  - "arm"
  - "esp32"
  wasmBinary: <base64-encoded-wasm-binary>
  config:
    resources:
      cpu_limit: "150m"
      memory_limit: "96Mi"
    environment:
      DEVICE_TYPE: "auto"
      LOG_LEVEL: "debug"
status:
  phase: "Running"
  message: "Multi-device application deployed successfully"
  deployedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
  instances:
  - deviceId: "riscv-device-001"
    status: "Running"
    message: "RISC-V instance running"
    deployedAt: "2024-01-01T00:00:00Z"
    lastUpdate: "2024-01-01T00:00:00Z"
  - deviceId: "arm-device-001"
    status: "Running"
    message: "ARM instance running"
    deployedAt: "2024-01-01T00:00:00Z"
    lastUpdate: "2024-01-01T00:00:00Z"
  - deviceId: "esp32-device-001"
    status: "Running"
    message: "ESP32 instance running"
    deployedAt: "2024-01-01T00:00:00Z"
    lastUpdate: "2024-01-01T00:00:00Z"
  metrics:
    totalInstances: 3
    runningInstances: 3
    failedInstances: 0
    averageCpuUsage: 52.1
    averageMemoryUsage: 73.4
```

## Device CRD

### Definition

**API Version**: `wasmbed.github.io/v1alpha1`
**Kind**: `Device`
**Plural**: `devices`

### Schema

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: devices.wasmbed.github.io
spec:
  group: wasmbed.github.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              deviceType:
                type: string
                enum: ["mpu", "mcu", "riscv", "arm", "esp32"]
                description: "Device type"
              capabilities:
                type: array
                items:
                  type: string
                  enum: ["wasm", "tls", "serial", "wifi", "ethernet", "bluetooth", "gps", "sensors"]
                description: "Device capabilities"
                minItems: 1
              publicKey:
                type: string
                description: "Device public key"
                minLength: 1
              gatewayEndpoint:
                type: string
                description: "Gateway endpoint"
                pattern: "^[a-zA-Z0-9.-]+:[0-9]+$"
              firmwareVersion:
                type: string
                description: "Device firmware version"
                pattern: "^[0-9]+\\.[0-9]+\\.[0-9]+$"
              hardwareInfo:
                type: object
                properties:
                  manufacturer:
                    type: string
                    description: "Device manufacturer"
                  model:
                    type: string
                    description: "Device model"
                  serialNumber:
                    type: string
                    description: "Device serial number"
                  cpu:
                    type: string
                    description: "CPU information"
                  memory:
                    type: string
                    description: "Memory information"
                  storage:
                    type: string
                    description: "Storage information"
                description: "Hardware information"
              networkConfig:
                type: object
                properties:
                  ipAddress:
                    type: string
                    format: ipv4
                    description: "Device IP address"
                  macAddress:
                    type: string
                    pattern: "^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$"
                    description: "Device MAC address"
                  networkInterface:
                    type: string
                    enum: ["wifi", "ethernet", "cellular"]
                    description: "Network interface type"
                description: "Network configuration"
              location:
                type: object
                properties:
                  latitude:
                    type: number
                    description: "Device latitude"
                    minimum: -90
                    maximum: 90
                  longitude:
                    type: number
                    description: "Device longitude"
                    minimum: -180
                    maximum: 180
                  altitude:
                    type: number
                    description: "Device altitude"
                description: "Device location"
            required:
            - deviceType
            - capabilities
            - publicKey
            - gatewayEndpoint
          status:
            type: object
            properties:
              phase:
                type: string
                enum: ["Enrolling", "Enrolled", "Connected", "Disconnected", "Failed"]
                description: "Device phase"
              message:
                type: string
                description: "Status message"
              lastHeartbeat:
                type: string
                format: date-time
                description: "Last heartbeat timestamp"
              pairingMode:
                type: boolean
                description: "Pairing mode status"
              connectionInfo:
                type: object
                properties:
                  connectedAt:
                    type: string
                    format: date-time
                    description: "Connection timestamp"
                  lastSeen:
                    type: string
                    format: date-time
                    description: "Last seen timestamp"
                  connectionDuration:
                    type: string
                    description: "Connection duration"
                  heartbeatInterval:
                    type: integer
                    description: "Heartbeat interval in seconds"
                  missedHeartbeats:
                    type: integer
                    description: "Number of missed heartbeats"
                description: "Connection information"
              metrics:
                type: object
                properties:
                  cpuUsage:
                    type: number
                    description: "CPU usage percentage"
                  memoryUsage:
                    type: number
                    description: "Memory usage percentage"
                  temperature:
                    type: number
                    description: "Device temperature"
                  batteryLevel:
                    type: number
                    description: "Battery level percentage"
                  signalStrength:
                    type: number
                    description: "Signal strength"
                description: "Device metrics"
              applications:
                type: array
                items:
                  type: object
                  properties:
                    appId:
                      type: string
                      description: "Application ID"
                    appName:
                      type: string
                      description: "Application name"
                    status:
                      type: string
                      enum: ["Deploying", "Running", "Stopped", "Failed"]
                      description: "Application status"
                    deployedAt:
                      type: string
                      format: date-time
                      description: "Deployment timestamp"
                description: "Deployed applications"
```

### Examples

**RISC-V Device**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Device
metadata:
  name: riscv-device-001
  namespace: wasmbed
spec:
  deviceType: "riscv"
  capabilities:
  - "wasm"
  - "tls"
  - "serial"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----"
  gatewayEndpoint: "wasmbed-gateway:8080"
  firmwareVersion: "1.0.0"
  hardwareInfo:
    manufacturer: "SiFive"
    model: "HiFive1"
    serialNumber: "SF001234567890"
    cpu: "RISC-V 32-bit"
    memory: "16MB"
    storage: "4MB"
  networkConfig:
    ipAddress: "192.168.1.100"
    macAddress: "00:11:22:33:44:55"
    networkInterface: "ethernet"
status:
  phase: "Connected"
  message: "Device connected successfully"
  lastHeartbeat: "2024-01-01T00:00:00Z"
  pairingMode: false
  connectionInfo:
    connectedAt: "2024-01-01T00:00:00Z"
    lastSeen: "2024-01-01T00:00:00Z"
    connectionDuration: "24h30m15s"
    heartbeatInterval: 30
    missedHeartbeats: 0
  metrics:
    cpuUsage: 45.2
    memoryUsage: 67.8
    temperature: 25.5
    batteryLevel: 100.0
    signalStrength: -45.0
  applications:
  - appId: "px4-drone-control"
    appName: "PX4 Drone Control"
    status: "Running"
    deployedAt: "2024-01-01T00:00:00Z"
```

**ARM Device**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Device
metadata:
  name: arm-device-001
  namespace: wasmbed
spec:
  deviceType: "arm"
  capabilities:
  - "wasm"
  - "tls"
  - "wifi"
  - "sensors"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----"
  gatewayEndpoint: "wasmbed-gateway:8080"
  firmwareVersion: "1.0.0"
  hardwareInfo:
    manufacturer: "STMicroelectronics"
    model: "STM32F4"
    serialNumber: "ST001234567890"
    cpu: "ARM Cortex-M4"
    memory: "32MB"
    storage: "8MB"
  networkConfig:
    ipAddress: "192.168.1.101"
    macAddress: "00:11:22:33:44:66"
    networkInterface: "wifi"
  location:
    latitude: 40.7128
    longitude: -74.0060
    altitude: 10.0
status:
  phase: "Connected"
  message: "Device connected successfully"
  lastHeartbeat: "2024-01-01T00:00:00Z"
  pairingMode: false
  connectionInfo:
    connectedAt: "2024-01-01T00:00:00Z"
    lastSeen: "2024-01-01T00:00:00Z"
    connectionDuration: "24h30m15s"
    heartbeatInterval: 30
    missedHeartbeats: 0
  metrics:
    cpuUsage: 52.1
    memoryUsage: 73.4
    temperature: 28.2
    batteryLevel: 85.0
    signalStrength: -50.0
  applications:
  - appId: "sensor-monitoring"
    appName: "Sensor Monitoring"
    status: "Running"
    deployedAt: "2024-01-01T00:00:00Z"
```

**ESP32 Device**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Device
metadata:
  name: esp32-device-001
  namespace: wasmbed
spec:
  deviceType: "esp32"
  capabilities:
  - "wasm"
  - "tls"
  - "wifi"
  - "bluetooth"
  - "sensors"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----"
  gatewayEndpoint: "wasmbed-gateway:8080"
  firmwareVersion: "1.0.0"
  hardwareInfo:
    manufacturer: "Espressif"
    model: "ESP32"
    serialNumber: "ES001234567890"
    cpu: "XTensa LX6"
    memory: "16MB"
    storage: "4MB"
  networkConfig:
    ipAddress: "192.168.1.102"
    macAddress: "00:11:22:33:44:77"
    networkInterface: "wifi"
status:
  phase: "Connected"
  message: "Device connected successfully"
  lastHeartbeat: "2024-01-01T00:00:00Z"
  pairingMode: false
  connectionInfo:
    connectedAt: "2024-01-01T00:00:00Z"
    lastSeen: "2024-01-01T00:00:00Z"
    connectionDuration: "24h30m15s"
    heartbeatInterval: 30
    missedHeartbeats: 0
  metrics:
    cpuUsage: 38.7
    memoryUsage: 61.2
    temperature: 26.8
    batteryLevel: 92.0
    signalStrength: -40.0
  applications:
  - appId: "iot-gateway"
    appName: "IoT Gateway"
    status: "Running"
    deployedAt: "2024-01-01T00:00:00Z"
```

## Configuration CRD

### Definition

**API Version**: `wasmbed.github.io/v1alpha1`
**Kind**: `Configuration`
**Plural**: `configurations`

### Schema

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: configurations.wasmbed.github.io
spec:
  group: wasmbed.github.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              name:
                type: string
                description: "Configuration name"
                minLength: 1
                maxLength: 63
              configType:
                type: string
                enum: ["gateway", "controller", "device", "application"]
                description: "Configuration type"
              settings:
                type: object
                additionalProperties: true
                description: "Configuration settings"
              scope:
                type: string
                enum: ["global", "namespace", "device", "application"]
                description: "Configuration scope"
              targetDevices:
                type: array
                items:
                  type: string
                description: "Target devices for this configuration"
              targetApplications:
                type: array
                items:
                  type: string
                description: "Target applications for this configuration"
            required:
            - name
            - configType
            - settings
            - scope
          status:
            type: object
            properties:
              phase:
                type: string
                enum: ["Creating", "Active", "Updating", "Failed"]
                description: "Configuration phase"
              message:
                type: string
                description: "Status message"
              appliedAt:
                type: string
                format: date-time
                description: "Application timestamp"
              lastUpdate:
                type: string
                format: date-time
                description: "Last update timestamp"
              appliedTo:
                type: array
                items:
                  type: object
                  properties:
                    resourceType:
                      type: string
                      enum: ["device", "application"]
                    resourceName:
                      type: string
                    appliedAt:
                      type: string
                      format: date-time
                description: "Resources this configuration was applied to"
```

### Examples

**Gateway Configuration**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Configuration
metadata:
  name: gateway-config
  namespace: wasmbed
spec:
  name: "Gateway Configuration"
  configType: "gateway"
  scope: "global"
  settings:
    server:
      host: "0.0.0.0"
      port: 8080
      tls_port: 4423
    logging:
      level: "info"
      format: "json"
    performance:
      max_connections: 10000
      connection_timeout: "30s"
      read_timeout: "60s"
      write_timeout: "60s"
    security:
      tls_enabled: true
      auth_enabled: true
      rate_limiting: true
status:
  phase: "Active"
  message: "Configuration applied successfully"
  appliedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
  appliedTo:
  - resourceType: "device"
    resourceName: "wasmbed-gateway"
    appliedAt: "2024-01-01T00:00:00Z"
```

**Device Configuration**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Configuration
metadata:
  name: device-config
  namespace: wasmbed
spec:
  name: "Device Configuration"
  configType: "device"
  scope: "device"
  targetDevices:
  - "riscv-device-001"
  - "arm-device-001"
  settings:
    heartbeat:
      interval: 30
      timeout: 90
      max_misses: 3
    pairing:
      enabled: false
      timeout: 300
      max_attempts: 3
    logging:
      level: "info"
      max_size: "10MB"
      max_files: 5
status:
  phase: "Active"
  message: "Configuration applied to devices"
  appliedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
  appliedTo:
  - resourceType: "device"
    resourceName: "riscv-device-001"
    appliedAt: "2024-01-01T00:00:00Z"
  - resourceType: "device"
    resourceName: "arm-device-001"
    appliedAt: "2024-01-01T00:00:00Z"
```

## Usage Guidelines

### CRD Management

**Creating CRDs**:
```bash
# Apply CRD definitions
kubectl apply -f resources/k8s/crds/application-crd.yaml
kubectl apply -f resources/k8s/crds/device-crd.yaml
kubectl apply -f resources/k8s/crds/configuration-crd.yaml

# Verify CRDs are created
kubectl get crds | grep wasmbed
```

**Creating Resources**:
```bash
# Create application
kubectl apply -f resources/k8s/test-application.yaml

# Create device
kubectl apply -f resources/k8s/test-device.yaml

# Create configuration
kubectl apply -f resources/k8s/test-configuration.yaml
```

**Managing Resources**:
```bash
# List resources
kubectl get applications -n wasmbed
kubectl get devices -n wasmbed
kubectl get configurations -n wasmbed

# Get resource details
kubectl get application px4-drone-control -n wasmbed -o yaml
kubectl get device riscv-device-001 -n wasmbed -o yaml

# Update resources
kubectl patch application px4-drone-control -n wasmbed --type='merge' -p='{"spec":{"version":"1.1.0"}}'

# Delete resources
kubectl delete application px4-drone-control -n wasmbed
kubectl delete device riscv-device-001 -n wasmbed
```

### Validation and Admission Control

**Schema Validation**:
- All CRDs include comprehensive schema validation
- Required fields are enforced
- Data types and formats are validated
- Enum values are restricted to valid options

**Admission Control**:
- Webhook validation for complex business rules
- Resource quota enforcement
- Security policy validation
- Dependency validation

### Best Practices

**Resource Naming**:
- Use descriptive names for resources
- Follow Kubernetes naming conventions
- Use consistent naming patterns
- Include version information where appropriate

**Resource Organization**:
- Use namespaces to organize resources
- Group related resources together
- Use labels and annotations for metadata
- Implement proper RBAC policies

**Resource Lifecycle**:
- Implement proper cleanup procedures
- Use finalizers for resource cleanup
- Monitor resource status and health
- Implement backup and recovery procedures

**Security Considerations**:
- Validate all input data
- Implement proper authentication and authorization
- Use secure communication protocols
- Monitor for security violations

**Performance Optimization**:
- Use appropriate resource limits
- Implement efficient data structures
- Monitor resource usage
- Optimize for scale
