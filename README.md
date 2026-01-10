# Wasmbed Platform

Piattaforma Kubernetes-native per il deployment di applicazioni WebAssembly su dispositivi embedded con emulazione Renode.

## Quick Start

```bash
# Setup ambiente
./scripts/setup-zephyr-workspace.sh

# Build componenti
./scripts/02-build-components.sh

# Deploy infrastruttura
./scripts/03-deploy-infrastructure.sh

# Verifica stato
./scripts/04-check-system-status.sh
```

## Documentazione

La documentazione completa è disponibile in [`doc/`](doc/):

- **[README.md](doc/README.md)**: Panoramica generale e guida introduttiva
- **[ARCHITECTURE.md](doc/ARCHITECTURE.md)**: Architettura del sistema
- **[FIRMWARE.md](doc/FIRMWARE.md)**: Documentazione firmware Zephyr
- **[DEPLOYMENT.md](doc/DEPLOYMENT.md)**: Guida al deployment

## Componenti Principali

- **API Server**: REST API e Kubernetes controllers
- **Gateway**: Server TLS per comunicazione con dispositivi
- **QEMU Manager**: Gestione emulazione Renode
- **TCP Bridge**: Tunneling TLS tra dispositivo e gateway
- **Firmware Zephyr**: RTOS con WAMR runtime

## Tecnologie

- **Renode**: Emulazione hardware ARM Cortex-M
- **Zephyr RTOS**: Sistema operativo real-time
- **WAMR**: Runtime WebAssembly
- **Kubernetes**: Orchestrazione e gestione
- **TLS/CBOR**: Comunicazione sicura

## Licenza

AGPL-3.0
