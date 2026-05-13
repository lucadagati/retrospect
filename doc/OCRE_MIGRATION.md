# RETROSPECT — Migrazione OCRE: Architettura, Flusso e Risultati

**Data**: 13 maggio 2026  
**Autore**: Antonio Ciliberto  

---

## 1. Panoramica del sistema

**RETROSPECT** (*secuRE inTegration middlewaRe fOr cpS comPutE ConTinuum*) è una piattaforma Kubernetes-native per il deployment e la gestione di applicazioni WebAssembly su dispositivi embedded attraverso il continuum Cloud–Fog–Edge.

Il sistema consente a un operatore di:
1. registrare dispositivi IoT/embedded in un cluster Kubernetes tramite enrollment TLS mutuo;
2. caricare moduli WebAssembly dalla dashboard;
3. farli eseguire sul dispositivo edge via un runtime WASM standard industriale (WAMR).

Questo documento descrive lo stato attuale del sistema **dopo la migrazione a OCRE**, avvenuta contestualmente a questo lavoro di tesi.

---

## 2. Stack tecnologico — versioni esatte, tutte ufficiali

| Componente | Progetto upstream | Versione / commit |
|---|---|---|
| **OCRE Runtime** | `github.com/project-ocre/ocre-runtime` | `main` @ `8afde85` |
| **Zephyr RTOS** | `zephyrproject-rtos/zephyr` | `v4.4.0` (tag ufficiale) |
| **Zephyr SDK** | `zephyrproject-rtos/sdk-ng` | `1.0.1` |
| **WAMR** | bundled da OCRE (`wasm-micro-runtime`) | `WAMR-2.4.1-240-gd1a577ea` |
| **west** | Zephyr meta-tool | `1.5.0` |
| **Rust workspace** | — | `1.88.0` (edition 2021) |
| **K3s** | `k3s-io/k3s` | in esecuzione su host Ubuntu |

**Nessuna patch custom** è stata applicata a nessuno di questi progetti. Il workspace OCRE viene inizializzato tramite `west init -m https://github.com/project-ocre/ocre-runtime`, che porta automaticamente Zephyr 4.4.0 e WAMR tramite il file `west.yml` ufficiale di OCRE. Il vecchio submodule `wamr/` con 4 patch downstream per Zephyr 3.5 è stato rimosso integralmente.

---

## 3. Cosa è cambiato rispetto alla versione precedente

| Aspetto | Prima (Zephyr 3.5 era) | Dopo (OCRE era) |
|---|---|---|
| RTOS | Zephyr 3.5.0 | Zephyr **4.4.0** |
| Runtime WASM | WAMR integrato manualmente, 4 patch custom | WAMR **bundled da OCRE**, nessuna patch |
| Board target E2E | STM32F746G Discovery (fisico/Renode) | `native_sim/native/64` (OCRE-native) |
| Orchestrazione container WASM | WAMR API diretta nel firmware | **OCRE container lifecycle API** (`ocre_container_runtime_*`) |
| Patch K8s | — | Bug fix: field naming snake_case nei patch di status CRD |
| Renode | Richiesto per emulazione STM32 | **Non necessario** per native_sim |

---

## 4. Architettura attuale

```
┌─────────────────────────────────────────────────────────────────┐
│  CLOUD — K3s cluster (namespace: wasmbed)                        │
│                                                                   │
│  Dashboard (React :3000)  →  API Server (Rust :3001)            │
│                                    │                              │
│          Device CRD │ Application CRD │ Gateway CRD              │
│                                    │                              │
│   Device Controller │ App Controller │ Gateway Controller        │
│             (Rust, kube-rs, watch loop)                          │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  wasmbed-gateway  (Rust)                                 │    │
│  │  :8080 HTTP northbound  │  :8081 TLS southbound          │    │
│  └─────────────────────────────────────────────────────────┘    │
└───────────────────────────────────┬─────────────────────────────┘
                                    │ TLS 1.3 + CBOR (minicbor)
                              Ed25519 key exchange
                                    │
┌───────────────────────────────────▼─────────────────────────────┐
│  EDGE — native_sim/native/64 (processo Linux x86-64)            │
│                                                                   │
│  Zephyr 4.4.0  ──►  OCRE Runtime  ──►  WAMR 2.4.x              │
│  (TLS 1.3 client)   (container mgmt)   (WASM execution)         │
│                                                                   │
│  firmware: ocre-workspace/build/native_sim_64/zephyr/zephyr.exe │
└─────────────────────────────────────────────────────────────────┘
```

### Componenti cloud

| Pod | Porta | Ruolo |
|---|---|---|
| `wasmbed-dashboard` | 3000 | UI React — upload WASM, visualizzazione stato |
| `wasmbed-api-server` | 3001 | REST API, gestione CRD, avvio emulatori |
| `gateway-1` | 8080/8081 | Hub TLS+CBOR; unico punto di contatto con i device |
| `wasmbed-device-controller` | — | Reconciliation loop Device CRD |
| `wasmbed-application-controller` | — | Reconciliation loop Application CRD, triggera deploy WASM |
| `wasmbed-gateway-controller` | — | Reconciliation loop Gateway CRD |

### Target edge

`native_sim/native/64` è il target di emulazione **nativo di OCRE**: il firmware compila come processo Linux a 64 bit (`zephyr.exe`) e gira direttamente sull'host senza Renode, QEMU o hardware fisico. È il target su cui OCRE effettua i propri test di integration nella CI upstream. Non richiede nessun driver di periferica né setup di rete esterno.

---

## 5. Protocollo di comunicazione Cloud ↔ Edge

**Stack**: TCP → TLS 1.3 → CBOR (`minicbor`, `no_std`-compatible)

```
Envelope {
  version:    V0 (0)
  message_id: u32 (wrapping counter per correlazione)
  message:    ClientMessage | ServerMessage
}
```

**Autenticazione**: il device invia la propria chiave pubblica Ed25519 (32 byte raw, codificata URL_SAFE_NO_PAD in base64 nel CRD `Device.spec.publicKey`) tramite un messaggio CBOR `PublicKey` dopo l'enrollment. Il gateway la confronta con tutte le `Device` CRD e, trovata la corrispondenza, aggiorna lo status via Kubernetes Merge Patch.

---

## 6. Flusso E2E verificato

### 6.1 Enrollment e connessione

```
1. Utente crea Device CRD (phase: Pending) via Dashboard o kubectl
2. API Server (se MCU fisico/Renode): avvia processo emulato
   Per native_sim: il firmware viene avviato manualmente o via script
3. zephyr.exe boots → legge endpoint gateway → TLS connect :8081
4. Firmware → CBOR: EnrollmentRequest
5. Gateway → CBOR: EnrollmentAccepted
6. Firmware → CBOR: PublicKey (raw 32 bytes, chiave statica 0xAB×32 per native_sim)
7. Gateway: trova Device CRD con publicKey = "q6urq6ur..." (base64 della chiave)
8. Gateway → K8s API: PATCH /status  { phase: "Connected", gateway: {name: "gateway-1"}, 
                                         connected_since: <now>, last_heartbeat: <now> }
9. Device CRD: phase = "Connected" ✓
10. Firmware → CBOR: Heartbeat ogni ~30s
11. Gateway → K8s API: PATCH /status  { last_heartbeat: <now> }
```

**Nota tecnica sul Merge Patch K8s**: il gateway invia un payload JSON parziale all'API di stato del CRD tramite la libreria `kube-rs` (`api.patch_status(..., &Patch::Merge(...))`). Questo è uno standard Kubernetes (`application/merge-patch+json`, RFC 7386) — non una modifica a Kubernetes. Il bug corretto durante questo lavoro era che i nomi dei campi nel payload erano in camelCase (`connectedSince`) mentre lo schema CRD li attende in snake_case (`connected_since`); Kubernetes scartava silenziosamente i campi non riconosciuti.

### 6.2 Deploy applicazione WASM

```
1. Utente carica modulo .wasm dalla Dashboard (POST /api/v1/applications)
2. API Server crea Application CRD (spec.wasm_binary, spec.target_devices: ["native-sim-1"])
3. Application Controller (watch loop) rileva nuova Application CRD
4. Application Controller → Gateway HTTP: POST /api/v1/devices/native-sim-1/deploy
5. Gateway legge wasm_binary dalla Application CRD
6. Gateway → TLS socket: ServerMessage::Deploy (payload WASM raw bytes in CBOR Envelope)
7. zephyr.exe riceve Deploy:
   a. ocre_container_data: { wasm_bytes, wasm_size }
   b. ocre_container_runtime_create()   — OCRE alloca container WASM isolato
   c. ocre_container_runtime_run()      — WAMR esegue il modulo
8. Firmware → CBOR: DeployAck (successo)
9. Gateway → K8s API: PATCH Application/status
   { phase: "Running", deviceStatuses: { "native-sim-1": { status: "Running" } } }
10. Dashboard mostra Application phase = "Running" ✓
```

---

## 7. Test effettuati e risultati

### Ambiente di test

- **Host**: Ubuntu (K3s + Docker)
- **Target firmware**: `native_sim/native/64` (processo Linux, Zephyr 4.4.0 + OCRE)
- **Modulo WASM usato**: `hello_world.wasm` da `ocre-runtime/samples/container_runtime/wasm_apps/` (campione ufficiale OCRE)

### Risultati

| Test | Risultato |
|---|---|
| Build firmware `west build -b native_sim/native/64` | ✓ Zero warning/errori |
| Connessione TLS firmware → gateway | ✓ |
| Enrollment CBOR e riconoscimento device da CRD | ✓ |
| `Device.status.phase` → `"Connected"` | ✓ |
| Heartbeat periodico → `last_heartbeat` aggiornato | ✓ |
| Upload WASM via Dashboard | ✓ |
| Application Controller → deploy via Gateway HTTP | ✓ |
| Ricezione payload WASM sul firmware | ✓ |
| OCRE container create + run | ✓ |
| WAMR esegue hello_world.wasm, exit code 0 | ✓ |
| `Application.status.phase` → `"Running"` | ✓ |
| `Application.status.deviceStatuses["native-sim-1"]` → `"Running"` | ✓ |

**Output firmware durante esecuzione WASM** (estratto da log):
```
[OCRE] Container runtime initialized
[OCRE]  ___   ____ ____  _____
[OCRE] / _ \ / ___|  _ \| ____|
[OCRE]| | | | |   | |_) |  _|
[OCRE]| |_| | |___|  _ <| |___
[OCRE] \___/ \____|_| \_\_____|
[OCRE]
[OCRE] powered by Ocre
...
[OCRE] Context completed successfully
[OCRE] Container exited with code 0
```

**Stato cluster al termine dei test** (`kubectl get pods -n wasmbed`):
```
gateway-1-deployment-*              Running
wasmbed-api-server-*               Running
wasmbed-application-controller-*   Running
wasmbed-dashboard-*                Running
wasmbed-device-controller-*        Running
wasmbed-gateway-controller-*       Running
```

```
kubectl get device native-sim-1 -n wasmbed -o jsonpath='{.status.phase}'
→ Connected

kubectl get application hello-ocre -n wasmbed -o jsonpath='{.status.phase}'
→ Running
```

---

## 8. Conferma stack ufficiale — nessuna patch custom

Di seguito la verifica esplicita dell'assenza di modifiche a codice upstream:

| Progetto | Come viene usato | Patch custom |
|---|---|---|
| `project-ocre/ocre-runtime` | `west init -m`, `west update` | **Nessuna** |
| `zephyrproject-rtos/zephyr` | pinned da `ocre-runtime/west.yml` a `v4.4.0` | **Nessuna** |
| `zephyrproject-rtos/sdk-ng` | installer ufficiale SDK 1.0.1 | **Nessuna** |
| `wasm-micro-runtime` (WAMR) | bundled dentro `ocre-runtime` come submodule | **Nessuna** |
| `kube-rs` | dipendenza Cargo standard | **Nessuna** |
| `rustls` / `minicbor` | dipendenze Cargo standard | **Nessuna** |

Il vecchio submodule `wamr/` (contenente 4 patch per la compatibilità con Zephyr 3.5 e un'integrazione manuale nel firmware) è stato **rimosso** e sostituito integralmente dalla versione bundled da OCRE.

---

## 9. Struttura del workspace firmware

```
retrospect/ocre-workspace/          ← manifest root (west init da ocre-runtime)
├── ocre-runtime/                   ← OCRE Runtime ufficiale (main @ 8afde85)
│   ├── wasm-micro-runtime/         ← WAMR bundled (WAMR-2.4.1-240-gd1a577ea)
│   ├── include/ocre/               ← API pubblica OCRE (ocre_container_runtime_*)
│   └── samples/container_runtime/ ← campioni ufficiali (hello_world.wasm usato nei test)
├── zephyr/                         ← Zephyr RTOS v4.4.0
├── modules/                        ← moduli Zephyr standard
└── build/native_sim_64/zephyr/
    └── zephyr.exe                  ← firmware compilato (binario ELF Linux x86-64)

retrospect/zephyr-app/              ← app Wasmbed (firmware sorgente)
├── CMakeLists.txt                  ← integrazione OCRE come Zephyr extra module
├── prj.conf                        ← Kconfig: CONFIG_OCRE=y, TLS, CBOR, POSIX
├── src/
│   ├── main.c                      ← boot, init rete, avvio thread
│   ├── ocre_integration.c/h        ← wrapper OCRE container lifecycle
│   ├── wasmbed_protocol.c          ← gestione messaggi CBOR (Deploy → OCRE)
│   └── network_handler.c           ← TLS client, enrollment, heartbeat
└── boards/
    └── native_sim.conf             ← Kconfig specifici native_sim (chiave statica)
```

---

## 10. Come riprodurre i test

### Prerequisiti
- Host Linux con K3s, Docker, Python 3
- Zephyr SDK 1.0.1 installato in `retrospect/zephyr-sdk-1.0.1/`
- `west` 1.5.0 (`pip install west`)
- Workspace OCRE già inizializzato in `retrospect/ocre-workspace/`

### Build firmware
```bash
cd retrospect/ocre-workspace
west build -b native_sim/native/64 \
  /home/ubuntu/Thesis/retrospect/zephyr-app \
  --pristine \
  --build-dir build/native_sim_64
```

### Deploy K3s
```bash
cd retrospect
./scripts/deploy-k3s.sh
```

### Avvio device e test E2E
```bash
# 1. Creare Device CRD con chiave statica native_sim
kubectl apply -n wasmbed -f - <<EOF
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: native-sim-1
  namespace: wasmbed
spec:
  publicKey: "q6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6s"
  runtimeTarget: OcreZephyr
EOF

# 2. Avviare il firmware
./ocre-workspace/build/native_sim_64/zephyr/zephyr.exe &

# 3. Verificare enrollment
kubectl get device native-sim-1 -n wasmbed -w

# 4. Caricare modulo WASM dalla Dashboard (http://localhost:3000)
#    oppure via curl:
curl -X POST http://localhost:3001/api/v1/applications \
  -F "name=hello-ocre" \
  -F "wasm=@ocre-workspace/ocre-runtime/samples/container_runtime/wasm_apps/hello_world.wasm" \
  -F "targetDevices=native-sim-1" \
  -F "targetRuntime=OcreZephyr"

# 5. Verificare esecuzione
kubectl get application hello-ocre -n wasmbed -w
```

### Health check
```bash
curl http://localhost:8080/health    # Gateway
curl http://localhost:3001/health    # API Server
```

---

## 11. Target di test: native_sim vs MCU fisico

### Perché native_sim

I test E2E descritti in questo documento sono stati eseguiti **esclusivamente su `native_sim/native/64`**. Questo target compila il firmware come **processo Linux nativo a 64 bit** (`zephyr.exe`) che gira direttamente sull'host senza Renode né hardware fisico. È il target di integrazione continua ufficiale di OCRE e consente di verificare l'intero stack software (TLS, CBOR, enrollment, OCRE container lifecycle, WAMR) senza vincoli di memoria MCU.

`native_sim` non è un emulatore MCU: non ha memoria limitata, non esegue codice ARM, non ha periferiche hardware. È l'equivalente funzionale di un ambiente MPU/Linux con l'astrazione Zephyr sopra.

### Differenze rispetto a un MCU reale

| Aspetto | `native_sim` (testato) | MCU (prossimo step) |
|---|---|---|
| CPU | x86-64, memoria virtuale illimitata | Cortex-M33, 786 KB SRAM |
| MMU | presente (Linux) | assente |
| Renode | non necessario | richiesto per emulazione |
| WAMR execution mode | AOT/JIT disponibili | solo interpreter o fast-interp |
| Stack TLS + OCRE in SRAM | nessun vincolo | da verificare nel budget ~786 KB |
| Test E2E RETROSPECT | **completato** | **non ancora eseguito** |

### Prossimo step: board `b_u585i_iot02a`

La board target pianificata per la validazione su MCU è la **STMicroelectronics B-U585I-IOT02A** (STM32U5, ARM Cortex-M33, 786 KB SRAM, 2 MB Flash). È una delle board supportate ufficialmente da OCRE upstream ed è stata scelta perché:

- supportata nativamente da `ocre-runtime` (nessun porting board necessario);
- 786 KB SRAM sufficienti per TLS 1.3 + OCRE + WAMR interpreter + stack Zephyr;
- presente nel database board di Zephyr 4.4.0 (`zephyr/boards/st/b_u585i_iot02a/`).

Per completare questo step servirà:
1. `west build -b b_u585i_iot02a /home/ubuntu/Thesis/retrospect/zephyr-app`
2. File overlay `boards/b_u585i_iot02a.overlay` con partizione LittleFS su flash interna (richiesta da OCRE per storage container)
3. Script Renode `.resc` per la board (o hardware fisico)
4. Verifica budget SRAM (OCRE + WAMR interpreter + TLS mbedTLS + stack Zephyr)
5. Aggiornamento `wasmbed-qemu-manager` con `McuType::BU585iIot02a` e path firmware

---

## 12. Bug risolti durante la migrazione

1. **Field naming CRD**: `device_client.rs` inviava `connectedSince`/`lastHeartbeat` (camelCase) invece di `connected_since`/`last_heartbeat` (snake_case atteso dallo schema CRD). K8s scartava i campi silenziosamente → stato mai aggiornato.

2. **Errore Renode su native_sim**: il Device Controller tentava di avviare un container Renode per ogni device; su `native_sim` l'operazione falliva e propagava l'errore (`?`) interrompendo il reconcile loop. Corretto con warn + continua.

3. **Gateway endpoint vuoto**: `GatewayReference::new()` lascia `endpoint: ""`. Il Device Controller e l'Application Controller usavano questo campo come URL diretto → errore "relative URL without base". Corretto con fallback a discovery gateway via K8s API.

4. **imagePullPolicy: IfNotPresent**: K3s usava immagini Docker precedenti dalla cache locale anche dopo `docker push` + `kubectl rollout restart`. Cambiato in `Always` sui deployment dei controller.

5. **Double scheme URL**: l'Application Controller aggiungeva `http://` a un endpoint che lo conteneva già → `"http://http://..."`. Aggiunto controllo `starts_with("http://")` prima di costruire l'URL.
