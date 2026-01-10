# Deployment Guide

## Prerequisiti

### Software Richiesto

- **Kubernetes**: Cluster Kubernetes 1.24+
- **Docker**: Per build e containerizzazione
- **kubectl**: Configurato per accedere al cluster
- **Rust**: Toolchain 1.88.0+
- **Zephyr SDK**: Per compilazione firmware (opzionale per deployment)

### Hardware/Emulazione

- **Renode**: Per emulazione dispositivi (via Docker)
- **Risorse cluster**: Minimo 4 CPU, 8GB RAM

## Setup Iniziale

### 1. Clone Repository

```bash
git clone <repository-url>
cd retrospect
```

### 2. Setup Zephyr Workspace

```bash
./scripts/setup-zephyr-workspace.sh
```

Questo script:
- Clona Zephyr RTOS
- Configura ambiente
- Setup WAMR (se non presente)

### 3. Build Componenti

```bash
./scripts/02-build-components.sh
```

Build di tutti i componenti Rust:
- API Server
- Gateway
- Controllers
- Utilities

### 4. Build Firmware (Opzionale)

```bash
./scripts/build-zephyr-app.sh
```

Compila il firmware Zephyr per le piattaforme supportate.

## Deployment Kubernetes

### 1. Deploy Namespace

```bash
kubectl apply -f k8s/namespace.yaml
```

### 2. Deploy CRDs

```bash
kubectl apply -f k8s/crds/
```

Custom Resource Definitions:
- Device
- Application
- Gateway

### 3. Deploy RBAC

```bash
kubectl apply -f k8s/rbac/
```

Permessi per controllers.

### 4. Deploy Components

```bash
./scripts/03-deploy-infrastructure.sh
```

Deploy di:
- API Server
- Gateway
- Controllers
- Dashboard

### 5. Verifica Deployment

```bash
./scripts/04-check-system-status.sh
```

Verifica stato di tutti i componenti.

## Configurazione

### Config File

File principale: `config/wasmbed-config.yaml`

```yaml
api:
  bind_addr: "0.0.0.0:8080"
  
gateway:
  bind_addr: "0.0.0.0:40029"
  http_addr: "0.0.0.0:8080"
  
tls:
  ca_cert: "certs/ca-cert.pem"
  server_cert: "certs/server-cert.pem"
  server_key: "certs/server-key.pem"
```

### Environment Variables

Componenti principali utilizzano variabili d'ambiente:

**API Server:**
- `WASMBED_API_BIND_ADDR`
- `WASMBED_API_NAMESPACE`

**Gateway:**
- `WASMBED_GATEWAY_BIND_ADDR`
- `WASMBED_GATEWAY_PRIVATE_KEY`
- `WASMBED_GATEWAY_CERTIFICATE`
- `WASMBED_GATEWAY_CLIENT_CA`

## Accesso Servizi

### Dashboard

```bash
kubectl port-forward -n wasmbed svc/dashboard 3000:3000
```

Accesso: http://localhost:3000

### API Server

```bash
kubectl port-forward -n wasmbed svc/api-server 8080:8080
```

API: http://localhost:8080

### Gateway

```bash
kubectl port-forward -n wasmbed svc/gateway 40029:40029
```

TLS endpoint per dispositivi.

## Gestione Dispositivi

### Creare Device

Via Dashboard:
1. Navigare a "Devices"
2. Click "Create Device"
3. Inserire nome e tipo MCU
4. Submit

Via API:
```bash
curl -X POST http://localhost:8080/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-device",
    "mcu_type": "RenodeArduinoNano33Ble"
  }'
```

### Verificare Status

```bash
kubectl get devices -n wasmbed
```

### Log Dispositivo

```bash
kubectl logs -n wasmbed -l app=renode-sidecar
```

## Deployment Applicazioni

### Creare Application

Via Dashboard:
1. Navigare a "Applications"
2. Click "Create Application"
3. Upload file WASM
4. Configurare parametri
5. Submit

Via API:
```bash
curl -X POST http://localhost:8080/api/v1/applications \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-app",
    "wasm_module": "<base64-encoded-wasm>"
  }'
```

### Deploy su Device

Via Dashboard:
1. Selezionare Application
2. Click "Deploy"
3. Selezionare Device target
4. Submit

## Monitoring

### Status Components

```bash
kubectl get pods -n wasmbed
kubectl get svc -n wasmbed
```

### Logs

```bash
# API Server
kubectl logs -n wasmbed -l app=api-server

# Gateway
kubectl logs -n wasmbed -l app=gateway

# Controllers
kubectl logs -n wasmbed -l app=device-controller
kubectl logs -n wasmbed -l app=application-controller
```

### Metrics

Dashboard mostra:
- Device status
- Application execution
- Network connectivity
- Resource usage

## Troubleshooting

### Pods non si avviano

```bash
kubectl describe pod <pod-name> -n wasmbed
kubectl logs <pod-name> -n wasmbed
```

### Device non si connette

1. Verificare TCP bridge attivo
2. Controllare certificati TLS
3. Verificare endpoint gateway
4. Controllare log firmware

### Application non si esegue

1. Verificare formato WASM
2. Controllare log WAMR
3. Verificare memoria disponibile
4. Controllare log dispositivo

### Network Issues

1. Verificare configurazione network policy
2. Controllare service endpoints
3. Verificare DNS resolution

## Scaling

### Gateway Scaling

```bash
kubectl scale deployment gateway -n wasmbed --replicas=3
```

### HPA Configuration

File: `k8s/gateway-hpa.yaml`

```bash
kubectl apply -f k8s/gateway-hpa.yaml
```

## Cleanup

### Stop Services

```bash
./scripts/05-stop-services.sh
```

### Remove All

```bash
kubectl delete namespace wasmbed
```

## Production Considerations

### Security

- Usare certificati validi
- Configurare network policies
- Abilitare RBAC completo
- Usare secrets per credenziali

### Performance

- Configurare resource limits
- Usare HPA per auto-scaling
- Monitorare metriche
- Ottimizzare configurazione

### Backup

- Backup CRDs (Device, Application)
- Backup configurazione
- Backup certificati
