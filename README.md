# Wasmbed Platform

Piattaforma Kubernetes-native per il deployment di applicazioni WebAssembly su dispositivi embedded con emulazione Renode.

## Quick Start

```bash
# Setup ambiente completo
./scripts/quick-setup.sh

# Oppure step-by-step:
# 1. Setup ambiente
./scripts/setup-zephyr-workspace.sh

# 2. Build componenti
./scripts/02-build-components.sh

# 3. Deploy infrastruttura
./scripts/03-deploy-infrastructure.sh

# 4. Verifica stato
./scripts/04-check-system-status.sh

# 5. Esegui tutti i test
./scripts/run-all-tests.sh
```

## Documentazione

La documentazione completa è disponibile in [`doc/`](doc/):

- **[README.md](doc/README.md)**: Panoramica generale e guida introduttiva
- **[ARCHITECTURE.md](doc/ARCHITECTURE.md)**: Architettura del sistema
- **[FIRMWARE.md](doc/FIRMWARE.md)**: Documentazione firmware Zephyr
- **[WASMBED_CAPABILITIES.md](doc/WASMBED_CAPABILITIES.md)**: Capacità e funzionalità dettagliate
- **[SEQUENCE_DIAGRAMS.md](doc/SEQUENCE_DIAGRAMS.md)**: Diagrammi di sequenza

## Testing

La piattaforma include una suite completa di test:

```bash
# Esegui tutti i test
./scripts/run-all-tests.sh

# Test API dashboard (45 endpoint, verifica con kubectl)
export API_BASE_URL="http://100.103.160.17:3000/api"
./scripts/test-dashboard-apis.sh
```

Vedi [scripts/TEST_REPORT.md](scripts/TEST_REPORT.md) e [scripts/API_TEST_REPORT.md](scripts/API_TEST_REPORT.md) per i dettagli.

## Componenti Principali

- **API Server**: REST API e Kubernetes controllers (45+ endpoint testati)
- **Gateway**: Server TLS per comunicazione con dispositivi
- **Renode Manager**: Gestione emulazione Renode (precedentemente QEMU Manager)
- **TCP Bridge**: Tunneling TLS tra dispositivo e gateway
- **Firmware Zephyr**: RTOS con WAMR runtime
- **Dashboard React**: Interfaccia web per gestione dispositivi e applicazioni

## Tecnologie

- **Renode**: Emulazione hardware ARM Cortex-M
- **Zephyr RTOS**: Sistema operativo real-time
- **WAMR**: Runtime WebAssembly
- **Kubernetes**: Orchestrazione e gestione
- **TLS/CBOR**: Comunicazione sicura

## Licenza

AGPL-3.0
