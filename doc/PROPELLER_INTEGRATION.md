# Propeller — Integrazione in RETROSPECT: guida ai prossimi step

**Data**: maggio 2026
**Riferimenti**: https://propeller.absmach.eu · https://github.com/absmach/propeller
**Licenza**: Apache-2.0
**Stato**: Step 1–2 completati e verificati (smoke test WASM passato, 16 maggio 2026)

---

## 1. Cos'è Propeller

Propeller è un orchestratore open-source per workload WebAssembly attraverso il continuum
Cloud-Edge. È sviluppato da [Abstract Machines](https://absmach.eu) nell'ecosistema
Magistrala/LF Edge e punta a deployare moduli `.wasm` portabili da un registry OCI fino
a microcontrollori con Zephyr RTOS, con cold start dichiarato < 10 ms e isolamento
100% WASM sandbox.

### Componenti

| Componente | Linguaggio | Ruolo |
|---|---|---|
| **Propeller Manager** | Go | Control-plane cloud: REST API, scheduler, registry task WASM, integrazione OCI |
| **Proplet** | Rust | Agent/runtime edge su Zephyr RTOS: riceve task via MQTT, esegue moduli WASM |
| **Proxy** | Go | Fetch moduli da OCI registry, autenticazione |
| **Magistrala** | Go | Piattaforma IoT multi-tenant: broker MQTT, gestione identità (domain/client/channel) |

### Stack di comunicazione

```
Cloud: Propeller Manager  ──→  REST API (port 7070)
                          ──→  Magistrala MQTT broker
Edge:  Proplet            ←──  MQTT (subscribe task channel)
                          ──→  MQTT (publish result/heartbeat)
```

---

## 2. Cosa sostituisce in RETROSPECT

L'adozione di Propeller comporta la **rimozione di TLS+CBOR** come protocollo
Cloud↔Device e la sostituzione dell'intero stack di comunicazione con MQTT via Magistrala.
Non si tratta di una migrazione graduale: Propeller è un rimpiazzo completo del control-plane
e del gateway. I crates `wasmbed-gateway` e `wasmbed-protocol` vengono dismessi.

| Componente RETROSPECT | Sostituito da | Note |
|---|---|---|
| `wasmbed-api-server` | **Propeller Manager** | REST API task WASM nativa, upload diretto o via OCI |
| `wasmbed-application-controller` | **Propeller Manager** scheduler | Reconcile manifest → device integrato |
| `wasmbed-gateway` **(TLS+CBOR — rimosso)** | **Magistrala** broker + **Proplet** | Protocollo Cloud↔Device diventa MQTT su TLS |
| `wasmbed-protocol` **(lib CBOR — rimossa)** | Protocollo MQTT Propeller (interno) | Non più necessario |
| `wasmbed-device-controller` | **Propeller Manager** (provisioning Proplet) | Enrollment via credenziali Magistrala |
| Firmware Zephyr + WAMR custom | **Proplet** (Rust su Zephyr) | Runtime WASM gestito da Propeller |
| `Application` CRD K8s | Task JSON / OCI image manifest | Source of truth diventa Propeller Manager |
| `Device` CRD K8s | Proplet registrato in Magistrala | Identità basata su Magistrala domain/client |
| Dashboard React | Adapter verso Propeller REST API | Layer proxy minimo sul `wasmbed-api-server` |

**Cosa NON cambia**: cluster K3s rimane (Manager gira come pod); OCI registry locale
`localhost:5000` riutilizzato; network TAP/dnsmasq per native_sim invariata.

---

## 3. Note per i test

Per i test di questa tesi **non è necessario migrare niente** dall'architettura attuale:
Propeller viene avviato come stack standalone separato. In particolare:

- **Il cambio protocollo non richiede lavoro**: MQTT è il protocollo interno di Propeller,
  già implementato nel Manager e nel Proplet. Non scriviamo niente di quel layer.
- **Le credenziali Magistrala sono generate da zero** con `propeller-cli provision`: nessuna
  compatibilità richiesta con le chiavi Ed25519 esistenti.
- **La board di test è `native_sim`**: non serve validare altre board. Il Proplet su
  `native_sim` è il target di riferimento per i test funzionali (deploy WASM → esecuzione
  → verifica risultato).

Il perimetro di test è quindi: **Propeller standalone + Proplet su native_sim + modulo WASM
di esempio**.

---

## 4. Prerequisiti

### Strumenti richiesti

```bash
go version          # >= 1.26.0
rustup toolchain list
docker --version    # >= 20.10
make --version      # >= 3.81
which mosquitto_pub mosquitto_sub
```

Opzionali per sviluppo WASM locale: TinyGo 0.34.0+, Wasmtime, ORAS CLI.

### Infrastruttura esistente riutilizzabile

- K3s cluster già attivo (`kubectl cluster-info`)
- OCI registry locale su `localhost:5000`
- Network TAP (`tap0`, dnsmasq, iptables) già configurata

---

## 5. Step di integrazione

### Step 1 — Bootstrap Propeller standalone

**Prerequisiti**: Go >= 1.26 (`/usr/local/go/bin`), Rust toolchain, Docker, wabt (`sudo apt install wabt`).

```bash
git clone https://github.com/absmach/propeller.git ~/propeller
cd ~/propeller

# Build binari Go (manager, cli, proxy)
export PATH="$PATH:/usr/local/go/bin:$HOME/go/bin"
make manager cli proxy -j$(nproc)
export GOBIN="$HOME/go/bin" && make install

# Build WASM di esempio (non richiede TinyGo)
make addition-wat   # genera build/addition-wat.wasm e build/addition-wat.b64

# Provisioning manuale (propeller-cli provision richiede TTY interattivo):
# usa lo script start-propeller-stack.sh che gestisce tutto incluso il fix DNS nginx.
# Oppure procedi manuale:

# 1. Ferma mosquitto se occupa porta 1883
sudo snap stop mosquitto

# 2. Avvia Magistrala
make start-magistrala
# Nota: nginx docker service non si aggancia alla rete magistrala-base-net automaticamente.
# Fix necessario dopo che nginx è up:
docker network disconnect magistrala-base-net magistrala-nginx 2>/dev/null || true
docker network connect --alias nginx magistrala-base-net magistrala-nginx
# Attendere che tutti i servizi siano stable (~60-90s)

# 3. Generare credenziali via REST API Magistrala (admin/12345678 = default .env)
# Vedi script start-propeller-stack.sh per automazione completa.
# Credenziali di esempio già nel config.toml:
#   domain_id  = "69df50b5-2111-42ae-a9e1-fdd4d2b0b54e"
#   manager client_id  = "da9791c8-5a9b-49ac-9c06-a1bd50bbd95a"
#   proplet  client_id = "45f0f099-a6c7-4e74-ac1c-ea1accf92530"
#   channel_id = "ae9fb9bf-3697-41f6-beb9-85d1d71954f5"

# 4. Avvia Manager + Proplet + Proxy
make start-propeller

# Verifica
curl http://localhost:7070/health
curl http://localhost:7070/proplets   # deve mostrare total:1, alive:true
```

**Script completo**: `~/propeller/start-propeller-stack.sh` automatizza tutti i passaggi sopra.

**Stato verificato** (16 maggio 2026): Manager health `{"status":"pass","version":"v0.4.0"}`,
1 Proplet registrato con `wasm_runtime: wasmtime`, alive=true.

### Step 2 — Deploy task WASM di esempio (smoke test)

**Stato verificato**: completato con successo il 16 maggio 2026.

Il Proplet usa il **nome del task** come nome della funzione WASM da invocare, e il campo
`inputs` come argomenti. Il modulo `addition-wat.wasm` esporta la funzione `add(i32,i32)->i32`.

```bash
# Crea task e carica WASM
TASK_ID=$(curl -s -X POST http://localhost:7070/tasks \
  -H "Content-Type: application/json" \
  -d '{"name":"add","inputs":["10","20"]}' | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

curl -s -X PUT  "http://localhost:7070/tasks/${TASK_ID}/upload" \
  -F "file=@build/addition-wat.wasm" > /dev/null

curl -s -X POST "http://localhost:7070/tasks/${TASK_ID}/start"

# Verifica risultato (state 3 = completed)
curl -s "http://localhost:7070/tasks/${TASK_ID}" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('state:', d['state'], 'results:', d.get('results'))"
# Output atteso: state: 3 results: 30
```

Stato task: `0`=pending, `1`=assigned, `2`=running, `3`=completed, `4`=failed.

Note:
- `addition-wat.wasm` (41 byte) è buildato da `make addition-wat` — non richiede TinyGo.
- Gli esempi Go (`addition.wasm`, `compute.wasm`) richiedono TinyGo <= 0.34 con Go <= 1.23;
  non compatibili con Go 1.26. Per i test usa `addition-wat.wasm`.
- Il runtime WASM del Proplet containerizzato è `wasmtime` (non WAMR).

### Step 3 — Deploy da OCI registry locale

```bash
oras push localhost:5000/wasmbed/add-wasm:v1 build/addition.wasm:application/wasm

curl -X POST http://localhost:7070/tasks \
  -H "Content-Type: application/json" \
  -d '{"name":"add","image_url":"localhost:5000/wasmbed/add-wasm:v1","inputs":[10,20]}'
```

### Step 4 — Portare Proplet su native_sim

Il Proplet è scritto in Rust e gira come Zephyr thread. Sorgenti in `propeller/proplet/`.

1. Verificare se Proplet usa WAMR come backend (guardare `propeller/proplet/Cargo.toml`).
   Se sì, OCRE+WAMR già nel workspace sono riutilizzabili.

2. Integrare nella board config `native_sim_64`:
   ```bash
   # Adattare prj.conf: aggiungere CONFIG_PROPLET=y se OCRE lo espone,
   # oppure sostituire main.c con l'entry point Proplet
   west build -b native_sim/native/64 /home/ubuntu/Thesis/retrospect/zephyr-app \
     --build-dir build/native_sim_64 --pristine
   ```

3. Configurare credenziali Magistrala nel firmware (da `config.toml`):
   `domain_id`, `client_id`, `client_key`, `channel_id` — passate via variabili d'ambiente
   o iniettate in SRAM come oggi si fa con l'endpoint gateway.

### Step 5 — Esporre Magistrala su NodePort K8s

```yaml
# magistrala-nodeport.yaml
apiVersion: v1
kind: Service
metadata:
  name: magistrala-mqtt-nodeport
  namespace: wasmbed
spec:
  type: NodePort
  ports:
    - port: 1883
      targetPort: 1883
      nodePort: 31883
      name: mqtt
    - port: 8883
      targetPort: 8883
      nodePort: 31884
      name: mqtts
  selector:
    app: magistrala-mqtt
```

```bash
kubectl apply -f magistrala-nodeport.yaml
# Proplet si connette a <node-ip>:31883
```

### Step 6 — Adattare la Dashboard (opzionale per i test)

Aggiungere proxy trasparente in `wasmbed-api-server`:
```
GET/POST /api/v1/tasks   → http://propeller-manager:7070/tasks
GET      /api/v1/proplets → http://propeller-manager:7070/proplets
```
La Dashboard React esistente può consumare queste route senza riscrittura completa.

---

## 6. Verifica E2E

```bash
# Magistrala up
docker ps | grep magistrala

# Manager raggiungibile
curl http://localhost:7070/health

# Almeno un Proplet registrato
curl http://localhost:7070/proplets | jq '.[].id'

# Task completato
curl http://localhost:7070/tasks/<id> | jq '.state'   # → 3
curl http://localhost:7070/tasks/<id> | jq '.result'  # → "30" (inputs [10,20])
```

---

## 7. Multi-Proplet (scale out)

```bash
propeller-cli provision add-proplets

PROPLET_CONFIG_SECTION=proplet2 propeller-proplet
PROPLET_CONFIG_SECTION=proplet3 propeller-proplet
```

---

## 8. Considerazioni per deploy produzione

Questi aspetti non impattano i test ma sono rilevanti per un'integrazione completa:

- **Rimozione TLS+CBOR**: `wasmbed-gateway` e `wasmbed-protocol` vanno dismessi.
  Il traffico Cloud↔Device transita interamente su MQTT (Magistrala gestisce TLS e
  autenticazione a livello broker). Non è un cambio incrementale: i due stack non
  coesistono.
- **Identità device**: il modello Ed25519 + enrollment attuale non è compatibile con
  le credenziali Magistrala (domain/client/channel). Per produzione serve decidere se
  mantenere la CA attuale integrandola come provider identità Magistrala, o migrare
  completamente al modello Magistrala.
- **CRD Kubernetes**: `Application` CRD e `Device` CRD vengono dismesse; Propeller
  Manager diventa la source of truth per task e device.
- **Dashboard**: `wasmbed-api-server` ridotto a proxy verso Manager API, oppure
  rimpiazzato da una nuova UI.
- **Lock-in Magistrala**: Propeller dipende da Magistrala come infrastruttura MQTT;
  sostituire il broker richiede un adattamento del layer di provisioning.

---

## 9. Riferimenti

| Risorsa | URL |
|---|---|
| Sito ufficiale | https://propeller.absmach.eu |
| Getting started | https://propeller.absmach.eu/docs/getting-started |
| Repository GitHub | https://github.com/absmach/propeller |
| Magistrala | https://magistrala.absmach.eu |
| Docker images | `ghcr.io/absmach/propeller/manager:latest` · `proplet:latest` · `proxy:latest` |
| Analisi comparativa RETROSPECT | [`RTOS_AND_RUNTIME_ALTERNATIVES.md`](./RTOS_AND_RUNTIME_ALTERNATIVES.md) |
