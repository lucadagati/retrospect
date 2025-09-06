# Wasmbed Platform Architecture

## Overview

Wasmbed è una piattaforma completa per il deployment e l'esecuzione di applicazioni WebAssembly su dispositivi IoT edge, progettata per fornire un'architettura scalabile, sicura e performante per l'edge computing.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Wasmbed Platform                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐ │
│  │   Kubernetes    │    │     Gateway     │    │ MCU Devices │ │
│  │   Control Plane │◄──►│      (MPU)       │◄──►│ (ESP32/RISC-V)│ │
│  │                 │    │                 │    │             │ │
│  │ • Device CRDs   │    │ • HTTP API      │    │ • WASM      │ │
│  │ • App CRDs      │    │ • TLS/CBOR      │    │   Runtime   │ │
│  │ • Controller    │    │ • Security      │    │ • Firmware  │ │
│  │ • Monitoring    │    │ • Monitoring    │    │ • Hardware  │ │
│  └─────────────────┘    └─────────────────┘    └──────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Kubernetes Control Plane

Il control plane Kubernetes gestisce l'orchestrazione di dispositivi e applicazioni attraverso Custom Resource Definitions (CRDs).

#### Custom Resources

**Device Resource**
```yaml
apiVersion: wasmbed.github.io/v1
kind: Device
metadata:
  name: device-name
  namespace: wasmbed
spec:
  deviceId: "unique-device-id"
  publicKey: "base64-encoded-public-key"
  deviceType: "hifive1" | "esp32"
  capabilities: ["wasm", "tls", "wifi"]
  state: "online" | "offline" | "error"
  lastSeen: "2025-01-27T10:00:00Z"
  applications: ["app1", "app2"]
```

**Application Resource**
```yaml
apiVersion: wasmbed.github.io/v1
kind: Application
metadata:
  name: app-name
  namespace: wasmbed
spec:
  name: "application-name"
  wasmBinary: "base64-encoded-wasm-binary"
  targetDevices: ["device1", "device2"]
  config:
    memoryLimit: 1048576
    executionTimeout: "30s"
    maxStackSize: 65536
  state: "deployed" | "running" | "stopped" | "error"
  deployedDevices: ["device1"]
  lastDeployed: "2025-01-27T10:00:00Z"
```

#### Controller Logic

Il controller Kubernetes monitora i CRDs e gestisce:
- **Device Lifecycle**: Registrazione, autenticazione, monitoraggio
- **Application Deployment**: Distribuzione, avvio, fermata, aggiornamento
- **State Management**: Sincronizzazione stato tra Gateway e dispositivi
- **Error Handling**: Gestione errori e recovery automatico

### 2. Gateway (MPU)

Il Gateway funge da intermediario tra il control plane Kubernetes e i dispositivi MCU, fornendo:

#### HTTP API Server
- **RESTful API** per gestione dispositivi e applicazioni
- **Health checks** e monitoring
- **Metrics collection** e reporting
- **Configuration management**

#### Communication Protocol
- **TLS/TLS 1.3** per sicurezza end-to-end
- **CBOR** per serializzazione efficiente
- **Ed25519** per firma digitale dei messaggi
- **AES-256-GCM** per crittografia dei dati sensibili

#### Security Layer
- **Certificate Management**: Gestione certificati TLS
- **Authentication**: Verifica identità dispositivi
- **Authorization**: Controllo accessi basato su ruoli
- **Encryption**: Crittografia messaggi e dati

#### Monitoring System
- **Performance Metrics**: CPU, memoria, rete
- **Application Metrics**: Esecuzione, errori, throughput
- **Device Health**: Stato, connettività, capacità
- **Alerting**: Notifiche per eventi critici

### 3. MCU Devices

I dispositivi MCU eseguono il firmware Wasmbed e forniscono:

#### WASM Runtime
- **RISC-V (HiFive1 QEMU)**: Runtime personalizzato compatibile con `no_std`
- **ESP32**: Runtime basato su `wasmi` per ambienti standard
- **Memory Management**: Gestione memoria con limiti configurabili
- **Execution Engine**: Esecuzione sicura di codice WASM

#### Firmware Components
- **Security Manager**: Gestione chiavi, firme, crittografia
- **Application Manager**: Lifecycle delle applicazioni WASM
- **WiFi Manager**: Gestione connettività di rete
- **Monitoring System**: Raccolta metriche e reporting
- **Wasmbed Client**: Comunicazione con Gateway

#### Hardware Abstraction
- **Device Drivers**: Interfaccia con hardware specifico
- **Peripheral Management**: GPIO, UART, SPI, I2C
- **Power Management**: Gestione consumo energetico
- **Real-time Operations**: Operazioni time-critical

## Communication Flow

### 1. Device Registration
```
Device → Gateway → Kubernetes
  ↓         ↓         ↓
1. Send   2. Verify  3. Create
   reg.      auth.     Device CRD
  ↓         ↓         ↓
4. Receive 5. Update  6. Monitor
   cert.     status    device
```

### 2. Application Deployment
```
Kubernetes → Gateway → Device
     ↓         ↓         ↓
1. Create   2. Route   3. Receive
   App CRD    to dev.   deployment
     ↓         ↓         ↓
4. Monitor  5. Track   6. Execute
   status     progress  WASM app
```

### 3. Runtime Communication
```
Device ←→ Gateway ←→ Kubernetes
   ↓         ↓         ↓
1. Send    2. Process  3. Update
   metrics   & store    CRDs
   ↓         ↓         ↓
4. Receive 5. Forward  6. Monitor
   config    updates    system
```

## Security Architecture

### 1. Certificate Chain
```
Root CA
   ↓
Intermediate CA
   ↓
Gateway Certificate
   ↓
Device Certificates
```

### 2. Message Security
```
Message → Sign → Encrypt → Send
   ↓         ↓        ↓       ↓
Plaintext  Ed25519  AES-256  TLS
           Signature  GCM    1.3
```

### 3. Authentication Flow
```
Device → Gateway → Kubernetes
  ↓         ↓         ↓
1. Send   2. Verify  3. Check
   cert.    cert.     auth.
  ↓         ↓         ↓
4. Receive 5. Grant   6. Allow
   token    access     operations
```

## Performance Considerations

### 1. Scalability
- **Horizontal Scaling**: Gateway repliche per carichi elevati
- **Load Balancing**: Distribuzione carico tra Gateway
- **Resource Management**: Limitazione risorse per dispositivo/app
- **Caching**: Cache locale per ridurre latenza

### 2. Optimization
- **Connection Pooling**: Riutilizzo connessioni TLS
- **Message Batching**: Aggregazione messaggi per efficienza
- **Compression**: Compressione dati per ridurre bandwidth
- **Async Processing**: Elaborazione asincrona per throughput

### 3. Monitoring
- **Real-time Metrics**: Metriche in tempo reale
- **Performance Profiling**: Profiling dettagliato
- **Resource Usage**: Monitoraggio utilizzo risorse
- **Alerting**: Notifiche per performance degradation

## Deployment Architecture

### 1. Kubernetes Cluster
```yaml
# Cluster Configuration
apiVersion: v1
kind: Namespace
metadata:
  name: wasmbed
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasmbed-gateway
  namespace: wasmbed
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wasmbed-gateway
  template:
    spec:
      containers:
      - name: gateway
        image: wasmbed-gateway:latest
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

### 2. Gateway Deployment
```yaml
# Gateway Service
apiVersion: v1
kind: Service
metadata:
  name: wasmbed-gateway
  namespace: wasmbed
spec:
  selector:
    app: wasmbed-gateway
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: https
    port: 8443
    targetPort: 8443
  type: LoadBalancer
```

### 3. Device Firmware
```rust
// Firmware Structure
pub struct WasmbedFirmware {
    runtime: WasmRuntime,
    security: SecurityManager,
    applications: ApplicationManager,
    wifi: WifiManager,
    monitoring: MonitoringSystem,
    client: WasmbedClient,
}
```

## Error Handling and Recovery

### 1. Error Categories
- **Network Errors**: Connessione, timeout, disconnessione
- **Authentication Errors**: Certificati, chiavi, autorizzazione
- **Runtime Errors**: WASM execution, memoria, stack overflow
- **System Errors**: Hardware, firmware, configurazione

### 2. Recovery Strategies
- **Automatic Retry**: Retry automatico per errori temporanei
- **Failover**: Switch a Gateway alternativi
- **Graceful Degradation**: Riduzione funzionalità invece di crash
- **State Recovery**: Ripristino stato dopo errori

### 3. Monitoring and Alerting
- **Health Checks**: Verifica stato componenti
- **Error Tracking**: Tracciamento errori e pattern
- **Performance Monitoring**: Monitoraggio performance
- **Alerting**: Notifiche per eventi critici

## Future Enhancements

### 1. Planned Features
- **Multi-Cloud Support**: Supporto per cloud multipli
- **Edge-to-Edge Communication**: Comunicazione diretta tra dispositivi
- **Advanced Analytics**: Analytics avanzate per IoT
- **Machine Learning**: Integrazione ML per ottimizzazione

### 2. Scalability Improvements
- **Federation**: Federazione cluster Kubernetes
- **Edge Computing**: Computing distribuito edge
- **5G Integration**: Integrazione con reti 5G
- **IoT Protocols**: Supporto protocolli IoT standard

### 3. Security Enhancements
- **Zero Trust**: Architettura zero trust
- **Hardware Security**: Security hardware-based
- **Quantum Resistance**: Resistenza a computer quantistici
- **Compliance**: Conformità standard sicurezza

---

Questa architettura fornisce una base solida per il deployment e la gestione di applicazioni WebAssembly su dispositivi IoT edge, con particolare attenzione alla sicurezza, scalabilità e performance.
