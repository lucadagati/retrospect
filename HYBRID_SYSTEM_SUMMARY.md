# Wasmbed Hybrid System - QEMU + Simulator

## Sistema Ibrido Implementato

Il sistema Wasmbed ora supporta **sia dispositivi QEMU reali che dispositivi simulati**, offrendo la massima flessibilità per test e sviluppo.

### Architettura Ibrida

```
┌─────────────────────────────────────────────────────────────┐
│                    Wasmbed Platform                        │
├─────────────────────────────────────────────────────────────┤
│  Kubernetes Cluster (k3d)                                  │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   Gateway Pod   │  │ Controller Pod │                  │
│  │  (TLS + HTTP)   │  │  (Reconciliation)│                 │
│  └─────────────────┘  └─────────────────┘                  │
├─────────────────────────────────────────────────────────────┤
│  Device Layer (Hybrid)                                     │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │  QEMU Devices   │  │ Simulated MCUs │                  │
│  │  (Real Hardware)│  │  (Rust Simulator)│                 │
│  │                 │  │                 │                  │
│  │ • qemu-device-1 │  │ • mcu-device-1  │                  │
│  │ • qemu-device-2 │  │ • mcu-device-2  │                  │
│  │                 │  │ • mcu-device-3  │                  │
│  │                 │  │ • mcu-device-4  │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Dispositivi QEMU (Real Hardware Emulation)

**Caratteristiche:**
- **Hardware Reale**: Emulazione completa del chip RISC-V HiFive1
- **Firmware Compilato**: `wasmbed-firmware-hifive1-qemu` per RISC-V
- **Comunicazione Seriale**: Socket Unix per comunicazione bidirezionale
- **Memoria**: 16KB RAM (configurazione HiFive1)
- **Processi**: 2 dispositivi QEMU in esecuzione simultanea

**Configurazione:**
```bash
qemu-system-riscv32 \
    -nographic \
    -monitor none \
    -machine sifive_e,revb=true \
    -serial unix:/tmp/wasmbed-qemu-qemu-device-N.sock,server,nowait \
    -kernel target/riscv32imac-unknown-none-elf/release/wasmbed-firmware-hifive1-qemu \
    -m 16K
```

**Gestione:**
- **Script**: `manage-qemu-devices.sh`
- **Comandi**: `start`, `stop`, `status`, `monitor`, `restart`
- **Log**: `/tmp/wasmbed-qemu-*.log`
- **Socket**: `/tmp/wasmbed-qemu-*.sock`

### Dispositivi Simulati (Rust Simulator)

**Caratteristiche:**
- **Simulazione Software**: `wasmbed-mcu-simulator` in Rust
- **Test Rapidi**: Avvio immediato senza overhead hardware
- **Debugging**: Log dettagliati e controllo completo
- **Scalabilità**: Facilmente espandibile per test di massa
- **Compatibilità**: Stesso protocollo dei dispositivi reali

**Funzionalità:**
- Generazione chiavi RSA
- Connessione TLS al gateway
- Enrollment automatico
- Heartbeat periodico
- Esecuzione applicazioni WASM
- Comunicazione microROS

### Risorse Kubernetes

**Dispositivi Totali: 6**
- **QEMU**: `qemu-device-1`, `qemu-device-2`
- **Simulati**: `mcu-device-1`, `mcu-device-2`, `mcu-device-3`, `mcu-device-4`

**Applicazioni:**
- **microROS**: `microros-px4-bridge` (deployata)

### Script di Gestione

#### 1. `manage-qemu-devices.sh`
Gestisce i dispositivi QEMU reali:
```bash
./manage-qemu-devices.sh start    # Avvia dispositivi QEMU
./manage-qemu-devices.sh stop     # Ferma tutti i dispositivi QEMU
./manage-qemu-devices.sh status   # Verifica stato dispositivi QEMU
./manage-qemu-devices.sh monitor  # Monitora dispositivi QEMU
./manage-qemu-devices.sh restart  # Riavvia dispositivi QEMU
```

#### 2. `test-hybrid-system.sh`
Testa il sistema ibrido completo:
```bash
./test-hybrid-system.sh comprehensive  # Test completo
./test-hybrid-system.sh qemu           # Solo dispositivi QEMU
./test-hybrid-system.sh simulated      # Solo dispositivi simulati
./test-hybrid-system.sh gateway        # Solo gateway
./test-hybrid-system.sh devices        # Solo enrollment dispositivi
./test-hybrid-system.sh apps           # Solo applicazioni
./test-hybrid-system.sh simulator      # Solo simulatore MCU
./test-hybrid-system.sh serial         # Solo comunicazione seriale QEMU
```

#### 3. `test-mcu-client.sh`
Testa i dispositivi simulati:
```bash
./test-mcu-client.sh  # Test completo dispositivi simulati
```

### Stato Attuale del Sistema

**✅ Sistema Operativo:**
- Kubernetes cluster (k3d) funzionante
- Gateway pod in esecuzione
- Controller pod in esecuzione
- Namespace `wasmbed` configurato

**✅ Dispositivi QEMU:**
- 2 dispositivi QEMU in esecuzione
- Socket seriali attivi
- Log di debug disponibili
- Risorse Kubernetes create

**✅ Dispositivi Simulati:**
- 4 dispositivi simulati configurati
- Simulatore MCU compilato e funzionante
- Test di connettività completati

**✅ Gateway:**
- API HTTP accessibile (porta 30080)
- Porta TLS accessibile (porta 30423)
- Health check funzionante

**✅ Applicazioni:**
- microROS-PX4-Bridge deployata
- Runtime WASM funzionante
- Comunicazione DDS attiva

### Vantaggi del Sistema Ibrido

1. **Flessibilità**: Test sia su hardware reale che simulato
2. **Velocità**: Simulatore per test rapidi, QEMU per test approfonditi
3. **Debugging**: Controllo completo su entrambi i tipi
4. **Scalabilità**: Facile aggiunta di nuovi dispositivi
5. **Affidabilità**: Ridondanza e fallback tra tipi di dispositivo

### Comandi di Utilizzo

**Avvio Sistema Completo:**
```bash
# Avvia dispositivi QEMU
./manage-qemu-devices.sh start

# Testa sistema completo
./test-hybrid-system.sh comprehensive

# Testa dispositivi simulati
./test-mcu-client.sh
```

**Monitoraggio:**
```bash
# Stato dispositivi QEMU
./manage-qemu-devices.sh status

# Monitoraggio QEMU
./manage-qemu-devices.sh monitor

# Test specifici
./test-hybrid-system.sh qemu
./test-hybrid-system.sh simulated
```

**Gestione:**
```bash
# Riavvio QEMU
./manage-qemu-devices.sh restart

# Fermata QEMU
./manage-qemu-devices.sh stop

# Verifica risorse Kubernetes
kubectl get devices -n wasmbed
kubectl get applications -n wasmbed
```

### Risultati Test

**Test Completo Eseguito:**
- ✅ Kubernetes cluster funzionante
- ✅ Namespace `wasmbed` esistente
- ✅ Gateway pod in esecuzione
- ✅ Controller pod in esecuzione
- ✅ 2 dispositivi QEMU attivi
- ✅ 4 dispositivi simulati configurati
- ✅ Gateway HTTP API accessibile
- ✅ Gateway TLS port accessibile
- ✅ 1 applicazione microROS deployata
- ✅ Simulatore MCU funzionante
- ✅ 2 socket seriali QEMU attivi
- ✅ Log QEMU disponibili

**Sistema Ibrido: 100% Operativo** 🎉

Il sistema Wasmbed ora supporta completamente sia l'emulazione hardware QEMU che la simulazione software, offrendo la massima flessibilità per sviluppo, test e deployment di applicazioni IoT e microROS.
