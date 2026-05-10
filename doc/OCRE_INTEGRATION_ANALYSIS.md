# Integrazione OCRE in RETROSPECT — Analisi per la Tesi

## Punto di partenza

RETROSPECT gira già WAMR su Zephyr (`zephyr-app/src/wamr_integration.c`):
il firmware riceve un payload `.wasm` via TLS/CBOR dal Gateway, lo carica con
`wasm_runtime_load`, lo instanzia e invoca la funzione esportata. L'esecuzione
funziona su board eterogenee (STM32, ESP32, RISC-V virtual, Cortex-R8 virtual)
perché WAMR interprete è ISA-agnostico per costruzione.

**OCRE** (Open Container Runtime for Embedded, LF Edge / atym-io) è un layer
Zephyr che si posiziona *sopra* WAMR e aggiunge:
- API lifecycle container (`create / run / stop / destroy`).
- Suite di host functions standard (GPIO, sensori, timer, messaging, RNG)
  mappate su Zephyr driver API, indipendenti dalla board.
- Modello "container" come unità deployabile (manifesto + bytecode + limiti
  di risorse), distinto dal concetto raw di modulo WASM.

---

## Cosa cambia integrando OCRE

### Lato firmware (zephyr-app)

Oggi il firmware chiama WAMR direttamente e non registra host functions
(`init_args.native_symbols = NULL`). Un'app WASM non può accedere a GPIO,
sensori o timer — può solo eseguire logica pura o fare socket WASI se si
abilitano le flag WASI (oggi disabilitate: `WAMR_BUILD_LIBC_WASI=0`).

Con OCRE:
- `wamr_integration.c` diventa un thin wrapper sopra
  `ocre_container_runtime_*`.
- Le host functions (GPIO, sensori, timer) vengono registrate
  automaticamente da OCRE al boot — le app WASM le importano come normali
  funzioni WASI-like senza sapere nulla di Zephyr.
- Il payload smette di essere un "modulo" e diventa un "container": porta
  con sé un manifesto che dichiara i limiti di memoria e le capability
  richieste, che OCRE valida prima dell'esecuzione.

**File toccati**: `CMakeLists.txt` (aggiunta OCRE come modulo west),
`wamr_integration.{c,h}` (refactor completo), `wasmbed_protocol.c` nella
funzione `handle_deploy_application` (traduzione CBOR → `ocre_container_data`).

### Lato control-plane (crates Rust + CRD)

Il Gateway oggi invia `DeployApplication { wasm_bytes, config }` via CBOR.
Per supportare OCRE servirebbe estendere `ApplicationConfig`
(`crates/wasmbed-protocol`) con un manifesto strutturato (limiti RAM, lista
capability), e aggiungere un campo `spec.runtime.target` alla Application CRD
(`crates/wasmbed-k8s-resource/src/application.rs`) per distinguere
`wamr_raw` da `ocre` a livello di intent Kubernetes.

---

## Conviene integrare OCRE?

### Motivi per cui sì

**1. API hardware-independent formalizzata.**
Oggi l'hardware-independence è garantita dall'interprete WAMR ma solo per la
logica pura: un'app che vuole leggere un sensore deve conoscere l'API Zephyr
specifica della board. Con OCRE le host functions sono identiche su qualsiasi
board — l'app importa `sensor_read()` e non sa se gira su STM32 o RISC-V.
Questo è il salto qualitativo rispetto allo stato attuale.

**2. Coerenza narrativa con il titolo "container per embedded".**
Usare OCRE consente di affermare che RETROSPECT implementa un vero modello
a container (manifesto + sandbox + lifecycle gestito) e non solo un loader
WASM. Il termine "OCRE" nella tesi diventa tecnico e preciso, non solo
concettuale.

**3. Allineamento con l'ecosistema LF Edge.**
OCRE è un progetto LF Edge attivo — citarlo come dipendenza posiziona la tesi
in un contesto industriale reale e apre confronti con lavori correlati.

**4. Host functions gratis.**
Registrare GPIO/sensori/timer manualmente su WAMR puro richiede molto
boilerplate C specifico per board. OCRE fornisce questo layer già scritto e
testato su Zephyr: è una funzionalità che altrimenti andrebbe implementata
da zero.

### Motivi per cui no (o per cui aspettare)

**1. OCRE è un progetto giovane.**
Il supporto di board in OCRE upstream è limitato; Cortex-R8 virtual e
RISC-V virtual (usati in RETROSPECT) potrebbero non avere binding DTS
pronti. Richiederebbe contribuire upstream o mantenere patch locali.

**2. Refactor non banale con rischio di regressione.**
`wamr_integration.c` è il cuore del firmware già verificato (TLS + enrollment
+ DeployAck funzionanti). Riscriverlo introducendo OCRE rompe l'unico
percorso end-to-end collaudato; servirebbe un piano di test rigoroso prima
di considerarlo stabile.

**3. Il problema principale non è il runtime, è il networking.**
Il gap più immediato tra MasterThesis e RETROSPECT non è l'astrazione del
ciclo di vita container, ma l'assenza di WASI networking lato device
(socket, address pool). Questo si risolve abilitando `WAMR_BUILD_LIBC_WASI=1`
e `WAMR_BUILD_LIBC_WASI_NETWORK=1` — due righe in `CMakeLists.txt`, nessuna
dipendenza esterna. OCRE non aggiunge nulla qui.

**4. Footprint RAM.**
OCRE aggiunge overhead rispetto a WAMR puro. Con il pool attuale da 48 KB,
su MCU reali (STM32F746G ha 340 KB SRAM) resta fattibile, ma va profilato
prima di affermare che funziona su tutte le board supportate.

---

## Verdetto

**Integrare OCRE conviene se l'obiettivo della tesi è dimostrare un modello
a container formale e hardware-agnostico con accesso all'hardware
(sensori/GPIO) dalla WASM app.** In quel caso OCRE risolve esattamente il
problema e la narrazione è pulita.

**Non conviene come primo passo** se il tempo è limitato o se il focus è
validare il flusso cloud-fog-edge end-to-end: il rischio di regressione sul
firmware è alto, il supporto di board è incerto, e i benefici principali
(hardware-independence, networking sandboxato) si ottengono in parte già
abilitando WASI su WAMR puro.

**Percorso suggerito per la tesi:**
1. Abilitare WASI + address pool su WAMR (1–2 settimane, nessuna dipendenza
   esterna) — dimostra networking sandboxato hardware-independent.
2. Aggiungere OCRE sopra su una board certificata (STM32F746G) con host
   functions GPIO/sensori (3–4 settimane) — dimostra il modello container
   completo.
3. Confronto sperimentale `wamr_raw` vs `ocre` sullo stesso control-plane —
   fornisce dati per il capitolo "Risultati".

---

## Analisi Compatibilità Upstream — 2026-05-10

> **Risultato: integrazione OCRE upstream rinviata.**
> Branch `OCRE-Integration-Test` creato per l'analisi; nessuna modifica al
> firmware. I dettagli tecnici qui documentati costituiscono la sezione
> "Incompatibilità e Future Work" del capitolo "Risultati" della tesi.

### Versione OCRE esaminata

Repository: `github.com/project-ocre/ocre-runtime` (branch `main`,
aggiornato 2026-05-08).

### Incompatibilità rilevate

| Dimensione | OCRE upstream | RETROSPECT attuale | Problema |
|---|---|---|---|
| **Zephyr versione** | **v4.4.0** (da `west.yml` del repo) | **Zephyr 3.5** (workspace build attivo) | Gap di una major release; Kconfig, DTS, driver API cambiate tra 3.5 e 4.x. Richiederebbe aggiornamento completo del workspace west e verifica di tutte le patch WAMR applicate in Fase C. |
| **Board supportate** | `native_sim`, `b_u585i_iot02a`, `pico_plus2/rp2350b/m33` | **STM32F746G Disco** (unica board E2E verificata) | STM32F746G non è nella lista ufficiale OCRE. Il porting richiederebbe aggiungere binding DTS/Kconfig board-specific o contribuire upstream. |
| **RAM overhead** | `HEAP_MEM_POOL_ADD_SIZE_OCRE = 32 KB` (da `zephyr/Kconfig`) | Headroom attuale **~8 KB** (91.7% RAM a 248/256 KB) | La sola attivazione di `CONFIG_OCRE=y` supererebbe il budget RAM disponibile su STM32F746G senza riduzioni altrove. |
| **WAMR istanza** | WAMR embeds come submodule proprio (`wasm-micro-runtime/`) | WAMR in `retrospect/wamr/` (submodule separato, patchato per Zephyr 3.5) | Due istanze WAMR in conflitto; OCRE usa la propria revision e non accetta un'istanza esterna. Le 4 patch Fase C (compat Zephyr 3.5) andrebbero riapplicate sulla versione OCRE. |
| **Storage richiesto** | LittleFS `/lfs/ocre` (partition flash) | Nessuna partizione flash configurata in `prj.conf` | `CONFIG_OCRE_STORAGE_PARTITION=y` richiede una flash partition hardware; STM32F746G ha flash esterna ma non mappata in Zephyr 3.5 out-of-the-box. |
| **Manifest west** | OCRE è progettato come top-level manifest (`west init -m ocre-runtime`) | Il nostro workspace usa Zephyr come manifest root | Usare OCRE come west module (non come manifest) è possibile ma non documentato upstream; richiede configurazione manuale. |

### Dipendenze Kconfig rilevate in `zephyr/Kconfig`

```
CONFIG_OCRE depends on:
  PTHREAD_IPC            (threading POSIX)
  POSIX_API              (già presente in Fase C)
  FLASH + FLASH_MAP      (non configurati per STM32F746G in Zephyr 3.5)
  NET_SOCKETS            (presente)
  HEAP_MEM_POOL_ADD_SIZE_OCRE = 32768   # +32 KB → supera headroom
```

### Percorso per abilitare OCRE in futuro

Per integrare OCRE in una revisione successiva della tesi o in produzione:

1. **Aggiornare Zephyr 3.5 → 4.4** nel workspace west. Rieseguire
   la procedura di build Fase C (`WAMR_BUILD_LIBC_WASI=1`) e le patch
   compat su `wamr/core/shared/platform/zephyr/`. Verificare E2E su
   STM32F746G con Zephyr 4.4.
2. **Aggiungere STM32F746G** come board supportata in OCRE (upstream PR
   o patch locale): DTS overlay + Kconfig fragment per `stm32f746g_disco`.
3. **Risolvere RAM**: ridurre `WAMR_HEAP_SIZE` (da 64 KB, oggi minimo
   funzionale per WASI) oppure scegliere una board con più SRAM
   (es. `b_u585i_iot02a` ha 786 KB SRAM — ampio margine).
4. **Configurare flash partition**: aggiungere `stm32f746g_disco.overlay`
   con LittleFS su QSPI o internal flash per soddisfare
   `CONFIG_OCRE_STORAGE_PARTITION`.
5. Procedere con i passi 2–6 del piano originale (wrapper OCRE API,
   manifest plumbing, GPIO host function smoke test).

### Impatto sulla tesi

L'analisi di compatibilità dimostra che OCRE è una dipendenza credibile
e tecnica per la roadmap di RETROSPECT, ma non integrabile senza un
aggiornamento di Zephyr. La Fase C (WAMR+WASI E2E verificata) costituisce
il contributo tecnico principale per il capitolo "Risultati"; OCRE viene
presentato come l'evoluzione naturale documentata con evidenza di
incompatibilità e percorso di migrazione.

---

## File di riferimento

| File | Rilevanza |
|---|---|
| `retrospect/zephyr-app/src/wamr_integration.{c,h}` | Punto di refactor principale per OCRE |
| `retrospect/zephyr-app/CMakeLists.txt` | Flag WAMR/OCRE e manifest west |
| `retrospect/zephyr-app/src/wasmbed_protocol.c` | `handle_deploy_application` |
| `crates/wasmbed-protocol/src/lib.rs` | `ApplicationConfig`, `DeployApplication` |
| `crates/wasmbed-k8s-resource/src/application.rs` | Schema CRD |
| `MasterThesis/workspace/app/CMakeLists.txt` | Flag WASI da riusare |
| `MasterThesis/hello-wasm/src/main.rs` | Pattern socket FFI WAMR |
