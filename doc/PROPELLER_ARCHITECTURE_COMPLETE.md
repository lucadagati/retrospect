# RETROSPECT — Architettura Propeller: documentazione completa

**Data**: maggio 2026  
**Autore**: Antonio Ciliberto  
**Stato**: Fase A completata e verificata (E2E `add(10,20)=30` verde, 17 maggio 2026)

---

## Indice

1. [Panoramica](#1-panoramica)
2. [Architettura precedente: stack OCRE](#2-architettura-precedente-stack-ocre)
3. [Architettura attuale: stack Propeller](#3-architettura-attuale-stack-propeller)
4. [OCRE vs Propeller — confronto diretto](#4-ocre-vs-propeller--confronto-diretto)
5. [Cosa è stato rimosso e perché](#5-cosa-è-stato-rimosso-e-perché)
6. [Bug risolti durante l'integrazione](#6-bug-risolti-durante-lintegrazione)
7. [Guida operativa: avviare lo stack](#7-guida-operativa-avviare-lo-stack)
8. [Test E2E passo per passo](#8-test-e2e-passo-per-passo)
9. [Struttura file rilevanti](#9-struttura-file-rilevanti)
10. [Roadmap e stato corrente](#10-roadmap-e-stato-corrente)

---

## 1. Panoramica

**RETROSPECT** (*secuRE inTegration middlewaRe fOr cpS comPutE ConTinuum*) è una
piattaforma per il deployment e la gestione di applicazioni **WebAssembly** su
dispositivi embedded attraverso il continuum Cloud–Fog–Edge.

Il percorso evolutivo del sistema si divide in tre fasi architetturali:

```
Fase 1 — Stack RETROSPECT originale
  Kubernetes + Gateway TLS+CBOR (Rust) + Zephyr RTOS + WAMR custom

Fase 2 — Migrazione a OCRE (maggio 2026, prima metà)
  K8s + Gateway TLS+CBOR + Zephyr 4.4.0 + OCRE container runtime → WAMR

Fase 3 — Integrazione Propeller (maggio 2026, seconda metà) ← CORRENTE
  Propeller Manager + Magistrala MQTT + embed-proplet Zephyr + WAMR (NSOS)
```

Questo documento descrive la **Fase 3** in dettaglio, confrontandola con la
Fase 2 (OCRE) e spiegando perché Propeller la sostituisce.

---

## 2. Architettura precedente: stack OCRE

### 2.1 Diagramma

```
┌─────────────────────────────────────────────────────────────────┐
│  CLOUD — K3s cluster (namespace: wasmbed)                       │
│                                                                  │
│  Dashboard (React :3000)  ──→  API Server (Rust :3001)          │
│                                      │                           │
│                    Device CRD / Application CRD / Gateway CRD   │
│                                      │                           │
│   Device Controller │ App Controller │ Gateway Controller        │
│                         (kube-rs, Rust)                          │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  wasmbed-gateway (Rust)                                  │   │
│  │  :8080 HTTP northbound  │  :8081 TLS 1.3 southbound      │   │
│  │  Enrollment Ed25519 + CBOR (minicbor)                    │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────┬────────────────────────────────┘
                                 │ TLS 1.3 + Envelope<CBOR>
                          Ed25519 client cert
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│  EDGE — native_sim/native/64 (processo Linux x86-64)            │
│                                                                  │
│  Zephyr 4.4.0                                                    │
│    └─ OCRE Runtime (github.com/project-ocre/ocre-runtime)       │
│         └─ WAMR 2.4.x (bundled da OCRE)                         │
│              └─ App WASM (.wasm module)                          │
│                                                                  │
│  build: ocre-workspace/build/native_sim_64/zephyr/zephyr.exe    │
│  conn:  0x20001000 = endpoint gateway in SRAM (inject Renode)   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Stack tecnologico OCRE

| Layer | Componente | Versione |
|---|---|---|
| RTOS | Zephyr | 4.4.0 |
| SDK | Zephyr SDK | 1.0.1 |
| Container runtime | OCRE | main@8afde85 |
| WASM runtime | WAMR (bundled OCRE) | 2.4.1 |
| Build tool | west | 1.5.0 |
| Control plane | Rust workspace (19 crates) | 1.88.0 |
| Protocollo edge | TLS 1.3 + CBOR (minicbor) | — |
| Autenticazione | Ed25519 client certificate | — |
| Emulatore | Renode (opzionale, STM32/ARM) | — |

### 2.3 Cosa offriva OCRE

**OCRE** (Open Container Runtime for Embedded, progetto LF Edge / atym-io) è un
layer che si posiziona *sopra* WAMR e aggiunge:

- **API lifecycle container**: `ocre_container_runtime_create / run / stop / destroy`
  — il modulo WASM diventa un "container" con manifesto, limiti di risorse e
  lifecycle gestito.
- **Host functions hardware-independent**: GPIO, sensori, timer, messaging, RNG
  mappate su Zephyr driver API. Un'app WASM importa queste funzioni senza sapere
  nulla della board sottostante.
- **Modello di deployment a container**: manifesto + bytecode + limiti RAM/stack,
  validati prima dell'esecuzione.
- **Zero patch su upstream**: WAMR e Zephyr 4.4.0 sono usati senza modifiche.

**Limitazioni di OCRE nel contesto RETROSPECT**:

- OCRE non definisce il protocollo di comunicazione Cloud↔Device: il transport
  TLS+CBOR era tutto a carico di `wasmbed-gateway` e `wasmbed-protocol`, sviluppati
  da zero per questa tesi.
- Il Gateway Rust deve gestire enrollment, enrollment challenge Ed25519, deploy CBOR,
  heartbeat, status CRD — un control-plane completo e complesso da mantenere.
- Non esiste un orchestratore pronto che parli OCRE: Propeller Manager (Go) è
  progettato per MQTT, non per CBOR su TLS.
- OCRE è ancora in fase di sviluppo attivo (API soggette a breaking change).

---

## 3. Architettura attuale: stack Propeller

### 3.1 Diagramma

```
┌─────────────────────────────────────────────────────────────────────┐
│  CLOUD — host Linux (Docker)                                        │
│                                                                     │
│  ┌──────────────────────────┐    ┌──────────────────────────────┐  │
│  │ Propeller Manager        │    │ Magistrala (stack Docker)    │  │
│  │ (Go, :7070)              │    │  FluxMQ broker  (:1883)      │  │
│  │  REST: /tasks /proplets  │◄──►│  Users service  (:5002)      │  │
│  │  Scheduler               │    │  Clients svc    (:9006)      │  │
│  │  Task registry           │    │  Channels svc   (:9005)      │  │
│  └────────────┬─────────────┘    │  SpiceDB (authz)             │  │
│               │                  │  OpenBao (secrets)           │  │
│               │ MQTT pub/sub     │  Postgres + Redis            │  │
│               │ topic: m/<domain>│  nginx ingress               │  │
│               │ /c/<channel>/... └──────────────────────────────┘  │
│  ┌────────────▼─────────────────────────────────────────────────┐  │
│  │ Propeller Proxy (Go) — OCI registry fetcher                  │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────┬───────────────────────────────┘
                                      │ MQTT / TCP :1883
                              (host loopback — NSOS)
                                      │
┌─────────────────────────────────────▼───────────────────────────────┐
│  EDGE — native_sim/native/64 (processo Linux x86-64, zephyr.exe)   │
│                                                                     │
│  Zephyr 4.4.0 + NSOS (Native Offloaded Sockets)                    │
│    └─ embed-proplet (C)                                             │
│         ├─ MQTT client Zephyr (sub/pub canale Magistrala)           │
│         ├─ cJSON parser (task JSON payload)                         │
│         ├─ WAMR 2.4.3 (interprete x86-64, heap 40 KB)              │
│         │    └─ invoca funzione esportata WASM con inputs           │
│         └─ Publish risultato su topic /results                      │
│                                                                     │
│  build: propeller/embed-proplet/build/zephyr/zephyr.exe            │
│  conn:  NSOS → TCP host loopback 127.0.0.1:1883                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Stack tecnologico Propeller

| Layer | Componente | Versione |
|---|---|---|
| RTOS | Zephyr | 4.4.0 |
| SDK | Zephyr SDK | 1.0.1 |
| Socket layer | NSOS (Native Offloaded Sockets) | built-in Zephyr |
| WASM runtime | WAMR | 2.4.3 |
| Build tool | cmake + ninja (senza west) | 3.28 / 1.11 |
| Control plane | Propeller Manager (Go) | 0.4.0 |
| Broker | Magistrala / FluxMQ | — |
| Protocollo edge | MQTT 3.1.1 (QoS 1) | — |
| Autenticazione | UUID client_id + client_key Magistrala | — |

### 3.3 Flusso di comunicazione

#### Enrollment (una-tantum, compile-time)

```
1. Operatore: POST /domains/{d}/clients → Magistrala
   → ottiene client_id + client_key (UUID)
2. Operatore: inietta in embed-proplet/src/credentials.h
3. Build: cmake + ninja → zephyr.exe
```

Non c'è enrollment dinamico. L'identità del device è hardcoded nel binario come
coppia UUID. Non serve CA, non servono certificati X.509 o chiavi Ed25519.

#### Boot e registrazione

```
zephyr.exe boot
  → NSOS: TCP connect 127.0.0.1:1883
  → MQTT CONNECT (client_id=UUID, password=UUID)
  → MQTT SUBSCRIBE m/<domain>/c/<channel>/control/manager/start
  → MQTT PUBLISH  m/<domain>/c/<channel>/control/proplet/create
    payload: {"description":"...", "wasm_runtime":"wamr", ...}
```

Il Manager vede il proplet apparire in `GET /proplets`.

#### Heartbeat

```
Ogni 10s:
  MQTT PUBLISH m/<domain>/c/<channel>/control/proplet/alive
    payload: {"status":"alive","proplet_id":"<uuid>","namespace":"embedded"}
```

Il Manager aggiorna `last_alive_at` e imposta `alive: true`.

#### Deploy WASM e risultato

```
Operatore:
  POST /tasks {"name":"add","inputs":["10","20"]}  → Manager
  PUT  /tasks/<id>/upload  (multipart .wasm)        → Manager
  POST /tasks/<id>/start                            → Manager

Manager:
  MQTT PUBLISH m/<domain>/c/<channel>/control/manager/start
    payload: {"id":"<task_id>","name":"add","file":"<base64_wasm>","inputs":["10","20"]}

embed-proplet riceve:
  1. base64_decode(file) → wasm_bytes
  2. strtoull(inputs[0]) = 10, strtoull(inputs[1]) = 20
  3. wasm_runtime_load(wasm_bytes)
  4. wasm_runtime_lookup_function(module_inst, "add")   ← func_name = task name
  5. wasm_runtime_call_wasm_a(exec_env, func, results, 2, args)
  6. results[0].of.i32 = 30

  MQTT PUBLISH m/<domain>/c/<channel>/control/proplet/results
    payload: {"task_id":"<id>","results":"30"}

Manager:
  state = 3 (completed), results = "30"
```

---

## 4. OCRE vs Propeller — confronto diretto

### 4.1 Tabella comparativa

| Dimensione | Stack OCRE | Stack Propeller | Vincitore |
|---|---|---|---|
| **Protocollo Cloud↔Edge** | TLS 1.3 + Envelope CBOR (custom) | MQTT 3.1.1 via Magistrala (standard) | Propeller |
| **Autenticazione device** | Ed25519 client cert + enrollment TLS | UUID client_id/key Magistrala | Propeller (più semplice) |
| **Control plane** | 13 crates Rust da zero | Propeller Manager (Go, open-source) | Propeller |
| **Orchestratore pronto** | No (solo Gateway custom) | Sì (Manager REST + scheduler) | Propeller |
| **API hardware device** | OCRE host functions (GPIO, sensori) | Solo WASM puro (no host functions) | OCRE |
| **Container lifecycle** | Sì (`create/run/stop/destroy`) | No (task one-shot) | OCRE |
| **Runtime WASM device** | WAMR (via OCRE) | WAMR (diretto) | Pari |
| **Interoperabilità standard** | OCRE API (LF Edge) | MQTT + OCI registry | Propeller |
| **Scalabilità multi-device** | No (custom) | Sì (Magistrala multi-tenant) | Propeller |
| **Complessità di bootstrap** | Alta (CA, certificati, enrollment) | Bassa (UUID da API REST) | Propeller |
| **Dipendenze esterne** | K3s (obbligatorio) | Docker compose | Propeller |
| **Moduli WASM con I/O HW** | Sì (GPIO, sensori via OCRE) | No (logica pura) | OCRE |
| **Cold start WASM dichiarato** | — | < 10 ms (obiettivo Propeller) | Propeller |
| **Maturità progetto** | Alpha (breaking changes attesi) | Beta/stable (v0.4.0) | Propeller |

### 4.2 Perché OCRE è interessante per questa tesi

OCRE risolve il problema dell'**hardware abstraction layer** lato firmware: con OCRE,
un modulo WASM può accedere a GPIO, sensori, timer in modo portabile, senza sapere
nulla della board. Questo è esattamente il "write once, run anywhere" che la tesi vuole
dimostrare, esteso a risorse hardware reali.

OCRE modella anche il device come **container runtime** con manifesto e limiti di risorse
— un'astrazione più ricca del semplice "esegui questa funzione".

### 4.3 Perché Propeller sostituisce OCRE nel sistema attuale

**1. OCRE non porta un control-plane.**
OCRE è solo il runtime lato device. Il control-plane (come si mandano task a OCRE, come
si enrollano i device, come si gestisce il lifecycle remoto) era tutto implementato
manualmente in `wasmbed-gateway`, `wasmbed-protocol`, i controller Rust e le CRD
Kubernetes. Questa è la parte più costosa e più fragile del sistema.

**Propeller porta il control-plane già fatto**: Manager REST, scheduler, registry
task, heartbeat monitoring — tutto pronto. Il prezzo è rinunciare all'API hardware
OCRE, ma per i moduli WASM di questa tesi (logica di calcolo puro, non I/O hardware)
questo non è un vincolo.

**2. Protocollo MQTT vs TLS+CBOR.**
MQTT è uno standard IoT ampiamente supportato. Magistrala gestisce autenticazione,
canali, multi-tenancy. Il protocollo TLS+CBOR di RETROSPECT era custom, testato solo
in questo repo, difficile da debuggare.

**3. Enrollment senza PKI.**
Con OCRE, l'enrollment richiedeva: generare coppia Ed25519, registrare pubkey nel
Device CRD, il device si connette con TLS + client cert, il Gateway verifica e assegna
identità. Con Propeller: si chiama un'API REST di Magistrala, si ottiene un UUID, si
compila nel firmware. Meno codice, meno surface di attacco, più veloce da testare.

**4. OCRE è ancora alpha.**
Il progetto OCRE (project-ocre/ocre-runtime) ha breaking changes frequenti nelle API.
Propeller v0.4.0 è stabile e attivamente mantenuto da Abstract Machines.

**5. Propeller ha un target di tesi più ricco.**
Propeller è progettato esplicitamente per il continuum Cloud-Edge con WASM. Supporta
OCI registry, multi-proplet, scheduling. È più rilevante come contributo originale.

### 4.4 Cosa si perde passando da OCRE a Propeller

- **Host functions hardware**: con Propeller i moduli WASM non possono accedere a GPIO
  o sensori (non ci sono host functions registrate). Questo significa che solo moduli
  WASM di logica pura (matematica, ML inference) sono eseguibili senza modifiche.
- **Container lifecycle**: OCRE definisce `create/run/stop/destroy`. Con Propeller il
  task è one-shot (start → result). Per task long-running o daemon serve estensione.
- **Manifest con resource limits**: OCRE valida limiti RAM/stack prima dell'esecuzione.
  Con Propeller i limiti sono hardcoded nel firmware.

Per il target di tesi (deploy di moduli WASM di calcolo su MCU embedded via MQTT) questi
trade-off sono accettabili.

---

## 5. Cosa è stato rimosso e perché

### 5.1 Crates Rust rimossi (da `retrospect/crates/`)

Questi 13 crates implementavano il control-plane CBOR+TLS, interamente sostituiti da
Propeller Manager + Magistrala:

| Crate | Funzione originale | Sostituito da |
|---|---|---|
| `wasmbed-gateway` | Hub TLS 1.3 + CBOR (port 8080/8081) | Magistrala FluxMQ broker |
| `wasmbed-protocol` | Serializzazione `Envelope<T>` CBOR no_std | Protocollo MQTT Propeller (JSON) |
| `wasmbed-protocol-tool` | CLI per debug CBOR | — |
| `wasmbed-gateway-controller` | Controller K8s Gateway CRD | Propeller Manager |
| `wasmbed-application-controller` | Controller K8s Application CRD | Propeller Manager scheduler |
| `wasmbed-device-controller` | Controller K8s Device CRD | Magistrala Clients API |
| `wasmbed-tls-utils` | Server TLS (rustls, callbacks) | Magistrala TLS broker |
| `wasmbed-tcp-bridge` | Bridge TCP | — |
| `wasmbed-cert` | Gestione certificati Ed25519 | UUID Magistrala (no PKI) |
| `wasmbed-cert-tool` | CLI generazione certificati | — |
| `wasmbed-infrastructure` | CA + secret store | OpenBao (in Magistrala stack) |
| `wasmbed-qemu-manager` | Lifecycle container Renode | Avvio diretto `zephyr.exe` |
| `wasmbed-wasm-runtime` | Runtime WASM custom | WAMR in embed-proplet |
| `wasmbed-edge-client` | Client edge (Linux userspace) | embed-proplet (Zephyr/C) |

### 5.2 File/directory rimossi a livello repo

| Path | Motivo rimozione |
|---|---|
| `retrospect/ocre-workspace/` | OCRE sostituito da Propeller; embed-proplet ha già WAMR integrato ufficialmente |
| `Dockerfile.{gateway,gateway-controller,device-controller,application-controller}` | Immagini Docker dei componenti rimossi |
| `scripts/{generate-gateway-certs.sh,verify-tls-and-deploy.sh,test_enrollment.py}` | Script per stack TLS+CBOR non più esistente |
| `k8s/crds/{application,device,gateway}*.yaml` | CRD Kubernetes non più usate |
| `k8s/deployments/gateway*.yaml`, `*-controller*.yaml` | Deployment dei componenti rimossi |

### 5.3 Cosa è stato conservato

| Path | Motivo conservazione |
|---|---|
| `retrospect/zephyr-app/` | `boards/*.conf` e `*.overlay` riusabili per portare embed-proplet su MCU fisici (b_u585i_iot02a, STM32, ESP32) |
| `retrospect/dashboard-react/` | UI esistente — candidata a riscrittura come adapter verso Propeller Manager REST |
| `retrospect/doc/` | Archivio storico del lavoro tesi pre-Propeller (OCRE, Renode, TLS) |
| `retrospect/scripts/setup-renode-net.sh` | Script networking TAP/Renode — utile se si porta embed-proplet su emulatore MCU |
| `retrospect/Cargo.toml` | Workspace radice — crates rimasti (dashboard, types, k8s-resource) |

---

## 6. Bug risolti durante l'integrazione

Questi bug erano latenti nel codice `embed-proplet` e sono stati scoperti e corretti
durante la messa in esercizio su `native_sim`.

### Bug 1 — Board conf non caricato → MQTT -22 (EINVAL)

**Sintomo**: `mqtt_connect()` ritornava `-22` (EINVAL). I log mostravano
`eth_native_tap: Cannot create zeth` e inizializzazione di NVS, entrambi disabilitati
nel board conf.

**Causa**: Il file si chiamava `boards/native_sim.conf`. Zephyr 4.4.0 costruisce il
nome del board conf concatenando le parti del qualificatore con underscore
(`extensions.cmake:1717`): `native_sim/native/64` → `native_sim_native_64`. Il file
`native_sim.conf` non veniva caricato e i KConfig rimanevano ai default (con ETH TAP,
NVS, FLASH abilitati), rendendo il MQTT socket invalido.

**Fix**: Rinominato `boards/native_sim.conf` → `boards/native_sim_native_64.conf`.

**File modificato**: `propeller/embed-proplet/boards/native_sim_native_64.conf`

---

### Bug 2 — Loop iniziale bloccante: alive non inviate per ~10 minuti

**Sintomo**: Il proplet si connetteva e si sottoscriveva correttamente, ma non
compariva come `alive: true` nel Manager. I log mostravano solo ping MQTT ogni 30s,
nessun messaggio alive.

**Causa**: La funzione `mqtt_client_process()` chiamava `poll_mqtt_socket` con timeout
= `mqtt_keepalive_time_left()` (30000 ms con `CONFIG_MQTT_KEEPALIVE=30`). Il loop
iniziale di 20 iterazioni (`for(i=0; i<20; i++)`) bloccava quindi ~30s per iterazione
dopo le prime 2-3 (quando non arrivavano più dati). L'intera fase di avvio richiedeva
~10 minuti prima che la variabile `next_alive` venisse impostata e le alive iniziassero.

**Fix**: Cap del timeout di poll a 1000ms. Il loop principale è ora reattivo.

```c
/* mqtt_client.c — mqtt_client_process() */
int32_t keepalive_ms = mqtt_keepalive_time_left(&client_ctx);
int32_t timeout_ms = (keepalive_ms > 1000) ? 1000 : keepalive_ms;
int ret = poll_mqtt_socket(&client_ctx, timeout_ms);
```

**File modificato**: `propeller/embed-proplet/src/mqtt_client.c`

---

### Bug 3 — Funzione WASM "main" hardcoded invece del nome del task

**Sintomo**: Warning `Function 'main' not found in WASM module. No entry point to call.`
Il modulo `addition-wat.wasm` esporta la funzione `add`, non `main`.

**Causa**: `wasm_handler.c` cercava sempre `"main"` hardcoded:
```c
wasm_function_inst_t func = wasm_runtime_lookup_function(module_inst, "main");
```
Il Propeller Manager usa il **nome del task** come nome della funzione WASM da invocare
(`{"name":"add","inputs":["10","20"]}`). Questo è il contratto della piattaforma.

**Fix**: `execute_wasm_module` ora accetta un parametro `func_name` e lo usa nel lookup.
I caller passano `t.name` (il nome del task corrente).

```c
/* wasm_handler.h */
void execute_wasm_module(const char *task_id, const char *func_name,
                         const uint8_t *wasm_data, size_t wasm_size,
                         const uint64_t *inputs, size_t inputs_count);

/* mqtt_client.c */
execute_wasm_module(t.id, t.name, wasm_binary, wasm_decoded_len, t.inputs, t.inputs_count);
```

**File modificati**: `propeller/embed-proplet/src/wasm_handler.{h,c}`,
`propeller/embed-proplet/src/mqtt_client.c`

---

### Bug 4 — Input JSON come stringhe non parsati (risultato sempre 0)

**Sintomo**: Il task veniva eseguito senza errori ma il risultato era `"0"` invece
di `"30"` (`add(0,0)=0` invece di `add(10,20)=30`).

**Causa**: Il Propeller Manager invia gli input come **array di stringhe JSON**:
`"inputs":["10","20"]`. Il parser nel firmware usava solo `cJSON_IsNumber()`:

```c
if (cJSON_IsNumber(input)) {
    t.inputs[i] = (uint64_t)input->valuedouble;
}
/* Se input è string JSON → cJSON_IsNumber = false → t.inputs[i] rimane 0 */
```

**Fix**: Aggiunto parsing per stringhe con `strtoull`:

```c
if (cJSON_IsNumber(input)) {
    t.inputs[i] = (uint64_t)input->valuedouble;
} else if (cJSON_IsString(input) && input->valuestring != NULL) {
    t.inputs[i] = strtoull(input->valuestring, NULL, 10);
}
```

**File modificato**: `propeller/embed-proplet/src/mqtt_client.c`

---

### Bug 5 — Kconfig warnings bloccanti nel board conf

**Sintomo**: `cmake` abortiva con `Aborting due to Kconfig warnings` anche dopo la
rinomina del conf.

**Causa**: Alcuni simboli del board conf avevano dipendenze non soddisfatte su
`native_sim`:

| Simbolo | Problema |
|---|---|
| `CONFIG_COMMON_LIBC_MALLOC_ARENA_SIZE` | Dipende da `COMMON_LIBC_MALLOC` (non disponibile su native_sim) |
| `CONFIG_UART_NATIVE_PTY=y` | Dipende da `CONFIG_SERIAL=y` (che il conf disabilitava) |
| `CONFIG_UART_NATIVE_PTY_0_ON_STDINOUT=y` | Idem |
| `CONFIG_SETTINGS_NVS=n` | Choice symbol che scompare quando `CONFIG_SETTINGS=n` |
| `CONFIG_WIFI_NM_MAX_MANAGED_INTERFACES=0` | Dipende da `CONFIG_WIFI_NM=y` (disabilitato) |
| `CONFIG_WAMR_GLOBAL_HEAP_SIZE=524288` | Simbolo non definito in Kconfig Zephyr |
| `CONFIG_MQTT_LOG_LEVEL_DBG=y` | Choice symbol senza parent abilitato |

**Fix**: Rimossi tutti i simboli con dipendenze non soddisfatte dal board conf. I
simboli funzionalmente necessari (NSOS, memoria, WiFi=n, NVS=n) sono stati mantenuti.

**File modificato**: `propeller/embed-proplet/boards/native_sim_native_64.conf`

---

## 7. Guida operativa: avviare lo stack

### 7.1 Prerequisiti di sistema

```bash
# Verifica tool necessari
go version         # >= 1.26.0 (in /usr/local/go/bin)
docker --version   # >= 20.10
cmake --version    # >= 3.28
ninja --version    # qualsiasi
gcc --version      # toolchain host per native_sim

# SDK Zephyr
ls /home/ubuntu/Thesis/retrospect/zephyr-sdk-1.0.1/   # deve esistere

# ZEPHYR_BASE (propeller-ws è un bind mount di ocre-workspace/zephyr)
mountpoint /home/ubuntu/Thesis/retrospect/propeller-ws   # deve essere mountpoint
# Se non è montato:
sudo mount /home/ubuntu/Thesis/retrospect/propeller-ws
```

### 7.2 Avvio stack Propeller (Manager + Magistrala + Proxy)

```bash
# Ferma mosquitto se occupa la porta 1883
sudo snap stop mosquitto 2>/dev/null || true

# Avvia lo stack completo
cd /home/ubuntu/propeller
./start-propeller-stack.sh

# Verifica Manager
curl http://localhost:7070/health
# → {"status":"pass","version":"v0.4.0"}

# Verifica proplet Docker (opzionale — gira in background nello stack)
curl -s http://localhost:7070/proplets | jq '.proplets[] | {id,alive,metadata.wasm_runtime}'
```

### 7.3 Build embed-proplet per native_sim

```bash
cd /home/ubuntu/Thesis/retrospect/propeller/embed-proplet

# Carica variabili ambiente Zephyr
source /home/ubuntu/Thesis/retrospect/.env.zephyr
# ZEPHYR_BASE=/home/ubuntu/Thesis/retrospect/propeller-ws

# Build (cmake diretto — non usare west, manca il workspace context)
cmake -B build -GNinja \
  -DBOARD=native_sim/native/64 \
  -DZEPHYR_BASE=$ZEPHYR_BASE \
  -DCMAKE_PREFIX_PATH=$ZEPHYR_BASE/share/zephyr-package/cmake \
  .
ninja -C build

# Verifica: il binario deve esistere
ls -la build/zephyr/zephyr.exe
```

**Nota**: `west build` non funziona senza `.west/` sopra la directory del progetto.
Si usa `cmake` direttamente — la variabile `ZEPHYR_MODULES` nel `CMakeLists.txt`
serve proprio per aggirare questa limitazione.

### 7.4 Avvio embed-proplet

```bash
cd /home/ubuntu/Thesis/retrospect/propeller/embed-proplet

# Verifica che il broker MQTT sia raggiungibile
nc -z -w2 127.0.0.1 1883 && echo "broker OK" || echo "broker DOWN"

# Avvia in foreground (Ctrl-C per fermare)
./build/zephyr/zephyr.exe

# Oppure in background con log su file
./build/zephyr/zephyr.exe > /tmp/proplet.log 2>&1 &
tail -f /tmp/proplet.log
```

**Output atteso a boot**:
```
*** Booting Zephyr OS build v4.4.0 ***
[inf] main: Starting Proplet...
[inf] mqtt_client: MQTT connection accepted by broker
[inf] mqtt_client: MQTT client connected successfully
[inf] mqtt_client: Successfully subscribed to topics for channel ID: ae9fb9bf-...
[inf] mqtt_client: Discovery published successfully to topic: m/.../control/proplet/create
```

**Dopo ~30s** (primo ciclo keepalive):
```
[inf] mqtt_client: Published to topic: .../control/proplet/alive
     Payload: {"status":"alive","proplet_id":"4f2d721a-...","namespace":"embedded"}
```

### 7.5 Fermare lo stack

```bash
# Ferma embed-proplet
kill $(pgrep -f "zephyr.exe")

# Ferma stack Propeller (Manager, Proxy, Proplet Docker)
cd /home/ubuntu/propeller && make stop-propeller

# Ferma Magistrala
cd /home/ubuntu/propeller && make stop-magistrala
```

---

## 8. Test E2E passo per passo

Questo è il test di riferimento per verificare che tutto funzioni end-to-end.
Risultato atteso: `state=3, results="30"`.

### Prerequisiti

1. Stack Propeller up (`./start-propeller-stack.sh`)
2. embed-proplet avviato (`./build/zephyr/zephyr.exe &`)
3. embed-proplet visibile come `alive: true` nel Manager

```bash
# Verifica che embed-proplet sia alive (aspetta ~35s dopo il boot)
curl -s http://localhost:7070/proplets | jq \
  '.proplets[] | select(.metadata.wasm_runtime=="wamr") | {id,alive,last_alive_at}'
# Atteso: alive: true
```

### Esecuzione test

```bash
cd /home/ubuntu/Thesis/retrospect/propeller

# Step 1: crea task
TASK_ID=$(curl -s -X POST http://localhost:7070/tasks \
  -H "Content-Type: application/json" \
  -d '{"name":"add","inputs":["10","20"]}' | jq -r .id)
echo "Task ID: $TASK_ID"

# Step 2: carica il modulo WASM (addition-wat.wasm = 41 byte, esporta add(i32,i32)->i32)
curl -s -X PUT "http://localhost:7070/tasks/${TASK_ID}/upload" \
  -F "file=@build/addition-wat.wasm"

# Step 3: avvia il task
curl -s -X POST "http://localhost:7070/tasks/${TASK_ID}/start"

# Step 4: aspetta e verifica
sleep 5
curl -s "http://localhost:7070/tasks/${TASK_ID}" | jq '{state, results, error}'
```

**Risultato atteso**:
```json
{
  "state": 3,
  "results": "30\n",
  "error": null
}
```

Legenda stati: `0`=pending, `1`=assigned, `2`=running, `3`=completed, `4`=failed.

### Rebuild dopo modifiche al sorgente

```bash
cd /home/ubuntu/Thesis/retrospect/propeller/embed-proplet
kill $(pgrep -f "zephyr.exe") 2>/dev/null
ninja -C build                         # rebuild incrementale (solo file modificati)
./build/zephyr/zephyr.exe > /tmp/proplet.log 2>&1 &
```

**Non è necessario re-eseguire cmake** a meno che non si modifichino i CMakeLists.txt
o i file `.conf`/`.overlay`.

---

## 9. Struttura file rilevanti

```
Thesis/
├── retrospect/
│   ├── propeller/                          ← Workspace Propeller (git clone)
│   │   ├── embed-proplet/                  ← App Zephyr in C (TARGET DI TESI)
│   │   │   ├── boards/
│   │   │   │   ├── native_sim_native_64.conf  ← Board conf NSOS (CRITICO: nome esatto)
│   │   │   │   └── native_sim.overlay         ← Overlay DTS (vuoto)
│   │   │   ├── src/
│   │   │   │   ├── main.c                  ← Entry point, loop alive/metrics
│   │   │   │   ├── mqtt_client.c           ← MQTT connect/subscribe/publish
│   │   │   │   ├── mqtt_client.h
│   │   │   │   ├── wasm_handler.c          ← WAMR load/instantiate/call
│   │   │   │   ├── wasm_handler.h
│   │   │   │   ├── credentials.c           ← Load credenziali (compile-time o NVS)
│   │   │   │   ├── credentials.h           ← UUID Magistrala (EDITARE per nuove creds)
│   │   │   │   ├── task_monitor.c
│   │   │   │   ├── native_sim_stubs.c      ← Stub __stdout_hook_install (native_sim)
│   │   │   │   └── wifi_manager_stub.c     ← Stub WiFi (native_sim, no WiFi)
│   │   │   ├── CMakeLists.txt              ← Build config (ZEPHYR_MODULES stub trick)
│   │   │   ├── prj.conf                    ← KConfig base (MQTT, WAMR, heap...)
│   │   │   └── build/
│   │   │       └── zephyr/
│   │   │           └── zephyr.exe          ← Binario nativo native_sim
│   │   ├── build/
│   │   │   └── addition-wat.wasm           ← Modulo WASM test (add i32,i32 → i32)
│   │   ├── start-propeller-stack.sh        ← Script avvio completo stack
│   │   └── config.toml                     ← Credenziali Magistrala (domain/client/channel)
│   │
│   ├── propeller-ws/                       ← BIND MOUNT di ocre-workspace/zephyr
│   │   │                                      (Zephyr 4.4.0, WAMR, OCRE)
│   │   │                                      Persistente via /etc/fstab
│   │   └── share/zephyr-package/cmake/     ← ZephyrConfig.cmake (richiesto da cmake)
│   │
│   ├── zephyr-sdk-1.0.1/                   ← SDK Zephyr (host tools, dtc)
│   ├── .env.zephyr                         ← ZEPHYR_BASE + ZEPHYR_SDK_INSTALL_DIR
│   │
│   ├── crates/                             ← Workspace Cargo (crates rimanenti)
│   ├── dashboard-react/                    ← Dashboard React (non funzionante con nuovo backend)
│   ├── zephyr-app/                         ← Legacy Zephyr firmware (boards/*.conf riusabili)
│   └── doc/                                ← Questo documento e tutta la documentazione
│
└── MasterThesis/                           ← PoC accademico WASM Cloud+Edge (separato)
```

### Credenziali Magistrala (embed-proplet)

Le credenziali sono in `propeller/embed-proplet/src/credentials.h`:

```c
static const struct proplet_credentials defaults = {
    .wifi_ssid   = "<non usato su native_sim>",
    .wifi_psk    = "<non usato su native_sim>",
    .proplet_id  = "4f2d721a-c03e-4ed5-8664-aabe22583435",  // client_id Magistrala
    .client_key  = "08733ab3-1e28-4cd3-9922-eb25bbcbd382",  // client_key Magistrala
    .domain_id   = "69df50b5-2111-42ae-a9e1-fdd4d2b0b54e",  // domain Magistrala
    .channel_id  = "ae9fb9bf-3697-41f6-beb9-85d1d71954f5",  // channel Magistrala
};
```

Per usare credenziali diverse: modificare i valori, fare `ninja -C build`.

---

## 10. Roadmap e stato corrente

### Completato (Fase A — E2E native_sim)

- [x] Board conf `native_sim_native_64.conf` con NSOS
- [x] Build embed-proplet su native_sim (cmake + ninja, senza west)
- [x] Connessione MQTT al broker Magistrala (NSOS → TCP loopback)
- [x] Discovery e heartbeat alive (Manager vede proplet `alive: true`)
- [x] Deploy WASM via Manager REST → MQTT → WAMR → risultato
- [x] Test E2E `add(10,20)=30` verde

### Completato (Fase B — Cleanup)

- [x] Rimossi 14 crates Rust legacy (gateway, protocol, tls-utils, cert, infrastructure, qemu-manager, api-server, edge-client, …)
- [x] `wasmbed-qemu-manager` archiviato in `archive/wasmbed-qemu-manager/` (non eliminato, vedi §10.1)
- [x] Rimossi Dockerfile.gateway, .gateway-controller, .device-controller, .application-controller
- [x] Rimossi script legacy (generate-gateway-certs.sh, verify-tls-and-deploy.sh, deploy-k3s.sh, cleanup-k3s.sh, test_enrollment.py)
- [x] Rimossi CRD K8s (application, device, gateway) e manifest deployment legacy
- [x] Namespace K3s `wasmbed` eliminato (6 deployment rimossi)
- [x] Workspace Cargo ridotto a 5 crates: config, k8s-resource, k8s-resource-tool, test-utils, types

### Da fare (Fase C — Documentazione)

- [ ] Riscrittura `PROPELLER_INTEGRATION.md` con architettura definitiva
- [ ] Tabella completa rimosso→sostituito con riferimenti commit

### Aperto (Fase D — Emulazione MCU)

- [ ] Research: QEMU vs Renode per STM32U5 / Cortex-M33 (b_u585i_iot02a)
- [ ] Portare embed-proplet su board MCU reale o emulata (vedi §10.1 per guida)

### 10.1 — Perché `wasmbed-qemu-manager` è archiviato e non eliminato

`wasmbed-qemu-manager` (ora in `archive/wasmbed-qemu-manager/`) era il **Renode Orchestrator** dell'architettura OCRE. Faceva queste cose:

1. Avviava il container Docker `antmicro/renode:nightly` con monitor TCP
2. Inviava comandi Renode via TCP per caricare il firmware `.elf` sulla board emulata
3. Gestiva un `TcpBridge` (proxy TCP) tra Renode e il `wasmbed-gateway` (TLS+CBOR)
4. Registrava la board al gateway (`POST /api/v1/board/register`)
5. Traduceva indirizzi ClusterIP K8s → NodePort (perché Renode girava su host network)

**Perché non serve più con Propeller**: tutta questa logica era al servizio dell'architettura OCRE. Il gateway TLS+CBOR è eliminato; `embed-proplet` si connette direttamente al broker MQTT Magistrala. Non c'è più né gateway da registrare né TcpBridge da gestire.

**Perché è archiviato e non eliminato**: contiene informazioni preziose per **Fase D** (emulazione MCU reale):

- La lista `McuType` con i nomi delle piattaforme Renode (`stm32f7_discovery-bb`, `frdm_k64f`, `esp32`, `nrf52840dk_nrf52840`, ecc.)
- I template `.resc` (Renode Script) che configura per ogni MCU (caricamento ELF, configurazione rete TAP)
- La logica di attesa boot e connessione monitor TCP

#### Come dovrà funzionare Fase D (emulazione con Propeller)

Il `wasmbed-qemu-manager` **non è riutilizzabile direttamente** per Propeller perché è cablato sul vecchio gateway. Quello che serve per far girare `embed-proplet` su un MCU emulato con Propeller è un tool molto più semplice:

```
┌─────────────────────────────────────────────────────────┐
│  Nuovo launcher MCU (script o tool Go/Python)           │
│                                                          │
│  1. Compila embed-proplet per la board target            │
│     cmake -DBOARD=b_u585i_iot02a ...                    │
│                                                          │
│  2. Avvia Renode con il .resc della board               │
│     renode --plain script.resc                          │
│     (carica zephyr.elf, configura TAP networking)       │
│                                                          │
│  3. Configura TAP per connettere la rete emulata        │
│     al broker Magistrala sull'host                      │
│     (setup-renode-net.sh già presente in scripts/)      │
│                                                          │
│  4. Il firmware si connette a MQTT Magistrala           │
│     esattamente come native_sim, ma su MCU emulato      │
└─────────────────────────────────────────────────────────┘
```

**Differenza chiave rispetto all'architettura OCRE**: non serve nessun TcpBridge né registrazione al gateway. Basta che la rete TAP del device emulato raggiunga l'host sulla porta 1883 (MQTT).

Il file `scripts/setup-renode-net.sh` (già presente) configura il TAP e il routing NAT necessari.

Come riferimento per i template `.resc` MCU e i nomi di piattaforma Renode, consultare `archive/wasmbed-qemu-manager/src/lib.rs` (funzione `renode_platform()` e generatore `.resc`).

### Nota sulla Dashboard

La Dashboard React in `dashboard-react/` è cablata su `wasmbed-api-server` (rimosso).
Attualmente non è funzionante. Opzioni:
1. **Adapter REST**: proxy sottile `wasmbed-api-server` → Propeller Manager REST.
   Costo: ~giorni. Riusa la UI esistente.
2. **Nuova UI**: consuma direttamente Manager API. Costo: settimane.

Per i test della tesi la Dashboard non è bloccante (si usa `curl`).

---

## Appendice — Comandi rapidi

```bash
# Health check completo in un comando
echo "=== Manager ===" && curl -s http://localhost:7070/health && \
echo "" && echo "=== Proplets ===" && \
curl -s http://localhost:7070/proplets | jq '.proplets[] | {id,alive,metadata}'

# Test add in one-liner
TASK_ID=$(curl -s -X POST http://localhost:7070/tasks \
  -H "Content-Type: application/json" \
  -d '{"name":"add","inputs":["10","20"]}' | jq -r .id) && \
curl -s -X PUT "http://localhost:7070/tasks/${TASK_ID}/upload" \
  -F "file=@/home/ubuntu/Thesis/retrospect/propeller/build/addition-wat.wasm" && \
curl -s -X POST "http://localhost:7070/tasks/${TASK_ID}/start" && \
sleep 5 && \
curl -s "http://localhost:7070/tasks/${TASK_ID}" | jq '{state,results,error}'

# Logs proplet in tempo reale
tail -f /tmp/proplet.log | grep -v "Ping response"

# Verifica che board conf sia caricato nel build
grep -E '^(# )?CONFIG_(ETH_NATIVE_TAP|NVS|SETTINGS|NET_SOCKETS_OFFLOAD|NET_NATIVE_OFFLOADED_SOCKETS)[= ]' \
  /home/ubuntu/Thesis/retrospect/propeller/embed-proplet/build/zephyr/.config

# Verifica bind mount propeller-ws dopo reboot
mountpoint /home/ubuntu/Thesis/retrospect/propeller-ws || sudo mount -a
```
