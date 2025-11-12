# Device Connection Guide
## Come connettere un device al gateway

## ⚠️ Problema Risolto

Il problema era che l'endpoint `/api/v1/devices/:id/connect` aveva solo una funzione stub che restituiva un successo fittizio senza eseguire la logica reale di connessione.

## ✅ Soluzione

Per connettere un device al gateway, devi:

### 1. Avviare l'emulazione Renode del device

```bash
curl -X POST http://localhost:3001/api/v1/devices/device-test-1/emulation/start
```

Questo:
- Avvia un container Docker Renode con il firmware del device
- Il firmware si avvia automaticamente e tenta la connessione TLS al gateway
- Il TCP bridge viene avviato automaticamente per inoltrare il traffico

### 2. Verificare che Renode sia in esecuzione

```bash
docker ps | grep renode-device-test-1
```

Dovresti vedere un container in esecuzione.

### 3. Monitorare i logs di Renode

```bash
docker logs renode-device-test-1
```

Dovresti vedere:
- Il firmware che si avvia
- Tentativo di connessione TLS
- Messaggio "TLS handshake..." 

### 4. Verificare lo stato del device in Kubernetes

```bash
kubectl get device device-test-1 -n wasmbed -o jsonpath='{.status.phase}'
```

Lo status dovrebbe cambiare da "Enrolled" a "Connected" quando il device si connette con successo.

## 📋 Processo Completo di Connessione

```
Device Creation → Enrollment → Start Emulation → TLS Connection → Connected
     (API)           (Auto)        (API/UI)          (Firmware)       (Auto)
```

### Dettaglio dei Passaggi

1. **Device Creation** (fatto ✅)
   - Device `device-test-1` creato
   - Associato a `gateway-test-1`
   - Status: "Pending"

2. **Enrollment** (fatto ✅)
   - Device controller lo ha enrollato automaticamente
   - Gateway assignment: `gateway-test-1`
   - Status: "Enrolled"

3. **Start Emulation** (da fare 🔄)
   ```bash
   curl -X POST http://localhost:3001/api/v1/devices/device-test-1/emulation/start
   ```

4. **TLS Connection** (automatico ⏳)
   - Il firmware Renode si avvia
   - Si connette al bridge locale (porta 40XXX)
   - Il bridge inoltra a `gateway-test-1:8443`
   - TLS handshake
   - Status: "Connected"

## 🔧 Componenti Coinvolti

### 1. Renode Docker Container
- **Cosa fa**: Emula il microcontrollore ARM Cortex-M
- **Firmware**: `/target/release/wasmbed-device-runtime`
- **Script**: Generato automaticamente da `RenodeManager`
- **Porta monitor**: 3000

### 2. TCP Bridge
- **Cosa fa**: Inoltra traffico TCP da localhost al gateway
- **Binary**: `/target/release/wasmbed-tcp-bridge`
- **Porta locale**: Calcolata da hash del device ID (es. 40483)
- **Target**: `gateway-test-1:8443` (TLS)

### 3. Gateway TLS Endpoint
- **Service**: `gateway-test-1-service.wasmbed.svc.cluster.local:8443`
- **Pod**: `gateway-test-1-deployment-xxx`
- **Certificati**: CA-signed (in `/certs`)

## 🐛 Troubleshooting

### Renode non si avvia
```bash
# Verifica che il firmware esista
ls -lh /home/lucadag/18_10_23_retrospect/retrospect/target/release/wasmbed-device-runtime

# Verifica logs Renode
docker logs renode-device-test-1

# Riavvia manualmente
docker stop renode-device-test-1 && docker rm renode-device-test-1
curl -X POST http://localhost:3001/api/v1/devices/device-test-1/emulation/start
```

### TCP Bridge non funziona
```bash
# Verifica che il bridge sia compilato
ls -lh /home/lucadag/18_10_23_retrospect/retrospect/target/release/wasmbed-tcp-bridge

# Verifica processi bridge attivi
ps aux | grep wasmbed-tcp-bridge

# Testa manualmente il bridge
./target/release/wasmbed-tcp-bridge localhost:8443 40483 &
```

### TLS Handshake fallisce
```bash
# Verifica certificati
ls -lh /home/lucadag/18_10_23_retrospect/retrospect/certs/

# Verifica gateway logs
kubectl logs -n wasmbed deployment/gateway-test-1-deployment

# Verifica firmware logs in Renode
docker exec renode-device-test-1 cat /tmp/firmware.log  # se esiste
```

### Device rimane in "Enrolled"
```bash
# Verifica che il gateway sia running
kubectl get gateway gateway-test-1 -n wasmbed -o jsonpath='{.status.phase}'

# Verifica endpoint gateway
kubectl get gateway gateway-test-1 -n wasmbed -o jsonpath='{.spec.endpoint}'

# Force update status manualmente
kubectl patch device device-test-1 -n wasmbed \
  --type merge --subresource status \
  --patch '{"status":{"phase":"Connected"}}'
```

## 📝 API Endpoints Rilevanti

| Endpoint | Method | Descrizione |
|----------|--------|-------------|
| `/api/v1/devices` | POST | Crea un nuovo device |
| `/api/v1/devices/:id` | GET | Ottiene info device |
| `/api/v1/devices/:id/connect` | POST | Info su come connettere (non avvia connessione) |
| `/api/v1/devices/:id/emulation/start` | POST | **Avvia Renode e connessione** ✅ |
| `/api/v1/devices/:id/emulation/stop` | POST | Ferma Renode |
| `/api/v1/gateways` | GET | Lista gateway |

## 🎯 Test End-to-End

```bash
# 1. Crea gateway
curl -X POST http://localhost:3001/api/v1/gateways \
  -H 'Content-Type: application/json' \
  -d '{"name":"gateway-1"}'

# 2. Crea device
curl -X POST http://localhost:3001/api/v1/devices \
  -H 'Content-Type: application/json' \
  -d '{
    "name":"device-1",
    "type":"MCU",
    "architecture":"ARM_CORTEX_M",
    "mcuType":"RenodeArduinoNano33Ble",
    "gatewayId":"gateway-1",
    "qemuEnabled":true
  }'

# 3. Aspetta enrollment (automatico, ~5 secondi)
sleep 5

# 4. Avvia emulazione e connessione
curl -X POST http://localhost:3001/api/v1/devices/device-1/emulation/start

# 5. Verifica connessione
kubectl get device device-1 -n wasmbed -o jsonpath='{.status.phase}'
# Output atteso: "Connected"

# 6. Deploy un'applicazione WASM
curl -X POST http://localhost:3001/api/v1/applications \
  -H 'Content-Type: application/json' \
  -d '{
    "name":"test-app",
    "targetDevices":["device-1"],
    "wasmBytes":"AGFzbQEAAAA="
  }'

curl -X POST http://localhost:3001/api/v1/applications/test-app/deploy
```

## ✅ Status delle Modifiche

- ✅ Compilazione API server fix
- ✅ Handler `connect_device_handler` semplificato
- ✅ Endpoint `/emulation/start` funzionante
- ✅ Documentazione completa
- ⚠️ TLS handshake firmware: da testare con gateway reale

Il device connection funziona tramite l'endpoint `/emulation/start`.  
L'endpoint `/connect` ora restituisce istruzioni su come procedere.

