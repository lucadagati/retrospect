# Wasmbed Deployment Scripts

Scripts per il deployment di Wasmbed su Kubernetes K3S.

## Script Disponibili

### `deploy-k3s.sh`
Deploy completo di Wasmbed su K3S.

**Prerequisiti:**
- K3S installato e configurato
- Docker installato
- kubectl configurato

**Installazione K3S:**
```bash
curl -sfL https://get.k3s.io | sh -s - --write-kubeconfig-mode 644
mkdir -p ~/.kube
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $(id -u):$(id -g) ~/.kube/config
```

**Utilizzo:**
```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
./scripts/deploy-k3s.sh
```

**Cosa fa:**
1. Verifica K3S e Docker
2. Avvia registry locale su `localhost:5000`
3. Builda e pusha tutte le immagini Docker
4. Crea namespace `wasmbed`
5. Applica CRDs (Device, Application, Gateway)
6. Configura RBAC
7. Genera certificati TLS per il gateway
8. Deploya tutti i servizi
9. Crea Gateway CRD iniziale

**Output:**
- 7 pods running in namespace `wasmbed`
- Registry locale con 6 immagini
- Gateway configurato
- Dashboard accessibile

### `cleanup-k3s.sh`
Rimozione completa del deployment Wasmbed.

**Utilizzo:**
```bash
./scripts/cleanup-k3s.sh
```

**Cosa fa:**
1. Termina port-forwards attivi
2. Ferma e rimuove containers Renode
3. Rimuove volumi Docker
4. Elimina namespace `wasmbed` (e tutte le risorse)
5. Opzionalmente ferma il registry locale

**Attenzione:** Questa operazione è irreversibile!

## Componenti Deployati

### Core Services
- **wasmbed-api-server**: API REST principale (porta 3001)
- **wasmbed-gateway**: Gateway per connessioni device (porte 8080 HTTP, 8081 TLS)
- **wasmbed-dashboard**: Dashboard web (porta 3000)

### Controllers
- **wasmbed-device-controller**: Gestione lifecycle devices
- **wasmbed-application-controller**: Gestione applicazioni WASM
- **wasmbed-gateway-controller**: Gestione gateways

### Custom Resource Definitions (CRDs)
- **Device**: `devices.wasmbed.github.io`
- **Application**: `applications.wasmbed.github.io`
- **Gateway**: `gateways.wasmbed.io`

## MCU Types Supportati

### Con Ethernet (Raccomandati)
- **Stm32F746gDisco** - STM32F746G Discovery (default)
- **FrdmK64f** - NXP FRDM-K64F

### Con WiFi
- **Esp32DevkitC** - ESP32 DevKitC

### Senza Network (Sviluppo)
- **Stm32F4Disco** - STM32F4 Discovery
- **Nrf52840DK** - Nordic nRF52840 DK

## Endpoints

Dopo il deployment:

**Dashboard:**
```bash
# Il LoadBalancer espone la dashboard sull'IP del nodo
http://<NODE_IP>:3000
```

**API Server:**
```bash
# Port-forward per accedere all'API
kubectl port-forward -n wasmbed svc/wasmbed-api-server 3000:3001
```

**Registry:**
```bash
# Registry locale
http://localhost:5000/v2/_catalog
```

## Test Rapido

### 1. Enrollment Device
```bash
kubectl port-forward -n wasmbed svc/wasmbed-api-server 3000:3001 &

curl -X POST http://localhost:3000/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "name":"test-stm32f7",
    "deviceType":"MCU",
    "mcuType":"Stm32F746gDisco",
    "gatewayId":"gateway-1"
  }'
```

### 2. Avvio Renode
```bash
curl -X POST http://localhost:3000/api/v1/devices/test-stm32f7/renode/start
```

### 3. Verifica
```bash
# Pods
kubectl get pods -n wasmbed

# Devices
kubectl get devices.wasmbed.github.io -n wasmbed

# Gateways
kubectl get gateways.wasmbed.io -n wasmbed

# Renode containers
docker ps --filter "name=renode-"
```

## Troubleshooting

### Pods non partono
```bash
kubectl describe pod -n wasmbed <pod-name>
kubectl logs -n wasmbed <pod-name>
```

### Immagini non trovate
```bash
# Verifica registry
curl http://localhost:5000/v2/_catalog

# Rebuild e push
cd /home/lucadag/18_10_23_retrospect/retrospect
docker build -t localhost:5000/wasmbed/api-server:latest -f Dockerfile.api-server .
docker push localhost:5000/wasmbed/api-server:latest
```

### Gateway non risponde
```bash
# Verifica certificati
kubectl get secret gateway-certificates -n wasmbed

# Logs gateway
kubectl logs -n wasmbed -l app=wasmbed-gateway
```

### Renode non parte
```bash
# Verifica firmware nell'immagine
docker run --rm localhost:5000/wasmbed/api-server:latest \
  ls -la /app/zephyr-workspace/build/*/zephyr/zephyr.elf

# Logs API server
kubectl logs -n wasmbed -l app=wasmbed-api-server --tail=50
```

## Architettura

```
┌─────────────────────────────────────────────────────────┐
│                    K3S Cluster                          │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Dashboard  │  │  API Server  │  │   Gateway    │ │
│  │   (React)    │  │   (Rust)     │  │   (Rust)     │ │
│  │   :3000      │  │   :3001      │  │ :8080/:8081  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Device     │  │ Application  │  │   Gateway    │ │
│  │ Controller   │  │ Controller   │  │ Controller   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
                         │
                         │ Docker Socket
                         ▼
          ┌────────────────────────────┐
          │   Renode Containers        │
          │  (Device Emulation)        │
          └────────────────────────────┘
```

## Note

- Il sistema richiede accesso al Docker socket (`/var/run/docker.sock`) per gestire containers Renode
- Le immagini sono buildare localmente e pushate a `localhost:5000`
- I certificati TLS sono auto-firmati e validi per 365 giorni
- Il namespace `wasmbed` contiene tutte le risorse
- Flannel CNI è usato per il networking (default K3S)

## Riferimenti

- **Repository**: `/home/lucadag/18_10_23_retrospect/retrospect`
- **Documentazione**: `./doc/`
- **CRDs**: `./k8s/crds/`
- **Deployments**: `./k8s/deployments/`
- **RBAC**: `./k8s/rbac/`
