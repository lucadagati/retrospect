# Kubernetes CRDs Reference

##  Overview

Wasmbed extends Kubernetes with Custom Resource Definitions (CRDs) to manage IoT devices and WASM applications. This document provides a comprehensive reference for all CRDs, their schemas, and usage examples.

##  CRD Definitions

### 1. Device CRD

The Device CRD represents an MCU device that can run WASM applications.

#### Schema Definition

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: devices.wasmbed.github.io
spec:
  group: wasmbed.github.io
  names:
    kind: Device
    listKind: DeviceList
    plural: devices
    singular: device
    shortNames:
    - dev
  scope: Namespaced
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
              device_type:
                type: string
                description: "Type of the MCU device (e.g., hifive1)"
              capabilities:
                type: array
                items:
                  type: string
                description: "Device capabilities (e.g., wasm, tls)"
              public_key:
                type: string
                description: "Base64 encoded public key"
              firmware_version:
                type: string
                description: "Firmware version"
              hardware_id:
                type: string
                description: "Unique hardware identifier"
            required:
            - device_type
            - capabilities
            - public_key
          status:
            type: object
            properties:
              phase:
                type: string
                enum:
                - Enrolling
                - Connected
                - Disconnected
                - Failed
              last_heartbeat:
                type: string
                format: date-time
              connection_info:
                type: object
                properties:
                  gateway_address:
                    type: string
                  connection_id:
                    type: string
                  established_at:
                    type: string
                    format: date-time
              error:
                type: string
```

#### Example Device Resource

```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Device
metadata:
  name: hifive1-001
  namespace: wasmbed
  labels:
    device-type: hifive1
    location: lab-1
spec:
  device_type: "hifive1"
  capabilities:
    - "wasm"
    - "tls"
    - "network"
  public_key: "LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0K..."
  firmware_version: "1.0.0"
  hardware_id: "hifive1-001"
status:
  phase: "Connected"
  last_heartbeat: "2024-09-01T22:00:00Z"
  connection_info:
    gateway_address: "172.19.0.2:30423"
    connection_id: "conn-12345"
    established_at: "2024-09-01T21:55:00Z"
```

### 2. Application CRD

The Application CRD represents a WASM application that can be deployed to MCU devices.

#### Schema Definition

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: applications.wasmbed.github.io
spec:
  group: wasmbed.github.io
  names:
    kind: Application
    listKind: ApplicationList
    plural: applications
    singular: application
    shortNames:
    - app
  scope: Namespaced
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
                description: "Human-readable application name"
              description:
                type: string
                description: "Application description"
              wasm_bytes:
                type: string
                description: "Base64 encoded WASM binary"
              target_devices:
                type: object
                properties:
                  device_names:
                    type: array
                    items:
                      type: string
                    description: "List of target device names"
                  device_labels:
                    type: object
                    additionalProperties:
                      type: string
                    description: "Device label selectors"
                  device_count:
                    type: integer
                    minimum: 1
                    description: "Number of devices to deploy to"
              config:
                type: object
                properties:
                  memory_limit:
                    type: integer
                    minimum: 1024
                    description: "Memory limit in bytes"
                  cpu_time_limit:
                    type: integer
                    minimum: 100
                    description: "CPU time limit in milliseconds"
                  auto_restart:
                    type: boolean
                    default: true
                    description: "Auto-restart on failure"
                  max_restarts:
                    type: integer
                    minimum: 0
                    maximum: 10
                    default: 3
                    description: "Maximum restart attempts"
                  timeout:
                    type: integer
                    minimum: 1000
                    description: "Application timeout in milliseconds"
                  environment_vars:
                    type: object
                    additionalProperties:
                      type: string
                    description: "Environment variables"
                  args:
                    type: array
                    items:
                      type: string
                    description: "Application arguments"
            required:
            - name
            - wasm_bytes
            - target_devices
          status:
            type: object
            properties:
              phase:
                type: string
                enum:
                - Creating
                - Deploying
                - Running
                - PartiallyRunning
                - Stopping
                - Stopped
                - Failed
                - Deleting
              message:
                type: string
                description: "Human-readable status message"
              device_statuses:
                type: object
                additionalProperties:
                  type: object
                  properties:
                    phase:
                      type: string
                      enum:
                      - Deploying
                      - Running
                      - Stopped
                      - Failed
                    last_updated:
                      type: string
                      format: date-time
                    error:
                      type: string
              metrics:
                type: object
                properties:
                  total_devices:
                    type: integer
                  running_devices:
                    type: integer
                  failed_devices:
                    type: integer
                  stopped_devices:
                    type: integer
              last_updated:
                type: string
                format: date-time
              error:
                type: string
```

#### Example Application Resource

```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: temperature-monitor
  namespace: wasmbed
  labels:
    app-type: monitoring
    version: v1.0.0
spec:
  name: "Temperature Monitor"
  description: "Monitors temperature sensors and reports data"
  wasm_bytes: "AGFzbQEB..."  # Base64 encoded WASM binary
  target_devices:
    device_names:
      - "hifive1-001"
      - "hifive1-002"
    device_labels:
      device-type: hifive1
      location: lab-1
    device_count: 2
  config:
    memory_limit: 1048576      # 1MB
    cpu_time_limit: 1000       # 1 second
    auto_restart: true
    max_restarts: 3
    timeout: 5000              # 5 seconds
    environment_vars:
      SENSOR_TYPE: "temperature"
      REPORT_INTERVAL: "30"
    args:
      - "--verbose"
      - "--log-level=info"
status:
  phase: "Running"
  message: "Running on 2/2 devices"
  device_statuses:
    hifive1-001:
      phase: "Running"
      last_updated: "2024-09-01T22:00:00Z"
    hifive1-002:
      phase: "Running"
      last_updated: "2024-09-01T22:00:00Z"
  metrics:
    total_devices: 2
    running_devices: 2
    failed_devices: 0
    stopped_devices: 0
  last_updated: "2024-09-01T22:00:00Z"
```

##  CRD Operations

### Creating Resources

#### Create Device

```bash
# Create device from YAML file
kubectl apply -f device.yaml

# Create device from command line
kubectl create device hifive1-001 \
  --device-type=hifive1 \
  --capabilities=wasm,tls \
  --public-key="LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0K..."
```

#### Create Application

```bash
# Create application from YAML file
kubectl apply -f application.yaml

# Create application from command line
kubectl create application temperature-monitor \
  --name="Temperature Monitor" \
  --wasm-bytes="AGFzbQEB..." \
  --target-devices=hifive1-001,hifive1-002
```

### Querying Resources

#### List Resources

```bash
# List all devices
kubectl get devices

# List all applications
kubectl get applications

# List with custom columns
kubectl get devices -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,TYPE:.spec.device_type

# List with wide output
kubectl get applications -o wide
```

#### Get Resource Details

```bash
# Get device details
kubectl get device hifive1-001 -o yaml

# Get application details
kubectl get application temperature-monitor -o yaml

# Get status only
kubectl get device hifive1-001 -o jsonpath='{.status.phase}'
```

#### Watch Resources

```bash
# Watch devices for changes
kubectl watch devices

# Watch applications for changes
kubectl watch applications

# Watch with custom format
kubectl watch applications -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,DEVICES:.status.metrics.running_devices
```

### Updating Resources

#### Patch Resources

```bash
# Patch device status
kubectl patch device hifive1-001 --type='merge' -p='{"status":{"phase":"Connected"}}'

# Patch application configuration
kubectl patch application temperature-monitor --type='merge' -p='{"spec":{"config":{"memory_limit":2097152}}}'
```

#### Scale Applications

```bash
# Scale application to more devices
kubectl patch application temperature-monitor --type='merge' -p='{"spec":{"target_devices":{"device_count":5}}}'
```

### Deleting Resources

```bash
# Delete device
kubectl delete device hifive1-001

# Delete application
kubectl delete application temperature-monitor

# Delete with cascade
kubectl delete application temperature-monitor --cascade=true
```

##  Status Fields

### Device Status Phases

| Phase | Description | Next Actions |
|-------|-------------|--------------|
| `Enrolling` | Device is being enrolled | Wait for enrollment completion |
| `Connected` | Device is connected and ready | Deploy applications |
| `Disconnected` | Device is disconnected | Check connectivity |
| `Failed` | Device enrollment failed | Check logs and retry |

### Application Status Phases

| Phase | Description | Next Actions |
|-------|-------------|--------------|
| `Creating` | Application is being created | Wait for validation |
| `Deploying` | Application is being deployed | Monitor deployment progress |
| `Running` | Application is running on all devices | Monitor performance |
| `PartiallyRunning` | Application is running on some devices | Check failed devices |
| `Stopping` | Application is being stopped | Wait for stop completion |
| `Stopped` | Application is stopped | Restart if needed |
| `Failed` | Application deployment failed | Check logs and retry |
| `Deleting` | Application is being deleted | Wait for cleanup |

##  Validation Rules

### Device Validation

```yaml
# Required fields
- spec.device_type: Must be non-empty string
- spec.capabilities: Must be non-empty array
- spec.public_key: Must be valid base64 string

# Optional validation
- spec.firmware_version: Should match semantic versioning
- spec.hardware_id: Should be unique within namespace
```

### Application Validation

```yaml
# Required fields
- spec.name: Must be non-empty string
- spec.wasm_bytes: Must be valid base64 string
- spec.target_devices: Must specify at least one target

# Validation rules
- spec.config.memory_limit: Must be >= 1024 bytes
- spec.config.cpu_time_limit: Must be >= 100 milliseconds
- spec.config.max_restarts: Must be between 0 and 10
- spec.target_devices.device_count: Must be >= 1
```

##  Advanced Usage

### Device Selectors

```yaml
# Select devices by labels
spec:
  target_devices:
    device_labels:
      device-type: hifive1
      location: lab-1
      environment: production

# Select devices by name
spec:
  target_devices:
    device_names:
      - "hifive1-001"
      - "hifive1-002"
      - "hifive1-003"

# Select devices by count
spec:
  target_devices:
    device_count: 5
    device_labels:
      device-type: hifive1
```

### Application Configuration

```yaml
# Environment variables
spec:
  config:
    environment_vars:
      DATABASE_URL: "postgresql://localhost:5432/wasmbed"
      API_KEY: "secret-api-key"
      LOG_LEVEL: "info"
      DEBUG: "true"

# Application arguments
spec:
  config:
    args:
      - "--config=/etc/app/config.yaml"
      - "--port=8080"
      - "--workers=4"
      - "--verbose"

# Resource limits
spec:
  config:
    memory_limit: 2097152      # 2MB
    cpu_time_limit: 2000       # 2 seconds
    timeout: 10000             # 10 seconds
```

### Status Monitoring

```bash
# Monitor application deployment
kubectl get application temperature-monitor -w

# Check device status
kubectl get device hifive1-001 -o jsonpath='{.status.connection_info}'

# Monitor metrics
kubectl get application temperature-monitor -o jsonpath='{.status.metrics}'
```

##  Security Considerations

### RBAC Permissions

```yaml
# Device permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: wasmbed-device-manager
rules:
- apiGroups: ["wasmbed.github.io"]
  resources: ["devices"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]

# Application permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: wasmbed-application-manager
rules:
- apiGroups: ["wasmbed.github.io"]
  resources: ["applications"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["wasmbed.github.io"]
  resources: ["applications/status"]
  verbs: ["get", "update", "patch"]
```

### Network Policies

```yaml
# Allow controller to access CRDs
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: wasmbed-controller-policy
spec:
  podSelector:
    matchLabels:
      app: wasmbed-k8s-controller
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 443
```

##  Testing CRDs

### Validation Tests

```bash
# Test CRD schema validation
kubectl create -f test-device.yaml --dry-run=client

# Test application validation
kubectl create -f test-application.yaml --dry-run=server

# Test status updates
kubectl patch device test-device --type='merge' -p='{"status":{"phase":"Connected"}}'
```

### Integration Tests

```bash
# Test device enrollment flow
kubectl apply -f test-device.yaml
kubectl wait --for=condition=Ready device/test-device --timeout=60s

# Test application deployment flow
kubectl apply -f test-application.yaml
kubectl wait --for=condition=Ready application/test-app --timeout=120s

# Test cleanup
kubectl delete -f test-application.yaml
kubectl delete -f test-device.yaml
```

##  Best Practices

### Resource Naming

```yaml
# Use consistent naming conventions
metadata:
  name: "hifive1-lab-001"           # device-type-location-number
  name: "temp-monitor-v1.0.0"        # app-name-version
  name: "sensor-gateway-prod"        # app-name-environment
```

### Labels and Annotations

```yaml
metadata:
  labels:
    device-type: hifive1
    location: lab-1
    environment: development
    version: v1.0.0
  annotations:
    wasmbed.github.io/description: "Temperature monitoring application"
    wasmbed.github.io/author: "team-iot"
    wasmbed.github.io/created: "2024-09-01T22:00:00Z"
```

### Resource Limits

```yaml
# Set appropriate resource limits
spec:
  config:
    memory_limit: 1048576      # 1MB for simple apps
    cpu_time_limit: 1000       # 1 second per execution
    timeout: 5000              # 5 seconds total timeout
```

### Error Handling

```yaml
# Configure retry behavior
spec:
  config:
    auto_restart: true
    max_restarts: 3
    timeout: 10000             # 10 seconds timeout
```

---

**Last Updated**: September 2024  
**Version**: CRDs v1alpha1  
**Maintainer**: Wasmbed Development Team
