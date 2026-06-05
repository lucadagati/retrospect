# Polyglot Wasm Demo — SpinKube

> Dimostrazione pratica del pattern **"scrivi in qualsiasi linguaggio → compila in WebAssembly → stesso runtime"** su SpinKube, con stato condiviso tra microservizi in linguaggi diversi.

---

## Cosa dimostra questa applicazione

### Il claim: "any language → WebAssembly → same runtime"

Questa demo dimostra: **qualsiasi linguaggio può essere compilato in WebAssembly e girare sullo stesso runtime**, condividendo risorse con altri componenti scritti in linguaggi completamente diversi.

La differenza è sottile ma importante:

| | `hello-wasm` (MasterThesis) | `polyglot-guestbook` (questa demo) |
|---|---|---|
| Claim | Stesso `.wasm`, ambienti diversi | Linguaggi diversi, stesso runtime |
| Portabilità | Cloud (Spin) + Edge (WAMR/Zephyr) | Rust + Go → stesso shim Kubernetes |
| Dipendenze host | Solo WASI standard | Spin host functions (KV, HTTP trigger) |
| "Write once, run anywhere" | ✓ in senso stretto | ✗ — i binari sono legati al runtime Spin |

**Perché `polyglot-guestbook` non è "write once, run anywhere":** i componenti usano l'SDK di Spin (`spin_key_value_open`, `wasi:http/incoming-handler`) che sono host functions specifiche del runtime Spin. WAMR su Zephyr espone solo WASI standard e non conosce queste funzioni — il caricamento fallirebbe con "import not satisfied". Per girare su WAMR bisognerebbe riscrivere i componenti senza Spin SDK, perdendo KV store e HTTP trigger nativi.

**Cosa dimostriamo invece:** WebAssembly come **target di compilazione universale**. Rust e Go sono linguaggi con paradigmi, gestione della memoria e toolchain radicalmente diversi. Entrambi producono lo stesso formato `.wasm` (`wasm32-wasip1`), vengono impacchettati in un unico artefatto OCI, ed eseguiti dallo stesso `containerd-shim-spin` all'interno dello stesso pod Kubernetes — condividendo stato tramite il KV store built-in di Spin. Il runtime non sa né gli importa in quale linguaggio sia scritto ciascun componente.

### La punchline dimostrabile con `curl`

```
POST /edge/add  "temperatura 42C"   ← scrive il componente Rust
POST /go/add    "anomalia rilevata" ← scrive il componente Go

GET  /edge/list  →  [0] [edge] temperatura 42C
                    [1] [go]   anomalia rilevata   ← vede anche i dati Go!

GET  /go/list    →  identico                       ← stesso store condiviso
```

Rust e Go, codice diverso, linguaggi diversi, un singolo `.wasm` ciascuno, **stesso KV store**, **stesso pod**, **stesso shim**.

---

## Architettura

```
┌─────────────────────────────────────────────────────────────────────┐
│  Kubernetes — namespace poc-runtime                                  │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Pod: polyglot-guestbook         (runtimeClass: wasmtime-spin-v2)│
│  │                                                               │  │
│  │  ┌──────────────────────────┐  ┌──────────────────────────┐  │  │
│  │  │  edge-collector (Rust)   │  │  go-telemetry (Go)       │  │  │
│  │  │  route: /edge/...        │  │  route: /go/...          │  │  │
│  │  │  spin-sdk 5.2.0          │  │  spin-go-sdk/v2 + TinyGo │  │  │
│  │  │  → wasm32-wasip1         │  │  → wasm32-wasip1         │  │  │
│  │  └──────────┬───────────────┘  └──────────┬───────────────┘  │  │
│  │             │                             │                   │  │
│  │             └──────────┬──────────────────┘                   │  │
│  │                        ▼                                      │  │
│  │          Spin KV store "default" (SQLite in-pod)             │  │
│  │                  chiave: "telemetry"                          │  │
│  │                                                               │  │
│  │  Runtime: containerd-shim-spin v0.24.0 (Spin v3, WASIp1/p2) │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  Service: polyglot-guestbook  (ClusterIP :80)                       │
└─────────────────────────────────────────────────────────────────────┘
```

**Flusso dati:**

1. `POST /edge/add <body>` → edge-collector legge KV `"telemetry"`, appende `[edge] <body>`, riscrive
2. `POST /go/add <body>` → go-telemetry fa lo stesso con prefisso `[go]`
3. `GET /edge/list` o `GET /go/list` → entrambi leggono lo stesso KV → stessi dati

Il KV store `"default"` di Spin è un SQLite locale al pod: nessun componente esterno, nessun Redis, nessun database separato. È parte del runtime Spin stesso.

---

## Stack tecnologico

| Componente | Linguaggio | Toolchain | Output |
|---|---|---|---|
| `edge-collector` | Rust | `rustc 1.88` + `cargo` | `wasm32-wasip1` |
| `go-telemetry` | Go | `TinyGo 0.34` via Docker | `wasm32-wasip1` |
| Runtime K8s | — | `containerd-shim-spin v0.24.0` | Spin v3 / WASIp1 |
| Packaging | — | `spin registry push` | OCI artifact |
| Registry | — | GHCR + zot locale | `ghcr.io/antoniodev0/polyglot-guestbook:v2` |

> **Nota TinyGo**: `spin-go-sdk/v2` usa CGo con C headers che mappano le system call WASI
> (pattern `// #include "key-value.h"`). Richiede TinyGo (non il compilatore Go standard)
> perché standard Go non produce WASI modules. Build via Docker evita dipendenze locali.

---

## Struttura sorgenti

```
retrospect/example_thesis/
├── spin.toml                        # Manifest Spin: due componenti, un'app
├── edge-collector/
│   ├── Cargo.toml                   # spin-sdk = "5.2.0", crate-type = ["cdylib"]
│   └── src/lib.rs                   # #[http_component] fn handle — KV read/write
├── go-telemetry/
│   ├── go.mod                       # spin-go-sdk/v2 v2.2.1, go 1.20
│   ├── main.go                      # spinhttp.Handle — KV read/write via OpenStore
│   └── build-tinygo.sh              # docker run tinygo/tinygo:0.34.0 ...
└── k8s/
    ├── polyglot-guestbook-spinapp.yaml   # SpinApp CRD → poc-runtime
    └── redis-deployment.yaml            # (esempio di pod container, non usato)
```

---

## Prerequisiti

- K3s con SpinKube installato (spin-operator, cert-manager, RuntimeClass `wasmtime-spin-v2`)
- `spin` CLI ≥ 3.x (`spin --version`)
- `cargo` con target `wasm32-wasip1` (`rustup target add wasm32-wasip1`)
- Docker (per il build Go via TinyGo)
- Registry locale zot su `localhost:5001` **oppure** accesso a GHCR

---

## Build

```bash
cd retrospect/example_thesis

# Build entrambi i componenti
spin build
# Internamente esegue:
#   edge-collector: cargo build --target wasm32-wasip1 --release
#   go-telemetry:   docker run tinygo/tinygo:0.34.0 tinygo build -target=wasip1 ...
```

Output atteso:
```
Building component go-telemetry with `bash build-tinygo.sh`
Building component edge-collector with `cargo build --target wasm32-wasip1 --release`
Finished building all Spin components
```

---

## Push OCI

### Registry locale (zot)

```bash
# Avvia zot (se non già in esecuzione)
docker run -d --name zot-registry -p 5001:5000 ghcr.io/project-zot/zot-linux-amd64:latest

spin registry push --insecure localhost:5001/spin/polyglot-guestbook:v2
```

### GHCR (GitHub Container Registry)

```bash
# Login (richiede PAT con scope write:packages)
spin registry login ghcr.io -u antoniodev0

spin registry push ghcr.io/antoniodev0/polyglot-guestbook:v2
```

> Il package va reso **pubblico** su GitHub (Package settings → Change visibility) affinché
> K3s possa fare pull senza imagePullSecret.

---

## Deploy su K3s

```bash
# Assicurarsi che /etc/rancher/k3s/registries.yaml sia configurato per localhost:5001
# (necessario solo per il registry locale)

kubectl apply -f k8s/polyglot-guestbook-spinapp.yaml

# Attendere che il pod sia Running (di solito < 10 secondi)
kubectl -n poc-runtime get pods -w
```

---

## Verifica E2E

```bash
# Port-forward del service
kubectl -n poc-runtime port-forward svc/polyglot-guestbook 8080:80 &

# Scrivere dal componente Rust (edge)
curl -s -X POST localhost:8080/edge/add -d 'temperatura 42C dal sensore edge'
# → edge reading recorded

# Scrivere dal componente Go (cloud)
curl -s -X POST localhost:8080/go/add -d 'analytics cloud: anomalia rilevata'
# → go telemetry recorded

# Leggere da Rust — vede entrambi
curl -s localhost:8080/edge/list
# === Telemetry (edge-collector view) ===
# [0] [edge] temperatura 42C dal sensore edge
# [1] [go] analytics cloud: anomalia rilevata
# total: 2 entries

# Leggere da Go — identico
curl -s localhost:8080/go/list
# === Telemetry (go-telemetry view) ===
# [0] [edge] temperatura 42C dal sensore edge
# [1] [go] analytics cloud: anomalia rilevata
# total: 2 entries

# Verificare che il pod usi il runtime Wasm
kubectl -n poc-runtime get pod -l app.kubernetes.io/name=polyglot-guestbook \
  -o jsonpath='{.items[0].spec.runtimeClassName}'
# → wasmtime-spin-v2
```

---

## Relazione con SPINKUBE_RUNTIME_ORCHESTRATION.md

Questa demo estende concettualmente ciò che è descritto in `SPINKUBE_RUNTIME_ORCHESTRATION.md`:

- **Lì**: architettura SpinKube, componenti operator/shim/RuntimeClass, coesistenza `runc`+`spin`
- **Qui**: applicazione concreta multi-componente, KV condiviso cross-language, build polyglot

Le due sezioni insieme raccontano: *come funziona SpinKube* + *perché ha senso usarlo*.

---

## Limitazioni note

- Il KV store `"default"` è **in-pod** (SQLite): i dati si perdono al riavvio del pod. Per persistenza serve un volume o un KV store esterno (Redis, Valkey). Il punto della demo è la condivisione cross-language, non la durability.
- `containerd-shim-spin v0.24.0` usa Spin v3 (WASIp2). I nuovi SDK Go (`spin-go-sdk/v3`) e Python richiedono Spin v4 (WASIp3) e non sono ancora compatibili con questo shim. Per questo motivo il componente Go usa `spin-go-sdk/v2` + TinyGo.
- TinyGo 0.34 è richiesto (non il Go standard); la build avviene tramite Docker per evitare dipendenze locali specifiche di versione.
