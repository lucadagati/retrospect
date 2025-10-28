# ROS 2 Integration for Wasmbed

Questo documento descrive l'integrazione di ROS 2 nel sistema Wasmbed, che permette alle applicazioni WebAssembly di comunicare con sistemi ROS 2 attraverso microROS.

## 🏗️ Architettura

### Componenti Principali

1. **microROS Agent**: Servizio ROS 2 che gestisce la comunicazione con dispositivi embedded
2. **Wasmbed microROS Bridge**: Bridge che connette il WASM runtime con ROS 2
3. **ROS 2 CRDs**: Custom Resource Definitions per gestire topics e servizi ROS 2
4. **HTTP API**: Interfaccia REST per interagire con il bridge

### Namespace Kubernetes

- `ros2-system`: Contiene i servizi di sistema ROS 2
- `ros2-apps`: Contiene le applicazioni ROS 2

## 🚀 Deploy

### Prerequisiti

- Cluster Kubernetes funzionante
- `kubectl` configurato
- Docker per build delle immagini

### Deploy Automatico

```bash
# Deploy completo dell'integrazione ROS 2
./scripts/deploy-ros2.sh
```

### Deploy Manuale

```bash
# 1. Creare i namespace
kubectl apply -f resources/k8s/ros2/namespace.yaml

# 2. Configurare RBAC
kubectl apply -f resources/k8s/ros2/rbac.yaml

# 3. Applicare la configurazione
kubectl apply -f resources/k8s/ros2/configmap.yaml

# 4. Deploy microROS agent
kubectl apply -f resources/k8s/ros2/microros-agent.yaml

# 5. Build e deploy del bridge
cd crates/wasmbed-microros-bridge
docker build -t wasmbed-microros-bridge:latest .
cd ../..
kubectl apply -f resources/k8s/ros2/wasmbed-microros-bridge.yaml

# 6. Deploy esempio applicazione
kubectl apply -f resources/k8s/ros2/examples/drone-ros2-app.yaml
```

## 🧪 Testing

### Test di Integrazione

```bash
# Eseguire tutti i test di integrazione
./scripts/test-ros2-integration.sh
```

### Test Manuali

```bash
# Port forwarding per accesso ai servizi
kubectl port-forward -n ros2-system svc/microros-agent 8888:8888
kubectl port-forward -n ros2-system svc/wasmbed-microros-bridge 8080:8080
kubectl port-forward -n ros2-system svc/microros-agent 9090:9090

# Test API endpoints
curl http://localhost:8080/health
curl http://localhost:8080/status
curl http://localhost:8080/topics
```

## 📡 API Reference

### Endpoints HTTP

#### Health Check
```http
GET /health
```

Risposta:
```json
{
  "status": "healthy",
  "initialized": true,
  "connected": true
}
```

#### Status
```http
GET /status
```

Risposta:
```json
{
  "initialized": true,
  "connected": true,
  "active_topics": 5,
  "error_count": 0,
  "last_heartbeat": "2025-01-10T20:40:42Z"
}
```

#### List Topics
```http
GET /topics
```

Risposta:
```json
{
  "input_topics": ["/fmu/in/vehicle_command", "/fmu/in/position_setpoint"],
  "output_topics": ["/fmu/out/vehicle_status", "/fmu/out/battery_status"]
}
```

#### Publish Message
```http
POST /topics/{topic}/publish
Content-Type: application/json

{
  "topic": "/drone/commands",
  "message_type": "geometry_msgs/Twist",
  "data": {
    "linear": {"x": 1.0, "y": 0.0, "z": 0.0},
    "angular": {"x": 0.0, "y": 0.0, "z": 0.5}
  }
}
```

#### Subscribe to Topic
```http
POST /topics/{topic}/subscribe
Content-Type: application/json

{
  "topic": "/drone/telemetry",
  "message_type": "sensor_msgs/NavSatFix",
  "callback_url": "http://wasmbed-gateway:8080/api/v1/ros2/callback"
}
```

## 🔧 Configurazione

### Variabili d'Ambiente

| Variabile | Default | Descrizione |
|-----------|---------|-------------|
| `ROS_DOMAIN_ID` | `0` | ID del dominio ROS 2 |
| `NODE_NAME` | `wasmbed_microros_bridge` | Nome del nodo ROS 2 |
| `WASMBED_GATEWAY_URL` | `http://wasmbed-gateway.wasmbed-system.svc.cluster.local:8080` | URL del gateway Wasmbed |
| `RUNTIME_ID` | `default-runtime` | ID del runtime WASM |
| `PORT` | `8080` | Porta del server HTTP |

### QoS Configuration

Il bridge supporta configurazione QoS per:
- **Reliability**: `reliable` o `best_effort`
- **Durability**: `volatile` o `transient_local`
- **History**: `keep_last` o `keep_all`
- **Depth**: Numero di messaggi da mantenere

## 📋 CRDs (Custom Resource Definitions)

### ROS2Topic

Gestisce i topic ROS 2:

```yaml
apiVersion: wasmbed.io/v1alpha1
kind: ROS2Topic
metadata:
  name: drone-telemetry
  namespace: ros2-apps
spec:
  topicName: "/drone/telemetry"
  messageType: "sensor_msgs/NavSatFix"
  qosProfile:
    reliability: "reliable"
    durability: "volatile"
    history: "keep_last"
    depth: 10
  publisher:
    enabled: true
    wasmFunction: "publish_telemetry"
  subscriber:
    enabled: false
```

### ROS2Service

Gestisce i servizi ROS 2:

```yaml
apiVersion: wasmbed.io/v1alpha1
kind: ROS2Service
metadata:
  name: drone-arm-service
  namespace: ros2-apps
spec:
  serviceName: "/drone/arm"
  serviceType: "std_srvs/Empty"
  server:
    enabled: true
    wasmFunction: "arm_drone_service"
  client:
    enabled: false
```

## 🔗 Integrazione con WASM Runtime

### Host Functions

Il bridge espone le seguenti funzioni host per le applicazioni WASM:

- `ros2_publish(topic, data)` - Pubblica un messaggio
- `ros2_subscribe(topic, callback)` - Sottoscrive a un topic
- `ros2_call_service(service, request)` - Chiama un servizio
- `ros2_create_service(service, handler)` - Crea un servizio

### Esempio di Utilizzo in WASM

```rust
// Pubblicare un messaggio
let topic = "/drone/commands";
let data = serde_json::json!({
    "linear": {"x": 1.0, "y": 0.0, "z": 0.0},
    "angular": {"x": 0.0, "y": 0.0, "z": 0.5}
});
ros2_publish(topic, data);

// Sottoscriversi a un topic
let callback = |msg| {
    println!("Received: {:?}", msg);
};
ros2_subscribe("/drone/telemetry", callback);
```

## 🐛 Troubleshooting

### Problemi Comuni

1. **microROS Agent non si avvia**
   ```bash
   kubectl logs -n ros2-system deployment/microros-agent
   ```

2. **Bridge non si connette**
   ```bash
   kubectl logs -n ros2-system deployment/wasmbed-microros-bridge
   ```

3. **Topic non trovato**
   ```bash
   kubectl get ros2topics -n ros2-apps
   ```

### Log e Debug

```bash
# Log del microROS agent
kubectl logs -n ros2-system -l app=microros-agent -f

# Log del bridge
kubectl logs -n ros2-system -l app=wasmbed-microros-bridge -f

# Status dei pod
kubectl get pods -n ros2-system -o wide
```

## 📚 Risorse Aggiuntive

- [ROS 2 Documentation](https://docs.ros.org/en/humble/)
- [microROS Documentation](https://micro.ros.org/)
- [FastDDS Documentation](https://fast-dds.docs.eprosima.com/)
- [Kubernetes Custom Resources](https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/)

## 🤝 Contribuire

Per contribuire all'integrazione ROS 2:

1. Fork del repository
2. Creare un branch per la feature
3. Implementare le modifiche
4. Aggiungere test
5. Creare una pull request

## 📄 Licenza

Questo progetto è rilasciato sotto licenza AGPL-3.0. Vedi il file LICENSE per i dettagli.
