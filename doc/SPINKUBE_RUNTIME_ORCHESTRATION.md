# SpinKube: orchestrazione mista Docker + Wasm su Kubernetes

## La domanda di ricerca

> SpinKube utilizza `containerd-shim-spin` per far girare WebAssembly. È possibile configurare lo spin-operator (o trovare una soluzione alternativa) per decidere **dinamicamente** se usare lo shim standard per i container Docker o lo shim di Spin per i workload Wasm? Come orientare l'orchestratore nella scelta del runtime corretto in base al tipo di carico di lavoro?

---

## Risposta

Lo spin-operator **non è progettato per scegliere dinamicamente** tra runtime diversi: è un controller mono-scopo che trasforma risorse `SpinApp` in `Deployment` configurati per lo shim Spin. La selezione del runtime in Kubernetes è una proprietà **statica del PodSpec** (`spec.runtimeClassName`), decisa al momento della creazione del workload, non da un'euristica a runtime.

Il meccanismo nativo Kubernetes che risolve il problema è la **RuntimeClass**: lo stesso scheduler, gli stessi nodi e lo stesso containerd possono eseguire contemporaneamente workload Docker (via `runc`) e workload Wasm (via `containerd-shim-spin`) in base al tipo di risorsa K8s utilizzata per descrivere il carico di lavoro.

| Tipo risorsa K8s | RuntimeClass applicata | Shim invocato |
|---|---|---|
| `SpinApp` (CRD SpinKube) | `wasmtime-spin-v2` | `containerd-shim-spin-v2` → Wasmtime |
| `Deployment` standard | *(nessuna, default)* | `containerd-shim-runc-v2` → runc/Linux |

**L'orchestratore viene orientato dalla struttura del manifest, non da un'euristica dinamica.**

---

## Architettura del meccanismo

### Flusso decisionale completo

```
Utente crea SpinApp              Utente crea Deployment
       │                                  │
       ▼                                  ▼
  spin-operator               kube-controller-manager
  (watch CRD SpinApp)         (watch Deployment)
       │                                  │
       ▼                                  │
  genera Deployment con        (già un Deployment)
  runtimeClassName:
    wasmtime-spin-v2
       │                                  │
       └──────────────┬───────────────────┘
                      ▼
              kube-scheduler
         (stesso scheduler per tutti i workload)
                      │
                      ▼
              kubelet sul nodo
                      │
          ┌───────────┴────────────────────┐
          │                                │
  runtimeClassName                nessun runtimeClassName
  = wasmtime-spin-v2               → usa default (runc)
  → handler: spin                          │
          │                                │
          ▼                                ▼
  containerd chiama              containerd chiama
  containerd-shim-spin-v2        containerd-shim-runc-v2
          │                                │
          ▼                                ▼
   Wasmtime esegue                runc crea container
   il modulo .wasm                 Linux tradizionale
          │                                │
          ▼                                ▼
  risposta HTTP dal              risposta HTTP dal
  componente Spin                 container nginx
```

### Catena di risoluzione del runtime (SpinKube)

```
SpinApp
  └─ executor: containerd-shim-spin
       └─ SpinAppExecutor (CRD)
            └─ runtimeClassName: wasmtime-spin-v2
                 └─ RuntimeClass: handler = "spin"
                      └─ containerd runtime config: spin → io.containerd.spin.v2
                           └─ binario: /usr/local/bin/containerd-shim-spin-v2
```

### Il ruolo di RuntimeClass

`RuntimeClass` è una risorsa Kubernetes (`node.k8s.io/v1`) che associa un nome logico a un handler containerd. Il kubelet legge `spec.runtimeClassName` dal PodSpec e lo usa per trovare la RuntimeClass corrispondente; il nome dell'handler viene passato a containerd per selezionare il binario shim.

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: wasmtime-spin-v2
handler: spin   # containerd cerca containerd-shim-spin-v2 nel PATH
```

La voce corrispondente nel config containerd (generata automaticamente da K3s):

```toml
[plugins.'io.containerd.cri.v1.runtime'.containerd.runtimes.'spin']
  runtime_type = "io.containerd.spin.v2"

[plugins.'io.containerd.cri.v1.runtime'.containerd.runtimes.'spin'.options]
  BinaryName = "/usr/local/bin/containerd-shim-spin-v2"
  SystemdCgroup = true
```

### Il ruolo di spin-operator

spin-operator è un Kubernetes controller che:
1. Osserva (`watch`) le risorse `SpinApp`
2. Per ogni `SpinApp`, genera un `Deployment` con `spec.runtimeClassName: wasmtime-spin-v2` e un `Service`
3. Gestisce scaling, aggiornamenti e status della SpinApp

Non contiene logica di selezione runtime: la scelta del runtime è determinata dal tipo di risorsa K8s creata dall'utente.

---

## Possibilità di selezione "dinamica" (future work)

Se si vuole che il sistema scelga autonomamente il runtime senza che l'utente distingua esplicitamente tra `SpinApp` e `Deployment`, esistono due approcci architetturali:

**MutatingAdmissionWebhook**: un webhook intercetta ogni `Pod` in creazione e inietta `spec.runtimeClassName: wasmtime-spin-v2` in base a label o contenuto dell'immagine OCI. Trasparente per l'utente ma richiede un webhook server custom con certificati TLS.

**Label-based injection**: convenzione di labeling (`runtime.k8s.io/type: wasm`) letta da un controller leggero che patcha il PodSpec prima della schedulazione.

Per la tesi, entrambi sono "future work": l'approccio statico via CRD è standard, affidabile e sufficiente a dimostrare la coesistenza dei runtime.

---

## Implementazione e test (2026-06-04)

### Ambiente di test

| Componente | Versione |
|---|---|
| K3s | v1.35.4+k3s1 |
| containerd | v2.2.3-k3s1 (config formato v3) |
| containerd-shim-spin-v2 | v0.24.0 |
| spin-operator | v0.6.1 (org `spinframework`) |
| cert-manager | v1.20.0 |
| Helm | v3.21.0 |
| Immagine Wasm | `ghcr.io/antoniodev0/hello-wasm:v1` (HTTP trigger Spin v2) |
| Immagine Docker | `nginx:1.27-alpine` |

### Nota su K3s e auto-detection dello shim

K3s v1.31.6+ con containerd 2.x **auto-rileva** i binari shim nel PATH al boot e genera automaticamente la runtime stanza nel config containerd v3. Non è necessario modificare manualmente il config o usare KWasm (che risulta non mantenuto dal 2024 e incompatibile con containerd 2.x).

**ATTENZIONE**: KWasm scrive l'obsoleto path `io.containerd.grpc.v1.cri` (formato v1/v2) che causa il crash di containerd 2.x. Da evitare su K3s 1.31+.

---

### Installazione passo per passo

#### 1. Installare containerd-shim-spin-v2

```bash
VERSION=v0.24.0
curl -L \
  https://github.com/spinframework/containerd-shim-spin/releases/download/${VERSION}/containerd-shim-spin-v2-linux-x86_64.tar.gz \
  -o /tmp/spin-shim.tar.gz
sudo tar -xzf /tmp/spin-shim.tar.gz -C /usr/local/bin/
sudo chmod +x /usr/local/bin/containerd-shim-spin-v2

# Backup config containerd
sudo cp /var/lib/rancher/k3s/agent/etc/containerd/config.toml \
        /var/lib/rancher/k3s/agent/etc/containerd/config.toml.pre-spin.bak

# Restart K3s → auto-detection del binario spin
sudo systemctl restart k3s
sleep 25

# Verifica: la stanza spin è stata auto-generata
sudo grep -A4 "runtimes.'spin'" /var/lib/rancher/k3s/agent/etc/containerd/config.toml
```

Output atteso dopo il restart:
```toml
[plugins.'io.containerd.cri.v1.runtime'.containerd.runtimes.'spin']
  runtime_type = "io.containerd.spin.v2"

[plugins.'io.containerd.cri.v1.runtime'.containerd.runtimes.'spin'.options]
  BinaryName = "/usr/local/bin/containerd-shim-spin-v2"
  SystemdCgroup = true
```

#### 2. Installare Helm

```bash
curl -fsSL -o /tmp/get_helm.sh https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3
chmod 700 /tmp/get_helm.sh
sudo /tmp/get_helm.sh
```

#### 3. Installare cert-manager

```bash
kubectl apply -f \
  https://github.com/cert-manager/cert-manager/releases/download/v1.20.0/cert-manager.yaml

kubectl wait --for=condition=Available --timeout=300s \
  -n cert-manager deployment/cert-manager-webhook
```

#### 4. Installare spin-operator v0.6.1

```bash
# CRD: SpinApp, SpinAppExecutor
kubectl apply -f \
  https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.crds.yaml

# RuntimeClass wasmtime-spin-v2 (handler: spin)
kubectl apply -f \
  https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.runtime-class.yaml

# Operator via Helm OCI
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
helm upgrade --install spin-operator \
  --namespace spin-operator \
  --create-namespace \
  --version 0.6.1 \
  --wait \
  oci://ghcr.io/spinframework/charts/spin-operator

# SpinAppExecutor: mappa logica → shim fisico
kubectl apply -f \
  https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.shim-executor.yaml
```

#### 5. Deploy workload misti

I manifest si trovano in `retrospect/k8s/poc-mixed-runtime/`.

**`namespace.yaml`**:
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: poc-runtime
```

**`wasm-spinapp.yaml`** — workload WebAssembly:
```yaml
apiVersion: core.spinkube.dev/v1alpha1
kind: SpinApp
metadata:
  name: hello-wasm
  namespace: poc-runtime
spec:
  image: "ghcr.io/antoniodev0/hello-wasm:v1"
  executor: containerd-shim-spin
  replicas: 1
```

**`docker-deployment.yaml`** — workload Docker tradizionale:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hello-docker
  namespace: poc-runtime
spec:
  replicas: 1
  selector:
    matchLabels: { app: hello-docker }
  template:
    metadata:
      labels: { app: hello-docker }
    spec:
      # Nessun runtimeClassName → kubelet usa il default (runc)
      containers:
        - name: nginx
          image: nginx:1.27-alpine
          ports: [{ containerPort: 80 }]
---
apiVersion: v1
kind: Service
metadata:
  name: hello-docker
  namespace: poc-runtime
spec:
  selector: { app: hello-docker }
  ports: [{ port: 80, targetPort: 80 }]
```

```bash
# Deploy (il SpinAppExecutor deve esistere nel namespace poc-runtime)
kubectl get spinappexecutor containerd-shim-spin -o yaml | \
  sed 's/namespace: default/namespace: poc-runtime/' | kubectl apply -f -

kubectl apply -f retrospect/k8s/poc-mixed-runtime/
```

---

### Risultati dei test

#### Pod in esecuzione

```
NAMESPACE     NAME                              READY   STATUS    RESTARTS
poc-runtime   hello-docker-68b658b85c-6srfq     1/1     Running   0
poc-runtime   hello-wasm-5957bcb8b5-ntpjw       1/1     Running   0
```

#### RuntimeClass assegnate

```bash
# Pod Wasm
kubectl -n poc-runtime get pod hello-wasm-5957bcb8b5-ntpjw \
  -o jsonpath='{.spec.runtimeClassName}'
# → wasmtime-spin-v2

# Pod Docker
kubectl -n poc-runtime get pod hello-docker-68b658b85c-6srfq \
  -o jsonpath='{.spec.runtimeClassName}'
# → (vuoto, usa runc di default)
```

#### Processi shim attivi sul nodo (prova fisica)

```bash
ps -ef | grep "containerd-shim" | grep "k8s.io" | grep -v grep
```

Output significativo (estratto):
```
# Shim Wasm — avviato da containerd per il pod hello-wasm
root  2611549  1  /usr/local/bin/containerd-shim-spin-v2
        -namespace k8s.io
        -id 72de3f0fa9858630b16ef73cec89ba860257b1242081d1b418df38f36b06f3e7

# Shim Linux — avviato da containerd per i pod Docker/runc
root  2611074  1  /var/lib/rancher/k3s/data/.../containerd-shim-runc-v2
        -namespace k8s.io
        -id 28029dafcc2905ae2f103927b729ef191f39a75979a9e60ba42cd5f9f8b5311c
```

I due shim sono processi distinti sullo stesso nodo, con lo stesso `containerd.sock` ma ID container diversi.

#### Risposte HTTP

```bash
kubectl -n poc-runtime port-forward svc/hello-wasm 8080:80 &
kubectl -n poc-runtime port-forward svc/hello-docker 8090:80 &
sleep 3

curl localhost:8080
# Hello World!

curl localhost:8090
# <!DOCTYPE html><html><head><title>Welcome to nginx!</title>...
```

---

### Riepilogo dei risultati

| Verifica | Risultato |
|---|---|
| `containerd-shim-spin-v2` installato e rilevato da K3s | ✅ `/usr/local/bin/containerd-shim-spin-v2` |
| Runtime stanza `spin` auto-generata nel config containerd | ✅ `runtime_type = "io.containerd.spin.v2"` |
| RuntimeClass `wasmtime-spin-v2` creata (handler: spin) | ✅ |
| spin-operator Running | ✅ `spin-operator-controller-manager` Running |
| Pod `hello-wasm` Running con `runtimeClassName: wasmtime-spin-v2` | ✅ |
| Pod `hello-docker` Running senza `runtimeClassName` (→ runc) | ✅ |
| Shim `containerd-shim-spin-v2` visibile in `ps` per il pod Wasm | ✅ PID 2611549 |
| Shim `containerd-shim-runc-v2` visibile in `ps` per il pod Docker | ✅ |
| Risposta HTTP Wasm: `Hello World!` | ✅ |
| Risposta HTTP Docker: pagina nginx | ✅ |

---

## Struttura file nel repository

```
retrospect/
├── doc/
│   └── SPINKUBE_RUNTIME_ORCHESTRATION.md   ← questo documento
└── k8s/
    └── poc-mixed-runtime/
        ├── namespace.yaml          # Namespace poc-runtime
        ├── wasm-spinapp.yaml       # Workload Wasm (SpinApp → wasmtime-spin-v2)
        └── docker-deployment.yaml  # Workload Docker (Deployment → runc)
```

---

## Per riprodurre l'ambiente

Prerequisiti di sistema da installare una volta sola:

```bash
# 1. containerd-shim-spin-v2 v0.24.0
VERSION=v0.24.0
curl -L https://github.com/spinframework/containerd-shim-spin/releases/download/${VERSION}/containerd-shim-spin-v2-linux-x86_64.tar.gz \
  -o /tmp/spin-shim.tar.gz
sudo tar -xzf /tmp/spin-shim.tar.gz -C /usr/local/bin/
sudo systemctl restart k3s && sleep 25

# 2. Helm
curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | sudo bash

# 3. cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.20.0/cert-manager.yaml
kubectl wait --for=condition=Available --timeout=300s -n cert-manager deployment/cert-manager-webhook

# 4. spin-operator
kubectl apply -f https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.crds.yaml
kubectl apply -f https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.runtime-class.yaml
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
helm upgrade --install spin-operator --namespace spin-operator --create-namespace \
  --version 0.6.1 --wait oci://ghcr.io/spinframework/charts/spin-operator
kubectl apply -f https://github.com/spinframework/spin-operator/releases/download/v0.6.1/spin-operator.shim-executor.yaml
```

Deploy dei workload:

```bash
# SpinAppExecutor nel namespace della POC
kubectl get spinappexecutor containerd-shim-spin -o yaml | \
  sed 's/namespace: default/namespace: poc-runtime/' | kubectl apply -f -

kubectl apply -f retrospect/k8s/poc-mixed-runtime/

# Verifica
kubectl -n poc-runtime get pods
kubectl -n poc-runtime port-forward svc/hello-wasm 8080:80 &
kubectl -n poc-runtime port-forward svc/hello-docker 8090:80 &
curl localhost:8080   # → Hello World! (Wasm)
curl localhost:8090   # → nginx (Docker)
```
