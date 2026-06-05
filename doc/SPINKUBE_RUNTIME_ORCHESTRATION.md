# SpinKube e orchestrazione mista Docker + Wasm su Kubernetes

## La domanda

> SpinKube utilizza `containerd-shim-spin` per far girare WebAssembly. È possibile configurare lo spin-operator (o trovare una soluzione alternativa) per decidere **dinamicamente** se usare lo shim standard per i container Docker o lo shim di Spin per i workload Wasm? Come orientare l'orchestratore nella scelta del runtime corretto in base al tipo di carico di lavoro?

---

## Risposta sintetica

Lo spin-operator **non è progettato per scegliere dinamicamente** tra runtime diversi: è un controller mono-scopo che trasforma `SpinApp` in `Deployment` configurati per lo shim Spin. La scelta del runtime in Kubernetes è **statica e per-workload**, non dinamica a livello di operatore.

Il meccanismo nativo K8s che risolve il problema è la **RuntimeClass**. Lo stesso cluster — con lo stesso scheduler, gli stessi nodi, lo stesso containerd — può eseguire contemporaneamente workload Docker (via `runc`) e workload Wasm (via `containerd-shim-spin`) semplicemente scegliendo il tipo di risorsa K8s con cui descrivere il carico di lavoro:

| Tipo risorsa K8s | Runtime selezionato | Shim invocato |
|---|---|---|
| `SpinApp` (CRD SpinKube) | `wasmtime-spin-v2` | `containerd-shim-spin-v2` |
| `Deployment` (standard) | *(default)* | `containerd-shim-runc-v2` |

**L'orchestratore viene orientato dalla struttura del manifest, non da un'euristica runtime.**

---

## Come funziona: lo stack completo

```
Utente crea SpinApp              Utente crea Deployment
       │                                  │
       ▼                                  ▼
  spin-operator               kube-controller-manager
  (watch SpinApp)             (watch Deployment)
       │                                  │
       ▼                                  ▼
  crea Deployment          (Deployment già presente)
  spec.runtimeClassName:
    wasmtime-spin-v2
       │                                  │
       └──────────────┬───────────────────┘
                      ▼
              kube-scheduler
         (stesso scheduler per tutti)
                      │
                      ▼
              kubelet sul nodo
                      │
          ┌───────────┴───────────────┐
          │                           │
  runtimeClassName           nessun runtimeClassName
  = wasmtime-spin-v2          → usa default (runc)
          │                           │
          ▼                           ▼
  containerd chiama         containerd chiama
  containerd-shim-spin-v2   containerd-shim-runc-v2
          │                           │
          ▼                           ▼
   Wasmtime esegue           runc crea container
   il modulo .wasm            Linux (Docker)
```

### Il ruolo della RuntimeClass

`RuntimeClass` è una risorsa K8s (`node.k8s.io/v1`) che associa un **nome logico** (`wasmtime-spin-v2`) a un **handler** (`spin`). Il kubelet usa il nome dell'handler per trovare quale binario `containerd-shim-*` invocare.

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: wasmtime-spin-v2
handler: spin          # ← containerd cerca containerd-shim-spin-v2
```

Sul nodo, containerd deve avere questa voce nel suo config:

```toml
[plugins."io.containerd.grpc.v1.cri".containerd.runtimes.spin]
  runtime_type = "io.containerd.spin.v2"
```

E il binario `containerd-shim-spin-v2` deve essere nel PATH del nodo.

### Il ruolo di spin-operator

Lo spin-operator è un Kubernetes controller che:
1. Osserva (`watch`) le risorse `SpinApp`
2. Per ogni `SpinApp`, emette un `Deployment` con `spec.runtimeClassName: wasmtime-spin-v2`
3. Gestisce scaling, update, status

Non contiene logica di selezione runtime. La scelta è hardcoded nell'executor configurato via `SpinAppExecutor` CRD.

---

## Possibilità di selezione "dinamica"

Se si vuole un'orchestrazione più automatica — dove il sistema sceglie autonomamente il runtime senza che l'utente specifichi `SpinApp` vs `Deployment` — esistono due approcci:

### Approccio 1: MutatingAdmissionWebhook (avanzato)

Un webhook di ammissione Kubernetes intercetta ogni `Pod` in creazione e può iniettare `spec.runtimeClassName` in base a logica custom:

```
Pod create request
       │
       ▼
MutatingAdmissionWebhook
  - ispeziona image/labels del Pod
  - se label "runtime=wasm" → aggiunge runtimeClassName: wasmtime-spin-v2
  - altrimenti → nessuna modifica
       │
       ▼
Pod schedulato con runtime corretto
```

**Vantaggi**: trasparente per chi scrive i manifest.
**Svantaggi**: richiede un webhook server custom (deployment separato), i certificati TLS per il webhook, e una convenzione di labeling. Più adatto a piattaforme multi-tenant.

### Approccio 2: Label-based RuntimeClass injection (semplificato)

Assegnare una label standard al Pod e usare un controller/webhook leggero che la legge:

```yaml
# Basta aggiungere questa label al Deployment
metadata:
  labels:
    runtime.k8s.io/type: wasm
# e un webhook inietta automaticamente runtimeClassName: wasmtime-spin-v2
```

**Per la tesi**: entrambi gli approcci sono "future work" rispetto all'approccio statico con `SpinApp`. La scelta statica via CRD è standard, supportata, e sufficiente per dimostrare la coesistenza dei runtime.

---

## Stato attuale del cluster K3s

Verifica effettuata al 2026-06-04:

```
RuntimeClass installate (K3s Addon "runtimes"):
  crun, lunatic, nvidia, nvidia-experimental, slight, spin,
  wasmedge, wasmer, wasmtime

Problema rilevato: le RuntimeClass sono "orfane"
  - Il containerd config.toml di K3s definisce SOLO runc come handler
  - Il binario containerd-shim-spin-v2 NON è presente sul nodo
  - Se un Pod richiede runtimeClassName: spin → errore di schedulazione

spin-operator: NON installato
cert-manager: non verificato
```

Questo significa che il cluster ha la **struttura K8s** per i runtime multipli (le RuntimeClass), ma manca lo **strato di sistema** (binari shim + config containerd). Va risolto prima di poter deployare workload Wasm.

---

## POC: far girare Docker e Wasm sullo stesso cluster

### Pre-requisiti

- K3s up e raggiungibile (`kubectl get nodes`)
- `helm` installato
- Immagine Wasm disponibile: `ghcr.io/antoniodev0/hello-wasm:v1` (da MasterThesis)

---

### Step 1 — Installare containerd-shim-spin sul nodo K3s

**Opzione A (consigliata) — KWasm operator**

KWasm è un operator K8s che installa automaticamente i binari degli shim Wasm sui nodi via DaemonSet privilegiato. Gestisce anche il template `config.toml` di K3s.

```bash
# 1. Annotare i nodi dove installare gli shim
kubectl annotate node --all kwasm.sh/kwasm-node=true

# 2. Installare l'operator
helm repo add kwasm http://kwasm.sh/kwasm-operator/
helm install kwasm-operator kwasm/kwasm-operator \
  --namespace kwasm \
  --create-namespace \
  --set kwasmOperator.installerImage=ghcr.io/kwasm/kwasm-node-installer:main

# 3. Attendere che l'installer DaemonSet completi
kubectl -n kwasm wait --for=condition=Ready pod -l app=kwasm-node-installer \
  --timeout=120s

# 4. Verifica: il binario è sul nodo?
which containerd-shim-spin-v2 || ls /opt/kwasm/bin/ | grep spin

# 5. Verifica: containerd è configurato?
sudo grep -A2 "runtimes.spin" \
  /var/lib/rancher/k3s/agent/etc/containerd/config.toml
```

**Opzione B — Manuale**

```bash
# Scaricare e installare il binario
VERSION=v0.15.1
curl -L \
  https://github.com/spinkube/containerd-shim-spin/releases/download/${VERSION}/containerd-shim-spin-v2-linux-x86_64.tar.gz \
  -o /tmp/spin-shim.tar.gz
sudo tar -xzf /tmp/spin-shim.tar.gz -C /usr/local/bin/
sudo chmod +x /usr/local/bin/containerd-shim-spin-v2

# Backup del config attuale e creazione del template
sudo cp /var/lib/rancher/k3s/agent/etc/containerd/config.toml \
        /var/lib/rancher/k3s/agent/etc/containerd/config.toml.bak

sudo tee -a /var/lib/rancher/k3s/agent/etc/containerd/config.toml.tmpl > /dev/null <<'EOF'

[plugins."io.containerd.grpc.v1.cri".containerd.runtimes.spin]
  runtime_type = "io.containerd.spin.v2"
EOF

# Riavvio K3s per applicare il template (~5-10s downtime)
sudo systemctl restart k3s

# Verifica
sudo grep -A2 "runtimes.spin" \
  /var/lib/rancher/k3s/agent/etc/containerd/config.toml
```

---

### Step 2 — Installare spin-operator

```bash
# cert-manager (prerequisito per i webhook del spin-operator)
kubectl apply -f \
  https://github.com/cert-manager/cert-manager/releases/download/v1.14.3/cert-manager.yaml

kubectl wait --for=condition=Available --timeout=180s \
  -n cert-manager deployment/cert-manager-webhook

# CRD SpinKube (SpinApp, SpinAppExecutor)
kubectl apply -f \
  https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.crds.yaml

# RuntimeClass wasmtime-spin-v2
kubectl apply -f \
  https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.runtime-class.yaml

# Operator via Helm (OCI registry)
helm install spin-operator \
  --namespace spin-operator \
  --create-namespace \
  --version 0.3.0 \
  --wait \
  oci://ghcr.io/spinkube/charts/spin-operator

# SpinAppExecutor: collega il CRD al binario shim fisico
kubectl apply -f \
  https://github.com/spinkube/spin-operator/releases/download/v0.3.0/spin-operator.shim-executor.yaml

# Verifica
kubectl -n spin-operator get pods
kubectl get runtimeclass wasmtime-spin-v2 -o jsonpath='{.handler}{"\n"}'
# atteso: spin
```

---

### Step 3 — Deploy manifest POC

I manifest si trovano in `retrospect/k8s/poc-mixed-runtime/`.

```bash
kubectl apply -f retrospect/k8s/poc-mixed-runtime/
```

Struttura:

```
retrospect/k8s/poc-mixed-runtime/
├── namespace.yaml           # namespace poc-runtime
├── wasm-spinapp.yaml        # workload Wasm via SpinApp
└── docker-deployment.yaml  # workload Docker (nginx) via Deployment standard
```

**`wasm-spinapp.yaml`**:
```yaml
apiVersion: core.spinoperator.dev/v1alpha1
kind: SpinApp
metadata:
  name: hello-wasm
  namespace: poc-runtime
spec:
  image: "ghcr.io/antoniodev0/hello-wasm:v1"
  executor: containerd-shim-spin
  replicas: 1
```

**`docker-deployment.yaml`**: Deployment nginx senza `runtimeClassName` — kubelet usa il default `runc`.

---

### Step 4 — Verifica

```bash
# Entrambi i pod in Running?
kubectl -n poc-runtime get pods -o wide

# Runtime del pod Wasm (deve essere wasmtime-spin-v2)
kubectl -n poc-runtime get pod \
  -l core.spinoperator.dev/app-name=hello-wasm \
  -o jsonpath='{.items[0].spec.runtimeClassName}{"\n"}'

# Runtime del pod Docker (deve essere vuoto → runc default)
kubectl -n poc-runtime get pod -l app=hello-docker \
  -o jsonpath='{.items[0].spec.runtimeClassName}{"\n"}'

# Prova fisica: processi shim attivi sul nodo
ps -ef | grep "containerd-shim" | grep -v grep
# atteso: containerd-shim-spin-v2 PER il pod Wasm
#         containerd-shim-runc-v2 PER il pod Docker

# Risposta HTTP dai due workload
kubectl -n poc-runtime port-forward svc/hello-wasm 8080:80 &
kubectl -n poc-runtime port-forward svc/hello-docker 8090:80 &
curl localhost:8080   # output del componente Wasm
curl localhost:8090   # welcome page nginx
```

---

## Note operative

**RuntimeClass orfane (Addon K3s)**: il cluster ha già 8 RuntimeClass installate dall'Addon K3s `runtimes` (spin, wasmtime, wasmedge, ecc.) ma senza corrispondenza nel config containerd. Non interferiscono con la POC, ma un Pod che le usa per errore va in `ContainerCreating` → timeout. Si possono eliminare dopo aver verificato che spin-operator non le usa:

```bash
# Lista RuntimeClass orfane (opzionale: eliminare dopo POC verde)
kubectl get runtimeclass
```

**Immagine Wasm `hello-wasm:v1`**: pubblicata su GHCR da MasterThesis. Se il tag non è più disponibile, rebuild:

```bash
cd /home/ubuntu/Thesis/MasterThesis/hello-wasm
spin build
spin registry push ghcr.io/antoniodev0/hello-wasm:v1
```

**Branch git**: siamo su `OCRE-Integration-Test`. I manifest in `retrospect/k8s/poc-mixed-runtime/` vanno committati su questo branch.
