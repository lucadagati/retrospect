# Wasmbed Platform

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-1.28+-blue.svg)](https://kubernetes.io/)
[![WebAssembly](https://img.shields.io/badge/webassembly-wasm-purple.svg)](https://webassembly.org/)

Una piattaforma completa per il deployment e l'esecuzione di applicazioni WebAssembly su dispositivi IoT edge, utilizzando Kubernetes come control plane e Gateway come intermediari per la comunicazione con i dispositivi MCU.

## 🚀 Caratteristiche Principali

- **🌐 Kubernetes Integration**: Orchestrazione completa tramite Custom Resource Definitions (CRDs)
- **🔒 Security First**: TLS 1.3, Ed25519 signatures, AES-256-GCM encryption
- **⚡ High Performance**: Runtime WASM ottimizzato per dispositivi MCU
- **📱 Multi-Platform**: Supporto per ESP32 e RISC-V (HiFive1)
- **🔧 Easy Deployment**: Script automatizzati per setup e testing
- **📊 Comprehensive Monitoring**: Metriche dettagliate e alerting
- **🧪 Extensive Testing**: Test end-to-end completi

## 🏗️ Architettura

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Kubernetes    │    │     Gateway     │    │   MCU Devices   │
│   Control Plane │◄──►│      (MPU)      │◄──►│   (ESP32/RISC-V)│
│                 │    │                 │    │                 │
│ • Device CRDs   │    │ • HTTP API      │    │ • WASM Runtime  │
│ • App CRDs      │    │ • TLS/CBOR      │    │ • Firmware      │
│ • Controller    │    │ • Security      │    │ • Hardware      │
│ • Monitoring    │    │ • Monitoring    │    │ • Communication│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📋 Prerequisiti

- **Rust** 1.75+
- **Kubernetes** 1.28+ (k3d consigliato)
- **QEMU** per emulazione RISC-V
- **Docker** per containerizzazione
- **OpenSSL** per generazione certificati

## 🚀 Quick Start

### 1. Clona il repository
```bash
git clone https://github.com/your-org/wasmbed.git
cd wasmbed
```

### 2. Installa le dipendenze
```bash
# Installa Rust (se non già installato)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Installa k3d
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Installa QEMU
sudo apt-get install qemu-system-misc
```

### 3. Genera certificati TLS
```bash
./scripts/generate-certs.sh
```

### 4. Avvia la piattaforma
```bash
# Compila tutto
cargo build --workspace

# Avvia cluster Kubernetes
k3d cluster create wasmbed-test

# Deploy Gateway
kubectl apply -f gateway-deployment.yaml

# Deploy test resources
kubectl apply -f test-device.yaml
kubectl apply -f test-wasm-app.yaml
```

### 5. Testa il sistema
```bash
# Esegui tutti i test
./scripts/run-all-tests.sh

# Oppure testa manualmente
cargo test --workspace
```

## 📚 Documentazione

- **[API Documentation](docs/API_DOCUMENTATION.md)**: Documentazione completa delle API
- **[Architecture](docs/ARCHITECTURE.md)**: Architettura dettagliata della piattaforma
- **[Examples](examples/)**: Esempi di utilizzo e configurazione
- **[Tests](tests/)**: Suite di test completa

## 🔧 Componenti

### Kubernetes Control Plane
- **Device CRD**: Gestione dispositivi IoT
- **Application CRD**: Gestione applicazioni WASM
- **Controller**: Orchestrazione automatica
- **Monitoring**: Metriche e alerting

### Gateway (MPU)
- **HTTP API**: RESTful API per gestione
- **TLS/CBOR**: Comunicazione sicura ed efficiente
- **Security**: Autenticazione e autorizzazione
- **Monitoring**: Raccolta metriche sistema

### MCU Devices
- **RISC-V (HiFive1)**: Runtime WASM personalizzato per `no_std`
- **ESP32**: Runtime WASM basato su `wasmi`
- **Firmware**: Gestione applicazioni e comunicazione
- **Hardware**: Interfaccia con periferiche

## 🧪 Testing

### Test Unitari
```bash
cargo test --workspace --lib
```

### Test di Integrazione
```bash
cargo test --manifest-path tests/Cargo.toml
```

### Test End-to-End
```bash
./scripts/run-all-tests.sh
```

### Test Manuali
```bash
# Test Gateway
curl -k https://localhost:8443/health

# Test Kubernetes
kubectl get devices -n wasmbed
kubectl get applications -n wasmbed

# Test QEMU
qemu-system-riscv32 -machine sifive_e -kernel target/riscv32imac-unknown-none-elf/debug/wasmbed-firmware-hifive1-qemu -nographic
```

## 🔒 Sicurezza

### Certificati TLS
- **CA Certificate**: `/etc/wasmbed/ca-cert.pem`
- **Server Certificate**: `/etc/wasmbed/server-cert.pem`
- **Server Private Key**: `/etc/wasmbed/server-key.pem`

### Crittografia
- **TLS 1.3**: Comunicazione sicura
- **Ed25519**: Firma digitale messaggi
- **AES-256-GCM**: Crittografia dati sensibili

### Autenticazione
- **Certificate-based**: Autenticazione basata su certificati
- **Public Key**: Verifica identità dispositivi
- **RBAC**: Controllo accessi basato su ruoli

## 📊 Monitoring

### Metriche Sistema
- **Devices**: Totale, online, offline
- **Applications**: Totale, running, stopped
- **Performance**: Latenza, throughput, errori

### Metriche Dispositivo
- **CPU Usage**: Utilizzo processore
- **Memory Usage**: Utilizzo memoria
- **Network**: Traffico di rete
- **Power**: Consumo energetico

### Alerting
- **Health Checks**: Verifica stato componenti
- **Error Tracking**: Tracciamento errori
- **Performance**: Degradazione performance
- **Security**: Eventi di sicurezza

## 🚀 Deployment

### Kubernetes
```yaml
# Namespace
apiVersion: v1
kind: Namespace
metadata:
  name: wasmbed

---
# Gateway Deployment
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
        ports:
        - containerPort: 8080
        - containerPort: 8443
```

### Docker
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/wasmbed-gateway /usr/local/bin/
COPY certs/ /etc/wasmbed/
EXPOSE 8080 8443
CMD ["wasmbed-gateway"]
```

## 🤝 Contribuire

1. **Fork** il repository
2. **Crea** un branch per la feature (`git checkout -b feature/amazing-feature`)
3. **Commit** le modifiche (`git commit -m 'Add amazing feature'`)
4. **Push** al branch (`git push origin feature/amazing-feature`)
5. **Apri** una Pull Request

### Guidelines
- Segui le convenzioni di codice Rust
- Aggiungi test per nuove funzionalità
- Aggiorna la documentazione
- Mantieni la compatibilità con le versioni esistenti

## 📄 Licenza

Questo progetto è rilasciato sotto licenza [AGPL-3.0](LICENSE).

## 🆘 Supporto

- **Issues**: [GitHub Issues](https://github.com/your-org/wasmbed/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/wasmbed/discussions)
- **Documentation**: [docs/](docs/)
- **Examples**: [examples/](examples/)

## 🗺️ Roadmap

### v0.2.0 (Prossima)
- [ ] Supporto ESP32 completo con wasmi
- [ ] Dashboard web per monitoring
- [ ] API GraphQL per query avanzate
- [ ] Supporto protocolli IoT standard

### v0.3.0 (Futuro)
- [ ] Multi-cloud deployment
- [ ] Edge-to-edge communication
- [ ] Machine learning integration
- [ ] 5G network support

## 🙏 Ringraziamenti

- **Rust Community** per l'ecosistema eccellente
- **Kubernetes** per l'orchestrazione
- **WebAssembly** per il runtime universale
- **Contributors** per il supporto e feedback

---

**Wasmbed** - Portando WebAssembly all'edge computing 🚀