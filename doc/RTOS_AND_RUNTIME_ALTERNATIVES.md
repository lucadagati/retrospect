# Alternative RTOS e Runtime per RETROSPECT — Analisi Comparativa

> **Scopo**: documento di analisi decisionale per la tesi. Confronta
> **Phoenix RTOS** (alternativa a Zephyr) e **Propeller** (alternativa al
> control-plane cloud) rispetto allo stack attuale di RETROSPECT, indicando
> cosa ciascuno sostituirebbe, a quale costo e con quale beneficio.
> Nessuna modifica al codice è proposta in questo documento.
>
> **Baseline tecnica**: le decisioni architetturali già prese (Opzione A,
> edge eterogeneo, Gateway invariato) sono documentate in
> [`OCRE_DEPLOYMENT_TOPOLOGY.md`](./OCRE_DEPLOYMENT_TOPOLOGY.md). Questo
> documento si innesta su quella baseline senza contraddirla.

---

## 1. Perimetro dell'analisi

RETROSPECT è una piattaforma Kubernetes-native per deployare applicazioni
WebAssembly su dispositivi embedded attraverso il continuum Cloud-Fog-Edge
(vedi [`ARCHITECTURE.md`](./ARCHITECTURE.md)). Lo stack attuale è:

```
CLOUD     Kubernetes: API Server (Rust) + Controllers + Dashboard (React)
             ↕  HTTP / K8s CRD
FOG       wasmbed-gateway: hub TLS 1.3 + CBOR (port 8080/8081)
             ↕  TLS + CBOR (wasmbed-protocol, no_std)
EDGE      Zephyr RTOS + WAMR + app WASM
          [STM32F746G, FRDM-K64F, ESP32, RISC-V virt, Cortex-R8 virt, …]
```

I punti dello stack potenzialmente sostituibili da tecnologie alternative:

| Layer | Componente | Sostituibile da |
|---|---|---|
| Edge firmware | `zephyr-app/` (Zephyr + WAMR) | Phoenix RTOS (Scenario B) |
| Edge MPU daemon | `wasmbed-edge-client` (Linux+wasmtime) | Phoenix RTOS (Scenario A) |
| Edge emulazione | `wasmbed-qemu-manager` + Renode | dipende dallo scenario |
| Cloud control-plane | `wasmbed-api-server` + controllers | Propeller |
| Cloud/Fog bridge | `wasmbed-gateway` (TLS+CBOR) | Propeller (parziale) |

**Non cambia** in nessuno degli scenari analizzati: il protocollo TLS 1.3
southbound, le CRD Kubernetes (`Device`, `Application`, `Gateway`), la
dashboard React, la gestione certificati Ed25519.

---

## 2. Phoenix RTOS vs Zephyr RTOS

### 2.1 Contesto

Phoenix RTOS (phoenix-rtos.com, licenza BSD-3) è un sistema operativo
real-time a **microkernel** pensato per dispositivi embedded con MMU, come
board ARMv7-A (i.MX 6ULL, Zynq-7000), RISC-V Linux-class e x86. Adotta un
modello di processi user-space isolati, un layer POSIX relativamente completo
(`libphoenix`: fork/exec, FS gerarchico, segnali) e una toolchain GCC propria
(`phoenix-rtos-project`).

Zephyr RTOS (zephyrproject.org, licenza Apache-2.0) è un **nanokernel
monolitico configurabile** che opera in singolo spazio di indirizzamento. Offre
il catalogo di board più ampio nel settore embedded open-source (> 500 board),
ha un port WAMR ufficiale e supporto OCRE upstream (LF Edge).

### 2.2 Confronto sintetico

| Dimensione | Zephyr RTOS | Phoenix RTOS |
|---|---|---|
| Modello kernel | Nanokernel monolitico | Microkernel, message passing |
| Isolamento memoria | MPU (parziale, `CONFIG_USERSPACE`) | MMU, processi user-space nativi |
| Catalogo board MCU | > 500 (STM32, nRF, ESP32, RISC-V, ARM Cortex-M/R/A, …) | i.MX 6ULL, Zynq-7000, alguns Cortex-M e x86 |
| Board usate in RETROSPECT | STM32F746G, FRDM-K64F, ESP32, RISC-V virt, Cortex-R8 virt | Nessuna — non presenti upstream |
| Port WAMR ufficiale | Sì (`core/shared/platform/zephyr/`) | No — richiederebbe porting manuale |
| OCRE upstream | Sì (`native_sim`, `b_u585i_iot02a`, `pico_plus2`) | No — nessun target Phoenix in OCRE |
| Integrazione Renode | Sì (`.repl` pronti per 13+ board) | No — nessun platform descriptor Phoenix |
| Networking socket | `zsock_*` + WAMR WASI-ready | `libphoenix` socket (POSIX), richiederebbe shim WAMR |
| POSIX | Subset (pthread, select, parziale FS) | Più completo (fork/exec, FS gerarchico, segnali) |
| Toolchain | `west` + Zephyr SDK | `phoenix-rtos-project`, toolchain GCC proprietaria |
| Licenza | Apache-2.0 | BSD-3 |
| Compatibilità AGPL-3.0 RETROSPECT | Sì | Sì |

### 2.3 Cosa Phoenix non supporta nativamente rispetto a Zephyr

Questi gap sono rilevanti per l'integrazione concreta con RETROSPECT:

**1. Port WAMR ufficiale.**
Zephyr offre `WAMR_BUILD_TARGET` con board file e CMake preconfigurati; su
Phoenix il porting sarebbe manuale. La modalità interprete WAMR è portabile,
ma fast-interp richiederebbe validazione e AOT (Ahead-Of-Time) dipende da un
ELF loader Phoenix che accetti il format WAMR.

**2. OCRE / LF Edge container runtime.**
I target ufficiali OCRE (`native_sim`, `b_u585i_iot02a`, `pico_plus2`) non
includono nessuna board Phoenix. Come documentato in
[`OCRE_INTEGRATION_ANALYSIS.md`](./OCRE_INTEGRATION_ANALYSIS.md), anche su
Zephyr il support è limitato alle board in lista; su Phoenix la distanza è
ancora maggiore.

**3. Board del catalogo RETROSPECT.**
Le 13+ board gestite da `wasmbed-qemu-manager`
(vedi [`MCU_SUPPORT.md`](./MCU_SUPPORT.md)) sono tutte Zephyr. Phoenix non ha
supporto ufficiale per STM32F746G Discovery, FRDM-K64F, ESP32 DevKitC, nRF52840,
RISC-V virt né Cortex-R8 virt. Phoenix è pensato per SoC con MMU, non per
microcontrollori bare-metal a cui Zephyr è ottimizzato.

**4. Emulazione Renode.**
Renode dispone di platform descriptor (`.repl`) per tutte le board Zephyr usate
in RETROSPECT. Phoenix non ha `.repl` equivalenti: andrebbe usato QEMU con
immagine Phoenix (supportata per i.MX 6ULL e ARM Versatile) oppure un
emulatore diverso, rompendo la filiera `wasmbed-qemu-manager → Renode → device`.

**5. WASI networking via `zsock_*`.**
WAMR su Zephyr usa internamente le zsock come shim POSIX; questo è ciò che
consente `WAMR_BUILD_LIBC_WASI=1` senza modifiche al kernel (fix applicato in
[`OCRE_EDGE_INTEGRATION.md`](./OCRE_EDGE_INTEGRATION.md) §2.5). Su Phoenix
serve un shim analogo verso `libphoenix` socket API: fattibile, ma non
disponibile upstream.

**6. Driver NIC per Renode TAP networking.**
Il setup TAP documentato in `CLAUDE.md` (§ "MCU Supportati e Networking")
usa driver `synopsys_emac` (STM32F746G) e `kinetis_eth` (FRDM-K64F), entrambi
Zephyr. Il supporto di quei NIC in Phoenix non esiste.

**7. Integrazione `McuType` in `wasmbed-qemu-manager`.**
L'enum `McuType` in `crates/wasmbed-qemu-manager/src/lib.rs` mappa ogni board
a un file `.repl` Renode e un percorso ELF Zephyr. Phoenix richiederebbe
varianti nuove (`PhoenixImx6`, ecc.) con un emulatore diverso — in pratica
un secondo backend di emulazione nel crate.

> **Nota onestà intellettuale.** Phoenix offre genuinamente più dei seguenti
> aspetti rispetto a Zephyr: isolamento processi con MMU (irrilevante per
> RETROSPECT perché la sandbox è già WASM), layer POSIX più ricco (utile solo
> se si abbandonasse la sandbox WASM), e una narrativa "Unix-like" più ordinata
> per device Linux-class. Nessuno di questi benefici è, però, necessario per
> l'obiettivo di RETROSPECT.

### 2.4 Scenari di integrazione con RETROSPECT

#### Scenario A — Phoenix come OS per device MPU (sostituto di Linux)

Phoenix prenderebbe il posto di Linux sui device `MpuRich` introdotti in
[`OCRE_EDGE_INTEGRATION.md`](./OCRE_EDGE_INTEGRATION.md) (§2.2 — `DeviceClass::MpuRich`,
`McuType::LinuxArm64/X86_64/RiscV`). Il crate `wasmbed-edge-client` (daemon
tokio+rustls+wasmtime) girerebbe su Phoenix invece di Linux.

Cosa serve:

- Porting di `tokio` su Phoenix: dipende da `epoll` o `io_uring`; Phoenix ha
  `select`/`poll` ma non epoll. `mio` (layer I/O di tokio) supporta `poll`
  come fallback, ma il porting non è documentato e richiede lavoro.
- `rustls` è Rust puro e non ha dipendenze OS — portatile senza modifiche.
- `wasmtime` dipende da primitive POSIX (`mmap`, `mprotect`, segnali) che
  Phoenix espone solo parzialmente; il JIT di wasmtime usa `mprotect` per
  rendere le pagine eseguibili — da verificare su Phoenix.
- Il meccanismo di avvio automatico (oggi systemd unit) andrebbe sostituito
  con l'init di Phoenix.
- Si introdurrebbe `DeviceClass::MpuPhoenix` e `targetRuntime: phoenix_wasmtime`
  nella CRD (estensione naturale — i campi sono `Option`).

Cosa resta invariato: `zephyr-app/`, Renode, tutti i device MCU, Gateway,
CRD esistenti.

**Costo**: medio. Il porting di tokio su Phoenix è il blocco principale.
**Beneficio**: isolamento processi MMU sul device edge — ridondante rispetto
alla sandbox WASM già presente.
**Verdetto**: tecnicamente realizzabile come ricerca avanzata; non giustificato
per la baseline della tesi.

#### Scenario B — Phoenix come RTOS (sostituto di Zephyr nel firmware MCU)

Phoenix prenderebbe il posto di `zephyr-app/` integralmente. Il firmware
girerebbe su Phoenix invece di Zephyr; la comunicazione TLS+CBOR verso il
Gateway resterebbe invariata (i protocolli sono OS-agnostici).

Cosa serve di fatto di riscrivere:

- **Porting WAMR su Phoenix** (non esiste upstream). L'interprete WAMR è C99
  portabile ma richiede un platform adapter (`platform/phoenix/`) con allocatore,
  mutex, thread e socket shim. Effort stimato: 4-8 settimane.
- **Riscrittura completa di `wamr_integration.c`** contro le API Phoenix.
- **TLS client**: mbedTLS è disponibile su Phoenix — questa parte è fattibile.
- **Framing CBOR**: `wasmbed-protocol` è `#![no_std]` e riusabile su qualsiasi
  target, quindi invariato.
- **Gateway endpoint injection**: oggi a `0x20001000` (indirizzo RAM fisso
  scritto da Renode prima del boot, come documentato in `CLAUDE.md` §
  "Flusso Enrollment"). Su Phoenix servirebbe un meccanismo equivalente:
  variabile d'ambiente nel boot, file in romfs, o parametro sul command line.
- **Emulazione**: i `.repl` Renode per Zephyr non si usano più. Si passerebbe
  a QEMU con immagine Phoenix (ARM Versatile, i.MX 6ULL) — ma allora le board
  MCU constrained (STM32, ESP32, nRF) non sono più emulabili nello stesso
  modo, perché Phoenix non le supporta.

Cosa resta invariato: Gateway, CRD, Dashboard, API Server, Application e
Device Controller.

**Costo**: molto alto. In pratica si riscrive l'intero edge stack senza
benefici architetturali, perché la sandbox WASM già garantisce l'isolamento
che Phoenix offrirebbe tramite MMU.
**Verdetto**: sconsigliato. Il rapporto costo/beneficio è fortemente negativo.

### 2.5 Tabella impatto per componente

| Componente RETROSPECT | Scenario A — Phoenix MPU | Scenario B — Phoenix MCU |
|---|---|---|
| `zephyr-app/` | Invariato | Sostituito da app Phoenix |
| `retrospect/wamr/` | Invariato (MPU usa wasmtime) | Da portare su Phoenix |
| `wasmbed-edge-client` | Adattato a Phoenix (porting tokio) | Non applicabile |
| `wasmbed-qemu-manager` `McuType` | Aggiunge `PhoenixMpu*` | Aggiunge varianti Phoenix MCU + nuovo emulatore |
| Renode `.repl` files | Invariati | Da sostituire (QEMU Phoenix) |
| Gateway TLS+CBOR | Invariato | Invariato |
| `wasmbed-protocol` | Invariato (`no_std`) | Invariato |
| CRD `Application`/`Device` | Aggiunge `runtimeTarget: phoenix_wasmtime` | Aggiunge varianti Phoenix |
| Dashboard React | Invariata | Invariata |

---

## 3. Propeller — Deploy Wasm from Cloud to Microcontrollers

### 3.1 Cos'è Propeller

Propeller è un progetto dell'ecosistema Atym/LF Edge (la stessa organizzazione
di OCRE) che fornisce un **control-plane cloud-native** per il deploy di moduli
WebAssembly su flotte di dispositivi embedded. Funziona come pipeline
dichiarativa: manifest → registry WASM → device agent. I componenti tipici:

- **Propeller Server**: control plane REST/gRPC, registry WASM, riconciliazione
  manifest verso device (logica simile a un controller Kubernetes).
- **Propeller Agent**: daemon lato device (o lato bridge), esegue i manifest
  interagendo con il runtime locale — tipicamente OCRE su Zephyr o WAMR puro.
- **Manifest dichiarativo**: struttura simile a un Pod Kubernetes con bytecode
  WASM, limiti di risorse, host capabilities richieste.

Propeller è pensato come strato intermedio tra un orchestratore cloud
(Kubernetes, ma non solo) e dispositivi con runtime WASM embedded, con lo
scopo di astrarre il trasporto e il ciclo di vita del deploy.

### 3.2 Sovrapposizione con RETROSPECT

Propeller copre esattamente il sottoinsieme funzionale che RETROSPECT ha
implementato in modo proprietario nel lato cloud/fog:

| Funzione | RETROSPECT | Propeller |
|---|---|---|
| Registry applicazioni WASM | `wasmbed-api-server` (upload + CRUD) | Propeller Server (registry integrato) |
| Reconcile manifest → device | `wasmbed-application-controller` | Propeller Server (reconciler nativo) |
| Bridge cloud ↔ device | `wasmbed-gateway` (TLS+CBOR) | Propeller Agent/bridge (MQTT o HTTPS) |
| Manifest dichiarativo | `Application` CRD Kubernetes | Manifest Propeller (simile a Pod) |
| Device registry / enrollment | `wasmbed-device-controller` + Gateway | Propeller Agent (enrollment integrato) |
| Pannello operatore | Dashboard React | UI Propeller (o integrazione K8s) |

Propeller non sostituisce un singolo crate ma **un sotto-stack completo** del
lato cloud/fog di RETROSPECT.

### 3.3 Modelli di integrazione

#### Modello A — Sostituzione completa del control-plane

Propeller Server sostituisce `wasmbed-api-server`, `wasmbed-application-controller`,
`wasmbed-device-controller`, `wasmbed-gateway-controller` e, parzialmente, il
Gateway. Il device agent di Propeller sostituisce il firmware Zephyr come punto
di ricezione dei manifest.

```
CLOUD     Kubernetes + Propeller Server
             ↕  Propeller API / MQTT
FOG/EDGE  Propeller Agent (su device o su bridge)
             ↕  OCRE o WAMR raw
EDGE      Zephyr+WAMR (invariato lato runtime)
```

Pro: eliminazione di tutto il codice cloud custom, allineamento all'ecosistema
LF Edge, manutenzione upstream.

Contro: si perde il protocollo TLS+CBOR proprietario (`wasmbed-protocol`) e
l'intera filiera Renode (`wasmbed-qemu-manager`) deve essere re-integrata come
"device provider" Propeller (non esiste tale adapter upstream). L'enrollment
Ed25519 con CRD lookup per chiave pubblica (vedi [`TLS_ENROLLMENT_COMPLETE.md`](./TLS_ENROLLMENT_COMPLETE.md))
andrebbe riscritto contro il modello identità Propeller.

#### Modello B — Propeller come upstream control-plane, RETROSPECT come Gateway

Propeller Server sostituisce `wasmbed-api-server` e `wasmbed-application-controller`,
ma il Gateway resta invariato. Il Gateway diventa un "Propeller device endpoint"
che riceve i manifest Propeller, li traduce in messaggi CBOR e li inoltra via
TLS ai device.

```
CLOUD     Kubernetes + Propeller Server
             ↕  Propeller API
FOG       wasmbed-gateway (adapter Propeller → CBOR)
             ↕  TLS 1.3 + CBOR
EDGE      Zephyr+WAMR (invariato)
```

Pro: si mantiene il protocollo TLS+CBOR e tutta la filiera Renode. Si elimina
solo il codice dell'api-server e dei controller.

Contro: bisogna scrivere l'adapter Propeller↔Gateway (non esiste upstream).
La complessità spostata nell'adapter potrebbe pareggiare quella eliminata nei
controller.

#### Modello C — Propeller come riferimento "Related Work" (senza integrazione)

Propeller viene usato come benchmark comparativo nel capitolo "Lavori correlati"
della tesi, senza alcuna integrazione. Questo modello è il più pulito dal
punto di vista della tesi: RETROSPECT è posizionato come implementazione
originale e Propeller come lo stato dell'arte con cui si confronta.

### 3.4 Cosa Propeller sostituirebbe (per componente)

| Componente RETROSPECT | Sostituito da Propeller? | Note |
|---|---|---|
| `wasmbed-api-server` | **Sì** (Modello A e B) | Propeller Server ha API equivalenti |
| `wasmbed-application-controller` | **Sì** (Modello A e B) | Propeller fa il reconcile manifest→device |
| `wasmbed-device-controller` | **Sì** (Modello A) | Propeller ha device registry proprio |
| `wasmbed-gateway-controller` | **Sì** (Modello A) | Propeller gestisce il ciclo di vita del bridge |
| `wasmbed-gateway` (TLS+CBOR hub) | **Parzialmente** (Modello A) | Propeller usa MQTT/HTTPS, non TLS+CBOR custom |
| `wasmbed-protocol` (CBOR) | **Sì** (Modello A), **No** (Modello B) | Nel Modello B il Gateway resta adapter |
| `Application` CRD | **Sostituito o mappato** | I manifest Propeller hanno semantica analoga |
| `wasmbed-qemu-manager` (Renode) | **No** | Propeller non gestisce emulazione device |
| `zephyr-app/` + WAMR | **No** | Propeller spinge il `.wasm` verso WAMR/OCRE on-device |
| Dashboard React | **Sostituita o affiancata** | Propeller ha propria UI |

### 3.5 Cosa Propeller non offre rispetto a RETROSPECT

- **Emulazione device integrata.** `wasmbed-qemu-manager` con Renode è
  specifico di RETROSPECT e non ha equivalente in Propeller. Propeller assume
  hardware fisico o VM pre-esistente — il provisioning dell'emulatore va
  gestito esternamente.

- **Enrollment Ed25519 con CRD lookup per chiave pubblica.** Il flusso descritto
  in [`TLS_ENROLLMENT_COMPLETE.md`](./TLS_ENROLLMENT_COMPLETE.md) (device
  presenta chiave pubblica → Gateway cerca `Device` CRD → aggiorna status)
  non corrisponde al modello identità Propeller, che usa token o certificati
  gestiti dal proprio PKI.

- **Integrazione k3s/CRD Kubernetes nativa.** RETROSPECT usa CRD (`Device`,
  `Application`, `Gateway`) come fonte di verità. Propeller può girare in
  Kubernetes ma non espone CRD come primitive primarie.

- **Multi-MCU emulato con 13+ board.** Il catalogo `McuType` (STM32, RISC-V
  virt, Cortex-R8, ESP32, ecc.) è una proprietà esclusiva dello stack di test
  di RETROSPECT.

- **Protocollo unificato MCU+MPU.** Il crate `wasmbed-protocol` (`no_std`)
  funziona identicamente su firmware Zephyr e daemon Linux (`wasmbed-edge-client`),
  garantendo un wire format comune per device eterogenei. Propeller separa i
  percorsi per device class.

---

## 4. Allineamento con la topologia OCRE esistente

Le analisi di Phoenix e Propeller si innestano senza contraddire le decisioni
già prese nei documenti OCRE:

**Phoenix Scenario A** aggiunge un `DeviceClass::MpuPhoenix` accanto a
`McuConstrained` e `MpuRich` già definiti in
[`OCRE_EDGE_INTEGRATION.md`](./OCRE_EDGE_INTEGRATION.md) (§2.2). Il Gateway
e il protocollo CBOR restano invariati — coerente con l'Opzione A
("Edge eterogeneo, Gateway invariato") di
[`OCRE_DEPLOYMENT_TOPOLOGY.md`](./OCRE_DEPLOYMENT_TOPOLOGY.md).

**Propeller Modello B** rispetterebbe lo stesso confine architetturale
dell'Opzione A: il Gateway resta il choke point TLS, i device restano ignari
del control-plane cloud. L'unica differenza è che il lato cloud non usa codice
RETROSPECT ma Propeller Server come upstream.

**OCRE e Propeller sono complementari**: OCRE è il runtime on-device (come
analizzato in [`OCRE_INTEGRATION_ANALYSIS.md`](./OCRE_INTEGRATION_ANALYSIS.md)),
Propeller è il control-plane cloud. Un'adozione congiunta Propeller (cloud) +
OCRE (firmware Zephyr) rappresenta lo stack LF Edge "canonical" — ed è il
principale punto di confronto da citare nel capitolo "Related Work" della tesi
per posizionare RETROSPECT come implementazione alternativa con caratteristiche
proprie (Renode, TLS+CBOR custom, CRD K8s native).

---

## 5. Sintesi decisionale

| Tecnologia | Sostituisce | Costo di adozione | Beneficio principale | Verdetto per RETROSPECT |
|---|---|---|---|---|
| **Phoenix RTOS** (Scenario A — MPU) | `wasmbed-edge-client` su Linux | Medio (porting tokio) | Isolamento MMU su device MPU | Non giustificato: sandbox è già WASM |
| **Phoenix RTOS** (Scenario B — MCU) | Intero `zephyr-app/` + WAMR + Renode | Molto alto (porting WAMR, emulatore) | Microkernel su MCU | Sconsigliato: costo/beneficio negativo |
| **Propeller** (Modello A — sostituzione) | `api-server` + controllers + gateway | Alto (adattare Renode, rifare enrollment) | Allineamento LF Edge upstream | Utile solo come evoluzione post-tesi |
| **Propeller** (Modello B — solo cloud) | `api-server` + application-controller | Medio (adapter Propeller↔Gateway) | Riduzione codice cloud custom | Interessante ma non prioritario |
| **Propeller** (Modello C — Related Work) | Niente | Nullo | Posizionamento accademico | **Consigliato per la tesi** |
| **OCRE** (Zephyr, MCU) | Solo `wamr_integration.c` | Medio (upgrade Zephyr 4.4 richiesto) | Host functions GPIO/sensori, narrativa container | Roadmap futura documentata in `OCRE_INTEGRATION_ANALYSIS.md` |

---

## 6. Riferimenti

### Documentazione interna

| Documento | Argomento |
|---|---|
| [`OCRE_INTEGRATION_ANALYSIS.md`](./OCRE_INTEGRATION_ANALYSIS.md) | Analisi costi/benefici OCRE su firmware Zephyr; incompatibilità upstream 2026-05 |
| [`OCRE_EDGE_INTEGRATION.md`](./OCRE_EDGE_INTEGRATION.md) | Modifiche CRD (`TargetRuntime`, `DeviceClass`), crate `wasmbed-edge-client`, WASI abilitato |
| [`OCRE_DEPLOYMENT_TOPOLOGY.md`](./OCRE_DEPLOYMENT_TOPOLOGY.md) | Decisione Opzione A; confronto con Opzione B e C |
| [`ARCHITECTURE.md`](./ARCHITECTURE.md) | Architettura generale RETROSPECT |
| [`MCU_SUPPORT.md`](./MCU_SUPPORT.md) | Catalogo completo board Zephyr supportate |
| [`FIRMWARE.md`](./FIRMWARE.md) | Struttura `zephyr-app/`, build west, CMakeLists.txt |
| [`TLS_ENROLLMENT_COMPLETE.md`](./TLS_ENROLLMENT_COMPLETE.md) | Flusso enrollment Ed25519 e lookup CRD |
| [`../../MasterThesis/ZEPHYR_OCRE_GUIDE.md`](../../MasterThesis/ZEPHYR_OCRE_GUIDE.md) | Esempio pratico Zephyr+WAMR su QEMU x86; baseline OCRE |

### Risorse esterne

> Le URL seguenti sono fornite a titolo indicativo. Verificarne l'attualità
> prima di citarle nella tesi — i progetti sono in sviluppo attivo.

- **Phoenix RTOS**: `https://phoenix-rtos.com` — documentazione ufficiale,
  board supportate, toolchain.
- **Phoenix RTOS su GitHub**: `https://github.com/phoenix-rtos` —
  sorgenti `phoenix-rtos-kernel`, `phoenix-rtos-project`.
- **OCRE (Open Container Runtime for Embedded)**: `https://github.com/project-ocre/ocre-runtime` —
  progetto LF Edge/Atym; `west.yml` rivela le dipendenze Zephyr e le board
  supportate.
- **Atym (già LF Edge)**: `https://github.com/atym-io` — organizzazione
  upstream per OCRE e Propeller.
- **Propeller**: cercare `atym-io/propeller` su GitHub o nella documentazione
  Atym per la versione aggiornata; il progetto è in sandbox CNCF/LF Edge.
